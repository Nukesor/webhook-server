use ::actix::prelude::*;
use ::actix_web::*;
use ::serde::Deserialize;
use ::serde_json;
use ::log::{info, warn};
use ::std::str;
use ::std::collections::HashMap;

use crate::authentication::verify_authentication_header;
use crate::queue_actor::QueueActor;
use crate::settings::{get_task_from_request, Settings};

/// Initialize the web server
/// Move the address of the queue actor inside the AppState for further dispatch
/// of tasks to the actor
pub fn init_web_server(queue_actor: Addr<QueueActor>, settings: Settings) {
    let settings_for_app = settings.clone();
    HttpServer::new(move || {
        App::new()
            .data(AppState {
                queue_actor: queue_actor.clone(),
                settings: settings_for_app.clone(),
            })
            .service(web::resource("/{webhook_name}").route(web::post().to(webhook)))
    })
    .bind(format!("{}:{}", settings.domain, settings.port))
    .unwrap()
    .start();
}

/// State of the actix-web application
struct AppState {
    queue_actor: Addr<QueueActor>,
    settings: Settings,
}

#[derive(Deserialize)]
struct Payload {
    parameters: Option<HashMap<String, String>>,
}

/// Index route
fn webhook(
    data: web::Data<AppState>,
    path_info: web::Path<String>,
    request: web::HttpRequest,
    body: String,
) -> Result<HttpResponse, HttpResponse> {
    let payload = get_payload(&body)?;
    let webhook_name = path_info.into_inner();

    info!("");
    info!("Incoming webhook for \"{}\":", webhook_name);
    info!("Got payload: {}", body);

    // Check the credentials and signature headers of the request
    verify_authentication_header(&data.settings, &request, body)?;

    // Create a new task with the checked parameters and webhook name
    let new_task = get_task_from_request(&data.settings, webhook_name, payload.parameters)?;

    // Send the task to the actor managing the queue
    data.queue_actor.do_send(new_task);

    Ok(HttpResponse::Ok().finish())
}


/// We do our own json handling, since Actix doesn't allow multiple extractors at once
fn get_payload(string: &String) -> Result<Payload, HttpResponse> {
    match serde_json::from_str(&string) {
        Ok(payload) => Ok(payload),
        Err(error) => {
            let message = format!("Json error: {}", error);
            warn!("{}", message);
            Err(HttpResponse::Unauthorized().body(message))
        }
    }
}
