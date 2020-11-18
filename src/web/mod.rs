use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::str;

use actix::prelude::*;
use actix_web::*;
use anyhow::{anyhow, Result};
use config::ConfigError;
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};
use serde::Deserialize;

mod authentication;
mod helper;
mod routes;

use crate::scheduler::Scheduler;
use crate::settings::Settings;
use routes::*;

/// State of the actix-web application
pub struct AppState {
    scheduler: Addr<Scheduler>,
    settings: Settings,
}

#[derive(Deserialize, Debug, Default)]
pub struct Payload {
    parameters: Option<HashMap<String, String>>,
}

/// Initialize the web server
/// Move the address of the queue actor inside the AppState for further dispatch
/// of tasks to the actor
pub fn init_web_server(scheduler: Addr<Scheduler>, settings: Settings) -> Result<()> {
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
        let chain_path = settings
            .ssl_cert_chain
            .as_ref()
            .ok_or(ConfigError::NotFound("ssl_cert_chain".to_string()))?;
        let key_path = settings
            .ssl_private_key
            .as_ref()
            .ok_or(ConfigError::NotFound("ssl_private_key".to_string()))?;
        let cert_file = &mut BufReader::new(File::open(chain_path)?);
        let key_file = &mut BufReader::new(File::open(key_path)?);

        let cert_chain = certs(cert_file).or(Err(anyhow!("Failed to read ssl certs")))?;
        let mut keys =
            rsa_private_keys(key_file).or(Err(anyhow!("Failed to read ssl private key")))?;

        let mut config = ServerConfig::new(NoClientAuth::new());
        config.set_single_cert(cert_chain, keys.remove(0))?;

        server.bind_rustls(address, config)?.run();
    } else {
        server.bind(address)?.run();
    }

    Ok(())
}
