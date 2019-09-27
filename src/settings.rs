use ::actix_web::http::StatusCode;
use ::actix_web::HttpResponse;
use ::config::*;
use ::handlebars::Handlebars;
use ::log::info;
use ::serde::Deserialize;
use ::shellexpand::tilde;
use ::std::collections::HashMap;
use ::std::path::Path;

use crate::messages::NewTask;

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    name: String,
    command: String,
    cwd: String,
    parallel_processes: i32,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub domain: String,
    pub port: i32,
    pub ssh_cert: Option<String>,
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
            ssh_cert: self.ssh_cert.clone(),
            workers: self.workers,
            webhooks: webhooks,
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        info!("Init settings file");
        let mut settings = config::Config::default();
        settings.set_default("domain", "127.0.0.1")?;
        settings.set_default("port", "8000")?;
        settings.set_default("ssh_cert", None::<String>)?;
        settings.set_default("workers", 8)?;

        settings = parse_config(settings);

        settings.try_into()
    }

    /// Get a specific webhook from the
    pub fn get_webhook_by_name(&self, name: String) -> Result<Webhook, HttpResponse> {
        for webhook in self.webhooks.iter() {
            return Ok(webhook.clone());
        }

        Err(HttpResponse::build(StatusCode::BAD_REQUEST)
            .json(format!("Couldn't find webhook with name: {}", name)))
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
    params: HashMap<String, String>,
) -> Result<NewTask, HttpResponse> {
    let webhook = settings.get_webhook_by_name(name)?;

    let command = verify_template_parameters(webhook.command, &params)?;

    Ok(NewTask {
        name: webhook.name,
        parameters: params,
        cwd: webhook.cwd,
        command: command,
    })
}

/// Verify that the template renders with the given parameters
pub fn verify_template_parameters(
    template: String,
    params: &HashMap<String, String>,
) -> Result<String, HttpResponse> {
    info!("Got parameters: {:?}", params);
    // Create a new handlebar instance and enable strict mode to prevent missing or malformed arguments
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    // Check the template for render errors with the current parameter
    let result = handlebars.render_template(&template, params);
    match result {
        Err(error) => {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).json(format!("{:?}", error)))
        }
        Ok(result) => {
            info!("Template renders properly: {}", result);
            Ok(result)
        }
    }
}
