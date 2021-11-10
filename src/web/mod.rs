use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str;

use actix::prelude::*;
use actix_web::*;
use anyhow::{anyhow, bail, Context, Result};
use config::ConfigError;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{pkcs8_private_keys, rsa_private_keys};
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
            .app_data(AppState {
                scheduler: scheduler.clone(),
                settings: settings_for_app.clone(),
            })
            .service(web::resource("/{webhook_name}").to(webhook))
            .service(web::resource("/").to(index))
    });

    let address = format!("{}:{}", settings.domain, settings.port);

    // Load the ssl key, if something is specified in the settings
    match (settings.ssl_cert_chain, settings.ssl_private_key) {
        (Some(chain_path), Some(key_path)) => {
            let certs = load_certs(&chain_path)?;
            let key = load_key(&key_path)?;

            let config = ServerConfig::builder()
                .with_safe_default_cipher_suites()
                .with_safe_default_kx_groups()
                .with_safe_default_protocol_versions()
                .expect("Couldn't enforce TLS1.2 and TLS 1.3. This is a bug.")
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .map_err(|err| anyhow!("Failed to build TLS Acceptor: {}", err))?;

            server.bind_rustls(address, config)?.run();
        }
        (None, None) => {
            server.bind(address)?.run();
        }
        (Some(_), None) => {
            Err(ConfigError::NotFound("ssl_cert_chain".to_string()))?;
        }
        (None, Some(_)) => {
            Err(ConfigError::NotFound("ssl_cert_key".to_string()))?;
        }
    }

    Ok(())
}

/// Load the passed certificates file
fn load_certs(path: &Path) -> Result<Vec<Certificate>> {
    let file = File::open(path).context(format!("Cannot open cert {:?}", path))?;
    let certs: Vec<Certificate> = rustls_pemfile::certs(&mut BufReader::new(file))
        .context("Failed to parse daemon certificate.")?
        .into_iter()
        .map(Certificate)
        .collect();

    Ok(certs)
}

/// Load the passed keys file.
/// Only the first key will be used. It should match the certificate.
fn load_key(path: &Path) -> Result<PrivateKey> {
    let file = File::open(path).context(format!("Cannot open key {:?}", path))?;

    // Try to read pkcs8 format first
    let keys =
        pkcs8_private_keys(&mut BufReader::new(&file)).context("Failed to parse pkcs8 format.");

    if let Ok(keys) = keys {
        if let Some(key) = keys.into_iter().next() {
            return Ok(PrivateKey(key));
        }
    }

    // Try the normal rsa format afterwards.
    let keys =
        rsa_private_keys(&mut BufReader::new(file)).context("Failed to parse daemon key.")?;

    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKey(key));
    }

    bail!("Couldn't extract private key from keyfile {:?}", path)
}
