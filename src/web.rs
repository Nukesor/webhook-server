use ::actix::prelude::*;
use ::actix_web::http::header::HeaderMap;
use ::actix_web::http::Method;
use ::actix_web::http::StatusCode;
use ::actix_web::*;
use ::chrono::prelude::*;
use ::handlebars::Handlebars;
use ::log::{debug, info, warn};
use ::serde::Deserialize;
use ::serde_json;
use ::std::collections::HashMap;
use ::std::fs::File;
use ::std::io::BufReader;
use ::std::str;
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};

use crate::authentication::verify_authentication_header;
use crate::messages::{GetQueue, NewTask};
use crate::scheduler::Scheduler;
use crate::settings::Settings;

/// Initialize the web server
/// Move the address of the queue actor inside the AppState for further dispatch
/// of tasks to the actor
pub fn init_web_server(scheduler: Addr<Scheduler>, settings: Settings) {
    let settings_for_app = settings.clone();
    let server = HttpServer::new(move || {
        App::new()
            .data(AppState {
                scheduler: scheduler.clone(),
                settings: settings_for_app.clone(),
            })
            .service(web::resource("/{webhook_name}").to(webhook))
            .service(web::resource("/").to(index))
    });

    let address = format!("{}:{}", settings.domain, settings.port);

    // Load the ssl key, if something is specified in the settings
    if settings.ssl_cert_chain.is_some() && settings.ssl_private_key.is_some() {
        let chain_path = settings.ssl_cert_chain.unwrap();
        let key_path = settings.ssl_private_key.unwrap();
        let cert_file = &mut BufReader::new(File::open(chain_path).unwrap());
        let key_file = &mut BufReader::new(File::open(key_path).unwrap());

        let cert_chain = certs(cert_file).unwrap();
        let mut keys = rsa_private_keys(key_file).unwrap();

        let mut config = ServerConfig::new(NoClientAuth::new());
        config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

        server.bind_rustls(address, config).unwrap().start();
    } else {
        server.bind(address).unwrap().start();
    }
}

/// State of the actix-web application
struct AppState {
    scheduler: Addr<Scheduler>,
    settings: Settings,
}

#[derive(Deserialize, Debug, Default)]
struct Payload {
    parameters: Option<HashMap<String, String>>,
}

/// Index route for getting current state of the server
fn index(
    data: web::Data<AppState>,
    request: web::HttpRequest,
) -> Result<HttpResponse, HttpResponse> {
    let headers = get_headers_hash_map(request.headers())?;

    // Check the credentials and signature headers of the request
    verify_authentication_header(&data.settings, &headers, &Vec::new())?;

    let json = data.scheduler.send(GetQueue {}).wait().unwrap();
    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(json))
}

/// Index route
fn webhook(
    data: web::Data<AppState>,
    path_info: web::Path<String>,
    request: web::HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, HttpResponse> {
    let body: Vec<u8> = body.to_vec();
    let payload: Payload;
    match request.method() {
        &Method::POST => {
            payload = get_payload(&body)?;
        }
        _ => {
            payload = Payload::default();
        }
    }
    let headers = get_headers_hash_map(request.headers())?;

    let webhook_name = path_info.into_inner();

    // Check the credentials and signature headers of the request
    verify_authentication_header(&data.settings, &headers, &body)?;

    info!("Incoming webhook for \"{}\":", webhook_name);
    debug!("Got payload: {:?}", payload);

    // Create a new task with the checked parameters and webhook name
    let new_task = get_task_from_request(&data.settings, webhook_name, payload.parameters)?;

    // Send the task to the actor managing the queue
    data.scheduler.do_send(new_task);

    Ok(HttpResponse::Ok().finish())
}

/// We do our own json handling, since Actix doesn't allow multiple extractors at once
fn get_payload(body: &Vec<u8>) -> Result<Payload, HttpResponse> {
    match serde_json::from_slice(body) {
        Ok(payload) => Ok(payload),
        Err(error) => {
            let message = format!("Json error: {}", error);
            warn!("{}", message);
            Err(HttpResponse::Unauthorized().body(message))
        }
    }
}

fn get_headers_hash_map(map: &HeaderMap) -> Result<HashMap<String, String>, HttpResponse> {
    let mut headers = HashMap::new();

    for (key, header_value) in map.iter() {
        let key = key.as_str().to_string();
        let value: String;
        match header_value.to_str() {
            Ok(header_value) => value = header_value.to_string(),
            Err(error) => {
                let message = format!("Couldn't parse header: {}", error);
                warn!("{}", message);
                return Err(HttpResponse::Unauthorized().body(message));
            }
        };

        headers.insert(key, value);
    }

    Ok(headers)
}

pub fn get_task_from_request(
    settings: &Settings,
    name: String,
    parameters: Option<HashMap<String, String>>,
) -> Result<NewTask, HttpResponse> {
    let parameters = parameters.unwrap_or_default();

    let webhook = settings.get_webhook_by_name(&name)?;
    let command = verify_template_parameters(webhook.command, &parameters)?;

    Ok(NewTask {
        webhook_name: webhook.name,
        parameters: parameters,
        cwd: webhook.cwd,
        command: command,
        added_at: Local::now(),
    })
}

/// Verify that the template renders with the given parameters
pub fn verify_template_parameters(
    template: String,
    parameters: &HashMap<String, String>,
) -> Result<String, HttpResponse> {
    if parameters.len() != 0 {
        info!("Got parameters: {:?}", parameters);
    }
    // Create a new handlebar instance and enable strict mode to prevent missing or malformed arguments
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    // Check the template for render errors with the current parameter
    let result = handlebars.render_template(&template, parameters);
    match result {
        Err(error) => {
            warn!(
                "Error rendering command with params: {:?}. Error: {:?}",
                parameters, error
            );
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).json(format!("{:?}", error)))
        }
        Ok(result) => {
            if parameters.len() != 0 {
                info!("Template renders properly: {}", result);
            }
            Ok(result)
        }
    }
}
