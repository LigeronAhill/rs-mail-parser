mod configuration;
mod mail_client;
mod storage;
mod telemetry;
mod web_parser;
mod xlparser;

use anyhow::Result;
use surrealdb::sql::Datetime;
use tracing::{error, info};

fn main() -> Result<()> {
    // logger
    telemetry::init("info");
    // config
    let cfg = configuration::get()?;
    info!("Config initialized successfully");
    // mail client
    let mut mc = mail_client::new(cfg.mail_user(), cfg.mail_pass(), cfg.mail_host())?;
    info!("Mail client initialized successfully");
    let (tx, rx) = std::sync::mpsc::channel();
    let mtx = tx.clone();
    std::thread::spawn(move || loop {
        info!("Fetching mail attachments...");
        if let Ok(mail_attachments) = mc.fetch() {
            if !mail_attachments.is_empty() {
                let pr = xlparser::parse(mail_attachments);
                if mtx.send(pr).is_err() {
                    error!("Error sending parsing result...");
                }
            }
        }
        info!("Emails fetched successfully. Paused for an hour");
        std::thread::sleep(std::time::Duration::from_secs(60 * 60));
    });
    // web parser
    let wp = web_parser::new(cfg.ort_user(), cfg.ort_pass())?;
    std::thread::spawn(move || loop {
        info!("Fetching web attachments...");
        if let Ok(ort) = wp.ortgraph() {
            let mut m = std::collections::HashMap::new();
            m.insert("ortgraph".to_string(), (ort, Datetime(chrono::Utc::now())));
            let pr = xlparser::parse(m);
            if tx.send(pr).is_err() {
                error!("Error sending parsing result...");
            }
        }
        if let Ok(vvk) = wp.vvk() {
            let mut m = std::collections::HashMap::new();
            m.insert("vvk".to_string(), (vvk, Datetime(chrono::Utc::now())));
            let pr = xlparser::parse(m);
            if tx.send(pr).is_err() {
                error!("Error sending parsing result...");
            }
        }
        info!("Web attachments fetched successfully. Paused for a day");
        std::thread::sleep(std::time::Duration::from_secs(60 * 60 * 24));
    });
    // db
    // TODO: add credentials
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        match storage::new("root", "root", "test", "test").await {
            Ok(db) => {
                info!("DB client initialized successfully");
                loop {
                    if let Ok(pr) = rx.recv() {
                        for r in pr {
                            let supplier = r.supplier.clone();
                            match db.update(r).await {
                                Ok(_) => info!("Stock of {} updated successfully", supplier),
                                Err(e) => error!("Error updating {} stock: {e:?}", supplier),
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Error initializing DB client: {e:?}"),
        }
    });
    Ok(())
}
