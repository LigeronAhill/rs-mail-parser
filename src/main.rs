mod telemetry;
mod configuration;
mod mail_client;
mod web_parser;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // logger
    telemetry::init("info");
    // config
    let cfg = configuration::get()?;
    info!("Config initialized successfully");
    // mail client
    let mc = mail_client::new(cfg.mail_user(), cfg.mail_pass(), cfg.mail_host()).await?;
    info!("Mail client initialized successfully");
    let attachments_map = mc.fetch().await?;
    for (s, a) in attachments_map {
        info!("Got {} attachments from '{s}'", a.len())
    }
    // web parser
    let wp = web_parser::new(cfg.ort_user(), cfg.ort_pass())?;
    let ort = wp.ortgraph().await?;
    let vvk = wp.vvk().await?;
    info!("Got {} ortgraph attachments and {} vvk attachments", ort.len(), vvk.len());
    // TODO: init excel parser
    // TODO: init sender
    // TODO: init app
    // TODO: run app
    Ok(())
}
