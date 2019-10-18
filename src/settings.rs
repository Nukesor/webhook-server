use ::actix_web::http::StatusCode;
use ::actix_web::HttpResponse;
use ::chrono::prelude::*;
use ::config::*;
use ::handlebars::Handlebars;
use ::log::{info, warn};
use ::serde::Deserialize;
use ::shellexpand::tilde;
use ::std::collections::HashMap;
use ::std::path::Path;
use ::std::process;

use crate::messages::NewTask;

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub name: String,
    pub command: String,
    pub cwd: String,
    #[serde(default = "webhook_mode_default")]
    pub mode: String,
    #[serde(default = "webhook_parallel_default")]
    pub parallel_processes: i32,
}

fn webhook_mode_default() -> String {
    "deploy".to_string()
}

fn webhook_parallel_default() -> i32 {
    8
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub domain: String,
    pub port: i32,
    pub ssl_private_key: Option<String>,
    pub ssl_cert_chain: Option<String>,
    pub secret: Option<String>,
    pub basic_auth_user: Option<String>,
    pub basic_auth_password: Option<String>,
    pub basic_auth_and_secret: bool,
    pub workers: usize,
    pub webhooks: Vec<Webhook>,
}

impl Clone for Settings {
    fn clone(&self) -> Self {
        let mut webhooks: Vec<Webhook> = Vec::new();
        for hook in self.webhooks.iter() {
            webhooks.push(hook.clone());
        }
        Settings {
            domain: self.domain.clone(),
            port: self.port,
            ssl_private_key: self.ssl_private_key.clone(),
            ssl_cert_chain: self.ssl_cert_chain.clone(),
            secret: self.secret.clone(),
            basic_auth_user: self.basic_auth_user.clone(),
            basic_auth_password: self.basic_auth_password.clone(),
            basic_auth_and_secret: self.basic_auth_and_secret,
            workers: self.workers,
            webhooks: webhooks,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        info!("Init settings file");
        let mut settings = config::Config::default();
        settings.set_default("domain", "127.0.0.1").unwrap();
        settings.set_default("port", "8000").unwrap();
        settings
            .set_default("ssl_private_key", None::<String>)
            .unwrap();
        settings
            .set_default("ssl_cert_chain", None::<String>)
            .unwrap();
        settings.set_default("workers", 8).unwrap();
        settings.set_default("secret", None::<String>).unwrap();
        settings
            .set_default("basic_auth_user", None::<String>)
            .unwrap();
        settings
            .set_default("basic_auth_password", None::<String>)
            .unwrap();
        settings
            .set_default("basic_auth_and_secret", false)
            .unwrap();

        settings = parse_config(settings);
        let settings: Settings = match settings.try_into() {
            Ok(settings) => settings,
            Err(err) => {
                println!("Error parsing settings: {:?}", err);
                process::exit(1);
            }
        };

        // Verify that everything is in place, if `basic_auth_and_secret` is activated
        if settings.basic_auth_and_secret
            && (settings.secret.is_none()
                || settings.basic_auth_user.is_none()
                || settings.basic_auth_password.is_none())
        {
            println!("If basic_auth_and_secret is true, all three values must be specified in your config");
            process::exit(1);
        }

        // Webhook mode must be a valid
        for webhook in &settings.webhooks {
            if webhook.mode == "single" || webhook.mode == "deploy" || webhook.mode == "parallel" {
                break;
            }
            println!(
                "Webhook mode must be one of 'single', 'deploy' or 'parallel'. Yours: {}",
                webhook.name
            );
            process::exit(1);
        }

        settings
    }

    /// Get settings for a specific webhook
    pub fn get_webhook_by_name(&self, name: &String) -> Result<Webhook, HttpResponse> {
        for webhook in self.webhooks.iter() {
            if &webhook.name == name {
                return Ok(webhook.clone());
            }
        }

        let error = format!("Couldn't find webhook with name: {}", name);
        warn!("{}", error);
        Err(HttpResponse::build(StatusCode::BAD_REQUEST).json(error))
    }
}

fn parse_config(mut settings: Config) -> Config {
    let config_paths = [
        "/etc/webhook_server.yml",
        &tilde("~/.config/webhook_server.yml").into_owned(),
        "./webhook_server.yml",
    ];
    info!("Parsing config files");

    for path in config_paths.into_iter() {
        info!("Checking path: {}", path);
        if Path::new(path).exists() {
            info!("Parsing config file at: {}", path);
            settings.merge(config::File::with_name(path)).unwrap();
        }
    }

    settings
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
    info!("Got parameters: {:?}", parameters);
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
            info!("Template renders properly: {}", result);
            Ok(result)
        }
    }
}
