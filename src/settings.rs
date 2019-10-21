use ::actix_web::http::StatusCode;
use ::actix_web::HttpResponse;
use ::anyhow::{anyhow, Result};
use ::config::ConfigError;
use ::config::*;
use ::log::{info, warn};
use ::serde::Deserialize;
use ::std::path::{Path, PathBuf};

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
    pub fn new() -> Result<Self> {
        info!("Init settings file");
        let mut settings = config::Config::default();
        settings.set_default("domain", "127.0.0.1")?;
        settings.set_default("port", "8000")?;
        settings.set_default("ssl_private_key", None::<String>)?;
        settings.set_default("ssl_cert_chain", None::<String>)?;
        settings.set_default("workers", 8)?;
        settings.set_default("secret", None::<String>)?;
        settings.set_default("basic_auth_user", None::<String>)?;
        settings.set_default("basic_auth_password", None::<String>)?;
        settings.set_default("basic_auth_and_secret", false)?;

        settings = parse_config(settings)?;
        let settings: Settings = settings.try_into()?;

        if settings.basic_auth_password.is_some() || settings.basic_auth_user.is_some() {
            settings
                .basic_auth_user
                .as_ref()
                .ok_or(ConfigError::NotFound("basic_auth_user".to_string()))?;
            settings
                .basic_auth_password
                .as_ref()
                .ok_or(ConfigError::NotFound("basic_auth_password".to_string()))?;
        }

        // Verify that everything is in place, if `basic_auth_and_secret` is activated
        if settings.basic_auth_and_secret {
            settings
                .secret
                .as_ref()
                .ok_or(ConfigError::NotFound("secret".to_string()))?;
            settings
                .basic_auth_user
                .as_ref()
                .ok_or(ConfigError::NotFound("basic_auth_user".to_string()))?;
            settings
                .basic_auth_password
                .as_ref()
                .ok_or(ConfigError::NotFound("basic_auth_password".to_string()))?;
        }

        // Webhook mode must be a valid
        for webhook in &settings.webhooks {
            if webhook.mode == "single" || webhook.mode == "deploy" || webhook.mode == "parallel" {
                continue;
            }
            return Err(anyhow!(
                "Webhook mode must be one of 'single', 'deploy' or 'parallel'. Yours: {}",
                webhook.name
            ));
        }

        Ok(settings)
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

fn parse_config(mut settings: Config) -> Result<Config> {
    info!("Parsing config files");
    let config_paths = get_config_paths()?;

    for path in config_paths.into_iter() {
        info!("Checking path: {:?}", &path);
        if path.exists() {
            info!("Parsing config file at: {:?}", path);
            let config_file = config::File::with_name(path.to_str().unwrap());
            settings.merge(config_file)?;
        }
    }

    Ok(settings)
}

#[cfg(target_os = "linux")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let home_dir = dirs::home_dir().ok_or(anyhow!("Couldn't resolve home dir"))?;
    paths.push(Path::new("/etc/webhook_server.yml").to_path_buf());
    paths.push(home_dir.join(".config/webhook_server.yml"));
    paths.push(Path::new("./webhook_server.yml").to_path_buf());

    Ok(paths)
}

#[cfg(target_os = "windows")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    let home_dir = dirs::home_dir().ok_or(anyhow!("Couldn't resolve home dir"))?;
    paths.push(home_dir.join("AppData\\Roaming\\webhook_server\\webhook_server.yml"));
    paths.push(Path::new(".\\webhook_server.yml").to_path_buf());

    Ok(paths)
}

#[cfg(target_os = "macos")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    let home_dir = dirs::home_dir().ok_or(anyhow!("Couldn't resolve home dir"))?;
    paths.push(home_dir.join("Library/Application Support/webhook_server.yml"));
    paths.push(home_dir.join("Library/Preferences/webhook_server.yml"));
    paths.push(Path::new("./webhook_server.yml").to_path_buf());

    Ok(paths)
}
