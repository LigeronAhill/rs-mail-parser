mod telemetry;
mod configuration;
mod mail_client;
mod web_parser;
mod xlparser;

use anyhow::Result;
use rayon::prelude::*;
use tracing::info;

fn main() -> Result<()> {
    // logger
    telemetry::init("info");
    // config
    let cfg = configuration::get()?;
    info!("Config initialized successfully");
    // mail client
    let mut mc = mail_client::new(cfg.mail_user(), cfg.mail_pass(), cfg.mail_host())?;
    info!("Mail client initialized successfully");
    let mut attachments_map = mc.fetch()?;
    // web parser
    let wp = web_parser::new(cfg.ort_user(), cfg.ort_pass())?;
    let ort = wp.ortgraph()?;
    attachments_map.insert(String::from("ortgraph"), ort);
    let vvk = wp.vvk()?;
    attachments_map.insert(String::from("vvk"), vvk);
    for (s, a) in &attachments_map {
        info!("Got {} attachments from '{s}'", a.len())
    }
    let opus_attachments = attachments_map.into_par_iter().find_first(|(s, _)| s.to_lowercase().contains("opus")).map(|(_, a)| a).unwrap_or_default();
    let opus = xlparser::opus::parser(opus_attachments);
    info!("{opus:#?}");
    // TODO: init sender
    // TODO: init app
    // TODO: run app
    Ok(())
}
