use actix_web::http::Method;
use actix_web::*;
use actix_web::{http, HttpResponse};
use log::{debug, info};

use crate::messages::GetQueue;
use crate::web::authentication::verify_authentication_header;
use crate::web::helper::*;
use crate::web::{AppState, Payload};

/// Index route for getting current state of the server
pub async fn index(data: web::Data<AppState>, request: web::HttpRequest) -> HttpResponse {
    let headers = match get_headers_hash_map(request.headers()) {
        Ok(headers) => headers,
        Err(response) => return response,
    };

    // Check the credentials and signature headers of the request
    if let Err(response) = verify_authentication_header(&data.settings, &headers, &Vec::new()) {
        return response;
    };

    match data.scheduler.send(GetQueue {}).await {
        Ok(json) => HttpResponse::Ok()
            .append_header((http::header::CONTENT_TYPE, "application/json"))
            .body(json),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Index route
pub async fn webhook(
    data: web::Data<AppState>,
    path_info: web::Path<String>,
    request: web::HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    let body: Vec<u8> = body.to_vec();
    let payload = match request.method() {
        &Method::POST => match get_payload(&body) {
            Ok(payload) => payload,
            Err(response) => return response,
        },
        _ => Payload::default(),
    };

    let headers = match get_headers_hash_map(request.headers()) {
        Ok(headers) => headers,
        Err(response) => return response,
    };

    let webhook_name = path_info.into_inner();

    // Check the credentials and signature headers of the request
    if let Err(response) = verify_authentication_header(&data.settings, &headers, &body) {
        return response;
    };

    info!("Incoming webhook for \"{}\":", webhook_name);
    debug!("Got payload: {:?}", payload);

    // Create a new task with the checked parameters and webhook name
    let new_task = match get_task_from_request(&data.settings, webhook_name, payload.parameters) {
        Ok(task) => task,
        Err(response) => return response,
    };

    // Send the task to the actor managing the queue
    data.scheduler.do_send(new_task);

    HttpResponse::Ok().finish()
}
