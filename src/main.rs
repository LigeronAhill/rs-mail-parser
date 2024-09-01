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
    let mut attachments_map = mc.fetch().await?;
    // web parser
    let wp = web_parser::new(cfg.ort_user(), cfg.ort_pass())?;
    let ort = wp.ortgraph().await?;
    attachments_map.insert(String::from("ortgraph"), ort);
    let vvk = wp.vvk().await?;
    attachments_map.insert(String::from("vvk"), vvk);
    for (s, a) in attachments_map {
        info!("Got {} attachments from '{s}'", a.len())
    }
    // TODO: init excel parser
    // TODO: init sender
    // TODO: init app
    // TODO: run app
    Ok(())
}
