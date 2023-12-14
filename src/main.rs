use figment::Figment;
use figment::providers::{Env, Format, Yaml};
use kube::Client;
use serde::{Deserialize, Serialize};
use tonic::transport::Server;
use tracing::{debug, info};
use tracing::level_filters::LevelFilter;

use atty::Stream;
use ingress::IngressHandler;

use crate::protogen::extension::extension_server::ExtensionServer;

mod protogen;
mod ingress;
mod logging;

#[derive(Debug, Deserialize, Serialize)]
pub enum LogFormat {
    Plain,
    Json,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl Into<LevelFilter> for &LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
        }
    }
}

impl Default for LogFormat {
    fn default() -> Self {
        match atty::is(Stream::Stdout) {
            true => LogFormat::Plain,
            false => LogFormat::Json,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Logging format to use
    #[serde(default)]
    log_format: LogFormat,
    // Log level
    log_level: LogLevel,
    // Dry-run
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let defaults = "\
log_level: Info
dry_run: false
    ";
    let config: Config = Figment::new()
        .merge(Yaml::string(defaults))
        .merge(Yaml::file("suffiks-ingress.yaml"))
        .merge(Env::prefixed("SUFFIKS_INGRESS__").split("__"))
        .extract()?;
    logging::init_logging(&config)?;
    let bin = env!("CARGO_BIN_NAME");
    let version = option_env!("VERSION").unwrap_or("unknown");
    info!("{} {} starting up", bin, version);
    info!("Config: {:?}", config);
    debug!("Creating kube client");
    let client = Client::try_default().await?;
    // creating a service
    let ingress = IngressHandler::new(client, config.dry_run);
    // adding our service to our server.
    let addr = "0.0.0.0:8080".parse().unwrap();  // TODO: Get from config
    info!("Server starting to listen on {}", addr);
    Server::builder()
        .add_service(ExtensionServer::new(ingress))
        .serve(addr)
        .await?;
    Ok(())
}
