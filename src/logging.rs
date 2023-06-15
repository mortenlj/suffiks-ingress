use anyhow::Result;
use tracing::info;
use tracing_subscriber::filter;
use tracing_subscriber::prelude::*;

use crate::{Config, LogFormat};

pub fn init_logging(config: &Config) -> Result<()> {
    let filter = filter::Targets::new()
        .with_default(&config.log_level);

    use tracing_subscriber::fmt as layer_fmt;
    let (plain_log_format, json_log_format) = match config.log_format {
        LogFormat::Plain => (Some(layer_fmt::layer().compact()), None),
        LogFormat::Json => (None, Some(layer_fmt::layer().json().flatten_event(true))),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(plain_log_format)
        .with(json_log_format)
        .init();

    info!("{:?} logger initialized", config.log_format);
    Ok(())
}
