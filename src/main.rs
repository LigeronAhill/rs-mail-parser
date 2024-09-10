mod telemetry;
mod configuration;
mod mail_client;
mod web_parser;
mod xlparser;
mod storage;

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
    let mu = cfg.mail_user().to_string();
    let mp = cfg.mail_pass().to_string();
    let h = cfg.mail_host().to_string();
    let mut mc = mail_client::new(&mu, &mp, &h)?;
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
    let opus_attachments = attachments_map.par_iter().find_first(|(s, _)| s.to_lowercase().contains("opus")).map(|(_, a)| a.to_vec()).unwrap_or_default();
    let opus = xlparser::opus::parser(opus_attachments);
    let fancy_attachments = attachments_map.par_iter().find_first(|(s, _)| s.to_lowercase().contains("fancy")).map(|(_, a)| a.to_vec()).unwrap_or_default();
    let fancy = xlparser::fancy::parser(fancy_attachments);
    // db client
    // TODO: add credentials
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        if let Ok(storage) = storage::new("root", "root", "test", "test").await {
            info!("DB client initialized successfully");
            match storage.update(opus).await {
                Ok(_) => info!("Opus stock updated successfully"),
                Err(e) => tracing::error!("Error while updating opus stock: {e:?}"),
            }
            match storage.update(fancy).await {
                Ok(_) => info!("Fancy stock updated successfully"),
                Err(e) => tracing::error!("Error while updating fancy stock: {e:?}"),
            }
        }
    });
    // TODO: init app
    // TODO: run app
    Ok(())
}