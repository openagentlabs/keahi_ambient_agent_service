use anyhow::Result;
use clap::Parser;
use signal_manager_service::config::{init_config, get_config};
use signal_manager_service::server::WebSocketServer;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_appender::non_blocking;
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use tracing_subscriber::fmt::writer::BoxMakeWriter;

struct MultiWriter<W1, W2> {
    w1: W1,
    w2: W2,
}

impl<W1: Write, W2: Write> Write for MultiWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let r1 = self.w1.write(buf);
        let r2 = self.w2.write(buf);
        match (&r1, &r2) {
            (Ok(n1), Ok(n2)) => Ok(std::cmp::min(*n1, *n2)),
            (Err(e), _) => Err(io::Error::new(e.kind(), e.to_string())),
            (_, Err(e)) => Err(io::Error::new(e.kind(), e.to_string())),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.w1.flush()?;
        self.w2.flush()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<()> {
    // Start the tokio runtime manually
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async_main())
}

async fn async_main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize configuration
    init_config(args.config.as_deref())?;
    let config = get_config();

    // Set up GCP authentication
    config.setup_gcp_auth()?;
    info!("GCP authentication configured with credentials from: {}", config.gcp.credentials_path);

    // Create logs directory if it doesn't exist
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        fs::create_dir_all(logs_dir)?;
        info!("Created logs directory: {:?}", logs_dir);
    }

    // Initialize logging based on configuration
    let log_level = config.logging.level.parse::<Level>().unwrap_or(Level::INFO);
    let env_filter = EnvFilter::from_default_env()
        .add_directive(format!("signal_manager_service={log_level}").parse()?);

    let subscriber = if config.logging.file_output && config.logging.console_output {
        let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "signal-manager-service.log");
        let (non_blocking, _guard) = non_blocking(file_appender);
        fmt()
            .with_env_filter(env_filter)
            .with_writer(BoxMakeWriter::new(move || MultiWriter {
                w1: std::io::stdout(),
                w2: non_blocking.clone(),
            }))
            .finish()
    } else if config.logging.file_output {
        let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "signal-manager-service.log");
        let (non_blocking, _guard) = non_blocking(file_appender);
        fmt()
            .with_env_filter(env_filter)
            .with_writer(BoxMakeWriter::new(move || non_blocking.clone()))
            .finish()
    } else {
        fmt()
            .with_env_filter(env_filter)
            .with_writer(BoxMakeWriter::new(|| std::io::stdout()))
            .finish()
    };
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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
