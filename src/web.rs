use ::actix::prelude::*;
use ::actix_web::*;

use ::log::info;
use ::std::collections::HashMap;

use crate::queue_actor::QueueActor;
use crate::settings::{Settings, get_task_from_request};

/// State of the actix-web application
struct AppState {
    queue_actor: Addr<QueueActor>,
    settings: Settings,
}

/// Index route
fn webhook(
    data: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
    path_info: web::Path<String>,
) -> Result<HttpResponse, HttpResponse> {
    // Verify that the parameters match the required parameters in the template string
    let params = query.into_inner();
    let webhook_name = path_info.into_inner();

    info!("");
    info!("Incoming webhook for \"{}\":", webhook_name);

    // Create a new task with the checked parameters and webhook name
    let new_task = get_task_from_request(&data.settings, webhook_name, params)?;

    // Send the task to the actor managing the queue
    data.queue_actor.do_send(new_task);

    Ok(HttpResponse::Ok().finish())
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
            .service(web::resource("/{webhook_name}").to(webhook))
    })
    .bind(format!("{}:{}", settings.domain, settings.port))
    .unwrap()
    .start();
}

