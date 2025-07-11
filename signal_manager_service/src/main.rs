use anyhow::Result;
use clap::Parser;
use signal_manager_service::config::{init_config, get_config};
use signal_manager_service::server::WebSocketServer;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize configuration
    init_config(args.config.as_deref())?;
    let config = get_config();

    // Set up GCP authentication
    config.setup_gcp_auth()?;
    info!("GCP authentication configured with credentials from: {}", config.gcp.credentials_path);

    // Initialize logging based on configuration
    let log_level = config.logging.level.parse::<Level>().unwrap_or(Level::INFO);
    let env_filter = EnvFilter::from_default_env()
        .add_directive(format!("signal_manager_service={}", log_level).parse()?);

    let mut builder = fmt::Subscriber::builder()
        .with_env_filter(env_filter);

    if config.logging.console_output {
        builder = builder.with_target(false);
    }

    if let Some(ref file_path) = config.logging.file_path {
        // Note: File logging would require additional setup with tracing-appender
        // For now, we'll just log to console
        info!("File logging configured for: {}", file_path);
    }

    builder.init();

    info!("Starting Signal Manager Service...");
    info!("Server will listen on: {}", config.socket_addr());
    info!("Metrics enabled: {}", config.metrics.enabled);
    if config.metrics.enabled {
        info!("Metrics will be available on: {}", config.metrics_addr());
    }

    // Create and start the WebSocket server
    let server = WebSocketServer::new(config.clone())?;
    
    info!("WebSocket server initialized, starting to listen...");
    
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
