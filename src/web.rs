use ::std::process;
use ::actix::prelude::*;
use ::actix_web::http::header::HeaderMap;
use ::actix_web::http::Method;
use ::actix_web::*;
use ::log::{debug, info, warn};
use ::openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use ::serde::Deserialize;
use ::serde_json;
use ::std::collections::HashMap;
use ::std::path::Path;
use ::std::str;

use crate::authentication::verify_authentication_header;
use crate::scheduler::Scheduler;
use crate::settings::{get_task_from_request, Settings};

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
    });

    let address = format!("{}:{}", settings.domain, settings.port);

    // Load the ssl key, if something is specified in the settings
    if settings.ssl_cert_chain.is_some() && settings.ssl_private_key.is_some() {
        let builder = get_ssl_builder(&settings);
        server.bind_ssl(address, builder).unwrap().start();
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
        },
        _ => {
            payload = Payload::default();
        },
    }
    let headers = get_headers_hash_map(request.headers())?;

    let webhook_name = path_info.into_inner();

    // Check the credentials and signature headers of the request
    verify_authentication_header(&data.settings, &headers, &body)?;

    info!("");
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

fn get_ssl_builder(settings: &Settings) -> SslAcceptorBuilder {
    info!("Initializing SSL");
    // At this point we already know that these have to be Some, thereby just unwrap
    let private_path_str = settings.ssl_private_key.clone().unwrap();
    let cert_path_str = settings.ssl_cert_chain.clone().unwrap();

    // Ensure both files exist
    let private_path = Path::new(&private_path_str);
    let cert_path = Path::new(&cert_path_str);
    if !private_path.exists() {
        println!(
            "Path to private key file is not correct: {}",
            private_path_str
        );
        process::exit(1);
    }
    if !cert_path.exists() {
        println!("Path to cert chain file is not correct: {}", cert_path_str);
        process::exit(1);
    }

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(private_path, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(cert_path).unwrap();

    builder
}
