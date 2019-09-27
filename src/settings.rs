use ::config::*;
use ::serde::Deserialize;
use ::shellexpand::tilde;
use ::std::path::Path;


#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    name: String,
    command: String,
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
        let mut settings = config::Config::default();
        settings.set_default("domain", "127.0.0.1")?;
        settings.set_default("port", "8000")?;
        settings.set_default("ssh_cert", None::<String>)?;
        settings.set_default("workers", 8)?;

        settings = parse_config(settings);

        settings.try_into()
    }
}


fn parse_config(mut settings: Config) -> Config {
    let config_paths = [
        "/etc/webhook_server.yml",
        &tilde("~/.config/webhook_server.yml").into_owned(),
        "./webhook_server.yml",
    ];

    for path in config_paths.into_iter() {
        if Path::new(path).exists() {
            settings.merge(config::File::with_name(path)).unwrap();
        }
    }

    settings
}
