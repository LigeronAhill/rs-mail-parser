use serde::Deserialize;
use anyhow::Result;
use secrecy::{ExposeSecret, SecretBox};
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    send_host: SecretBox<String>,
    ortgraph: Ortgraph,
    mail: Mail,
}
#[derive(Debug, Deserialize)]
pub struct Ortgraph {
    username: SecretBox<String>,
    password: SecretBox<String>,
}
#[derive(Debug, Deserialize)]
pub struct Mail {
    host: SecretBox<String>,
    user: SecretBox<String>,
    pass: SecretBox<String>,
}
#[instrument]
pub fn get() -> Result<Configuration> {
    let settings = config::Config::builder().add_source(config::File::with_name("configuration.toml")).build()?;
    let configuration: Configuration = settings.try_deserialize()?;
    Ok(configuration)
}

impl Configuration {
    pub fn host(&self) -> &str {
        &self.send_host.expose_secret()
    }
    pub fn ort_user(&self) -> &str {
        &self.ortgraph.username.expose_secret()
    }
    pub fn ort_pass(&self) -> &str {
        &self.ortgraph.password.expose_secret()
    }
    pub fn mail_host(&self) -> &str {
        &self.mail.host.expose_secret()
    }
    pub fn mail_user(&self) -> &str {
        &self.mail.user.expose_secret()
    }
    pub fn mail_pass(&self) -> &str {
        &self.mail.pass.expose_secret()
    }
}