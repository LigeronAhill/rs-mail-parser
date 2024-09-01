use std::str::FromStr;

pub fn init(level: &str) {
    let lvl = tracing::Level::from_str(level).unwrap_or(tracing::Level::INFO);
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(lvl)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    tracing::info!("Starting up");
    tracing::warn!("Are you sure this is a good idea?");
    tracing::error!("This is an error!");
    tracing::debug!("And debug goes to...")
}