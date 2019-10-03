use ::actix::prelude::*;
use ::actix_web::*;
use ::serde::Deserialize;

use ::log::{info,warn};
use ::std::collections::HashMap;

use crate::queue_actor::QueueActor;
use crate::settings::{get_task_from_request, Settings};

/// State of the actix-web application
struct AppState {
    queue_actor: Addr<QueueActor>,
    settings: Settings,
}

#[derive(Deserialize)]
struct Payload {
    secret: Option<String>,
    parameters: Option<HashMap<String, String>>,
}


/// Get index route
fn webhook(
    data: web::Data<AppState>,
    path_info: web::Path<String>,
    json: web::Json<Payload>,
) -> Result<HttpResponse, HttpResponse> {
    // Verify that the parameters match the required parameters in the template string
    let payload = json.into_inner();
    let webhook_name = path_info.into_inner();

    info!("");
    info!("Incoming webhook for \"{}\":", webhook_name);

    verify_secret(&payload, &data)?;

    // Create a new task with the checked parameters and webhook name
    let new_task = get_task_from_request(&data.settings, webhook_name, payload.parameters)?;

    // Send the task to the actor managing the queue
    data.queue_actor.do_send(new_task);

    Ok(HttpResponse::Ok().finish())
}


// If a secret is specified in the config file, verify, that the secret exists in the payload
fn verify_secret(payload: &Payload, data: &web::Data<AppState>) -> Result<(), HttpResponse> {
    let secret: String;
    // Accept the payload, if no secret is verified or the secret is an empty string.
    match data.settings.secret.as_ref() {
        Some(value) => {
            secret = value.clone();
            if secret == "" {
                return Ok(())
            }
        },
        None => {
            return Ok(());
        },
    }

    // At this point we know that a secret is required. Check if it exists in the payload
    match payload.secret.as_ref() {
        Some(payload_secret) => {
            if *payload_secret == secret {
                return Ok(())
            }
            warn!("Wrong secret: {}, parameters: {:?}", payload_secret, payload.parameters);
            Err(HttpResponse::Unauthorized()
                .body("Wrong secret"))
        },
        None => {
            warn!("Call with no secret: {:?}", payload.parameters);
            Err(HttpResponse::Unauthorized()
                .body("No secret specified"))
        }
    }
}



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
