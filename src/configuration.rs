use anyhow::Result;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    ortgraph: Ortgraph,
    mail: Mail,
    db: Db,
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
#[derive(Debug, Deserialize)]
pub struct Db {
    user: SecretBox<String>,
    pass: SecretBox<String>,
    addr: SecretBox<String>,
}
#[instrument]
pub fn get() -> Result<Configuration> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration.toml"))
        .build()?;
    let configuration: Configuration = settings.try_deserialize()?;
    Ok(configuration)
}

impl Configuration {
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
    pub fn db_user(&self) -> &str {
        &self.db.user.expose_secret()
    }
    pub fn db_pass(&self) -> &str {
        &self.db.pass.expose_secret()
    }
    pub fn db_addr(&self) -> &str {
        &self.db.addr.expose_secret()
    }
}
