use actix_web::http::Method;
use actix_web::*;
use actix_web::{http, HttpResponse};
use log::{debug, info};

use crate::messages::GetQueue;
use crate::web::authentication::verify_authentication_header;
use crate::web::helper::*;
use crate::web::{AppState, Payload};

/// Index route for getting current state of the server
pub async fn index(
    data: web::Data<AppState>,
    request: web::HttpRequest,
) -> Result<HttpResponse, HttpResponse> {
    let headers = get_headers_hash_map(request.headers())?;

    // Check the credentials and signature headers of the request
    verify_authentication_header(&data.settings, &headers, &Vec::new())?;

    let json = data
        .scheduler
        .send(GetQueue {})
        .await
        .or(Err(HttpResponse::InternalServerError()))?;
    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(json))
}

/// Index route
pub async fn webhook(
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
