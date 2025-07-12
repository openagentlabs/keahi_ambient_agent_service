use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::collections::HashMap;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub session: SessionConfig,
    pub security: SecurityConfig,
    pub gcp: GcpConfig,
    pub firestore: FirestoreConfig,
    pub cloudflare: CloudflareConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub heartbeat_interval: u64,
    pub tls_enabled: bool,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub read_buffer_size: usize,
    pub write_buffer_size: usize,
    pub max_message_size: usize,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token_secret: String,
    pub token_expiry: u64,
    pub auth_method: String,
    pub api_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: Option<String>,
    pub console_output: bool,
    pub max_file_size: usize,
    pub max_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    pub host: String,
    pub connection_stats_interval: u64,
    pub message_stats_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout: u64,
    pub cleanup_interval: u64,
    pub max_sessions_per_client: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub rate_limit_enabled: bool,
    pub max_messages_per_minute: usize,
    pub max_connections_per_ip: usize,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpConfig {
    /// Path to the GCP service account key file
    pub credentials_path: String,
    /// GCP Project ID
    pub project_id: String,
    /// GCP region (e.g., "europe-west2" for London)
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirestoreConfig {
    /// Firestore database name
    pub database_name: String,
    /// Firestore collection name for registered clients
    pub collection_name: String,
    /// Firestore project ID (inherited from GCP config)
    pub project_id: String,
    /// Firestore region (inherited from GCP config)
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare Realtime App ID
    pub app_id: String,
    /// Cloudflare Realtime App Secret
    pub app_secret: String,
    /// Cloudflare Realtime API base URL
    pub base_url: String,
    /// Cloudflare STUN server URL
    pub stun_url: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::File::with_name("app-config").required(false))
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("SIGNAL_MANAGER"))
            .build()?;

        settings.try_deserialize()
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid socket address")
    }

    pub fn metrics_addr(&self) -> SocketAddr {
        format!("{}:{}", self.metrics.host, self.metrics.port)
            .parse()
            .expect("Invalid metrics socket address")
    }

    pub fn parse_api_keys(&self) -> HashMap<String, String> {
        let mut keys = HashMap::new();
        for key_pair in &self.auth.api_keys {
            if let Some((client_id, token)) = key_pair.split_once(':') {
                keys.insert(client_id.to_string(), token.to_string());
            }
        }
        keys
    }

    /// Set up GCP authentication using the configured credentials path
    pub fn setup_gcp_auth(&self) -> Result<(), std::env::VarError> {
        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", &self.gcp.credentials_path);
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 1000,
                heartbeat_interval: 30,
                tls_enabled: false,
                tls_cert_path: "".to_string(),
                tls_key_path: "".to_string(),
                read_buffer_size: 8192,
                write_buffer_size: 8192,
                max_message_size: 1048576,
            },

            auth: AuthConfig {
                token_secret: "your-secret-key-change-in-production".to_string(),
                token_expiry: 3600,
                auth_method: "token".to_string(),
                api_keys: vec![
                    "test_client_1:test_token_1".to_string(),
                    "test_client_2:test_token_2".to_string(),
                ],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_path: None,
                console_output: true,
                max_file_size: 10485760,
                max_files: 5,
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9090,
                host: "127.0.0.1".to_string(),
                connection_stats_interval: 60,
                message_stats_interval: 30,
            },
            session: SessionConfig {
                session_timeout: 3600,
                cleanup_interval: 300,
                max_sessions_per_client: 1,
            },
            security: SecurityConfig {
                rate_limit_enabled: true,
                max_messages_per_minute: 1000,
                max_connections_per_ip: 10,
                allowed_origins: vec!["*".to_string()],
            },
            gcp: GcpConfig {
                credentials_path: "/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json".to_string(),
                project_id: "your-gcp-project-id".to_string(),
                region: "europe-west2".to_string(),
            },
            firestore: FirestoreConfig {
                database_name: "signal-manager-service-db".to_string(),
                collection_name: "registered_clients".to_string(),
                project_id: "your-gcp-project-id".to_string(),
                region: "europe-west2".to_string(),
            },
            cloudflare: CloudflareConfig {
                app_id: "your-cloudflare-app-id".to_string(),
                app_secret: "your-cloudflare-app-secret".to_string(),
                base_url: "https://rtc.live.cloudflare.com/v1".to_string(),
                stun_url: "stun:stun.cloudflare.com:3478".to_string(),
            },
        }
    }
}

// Global configuration accessor
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        Config::load("app-config.toml")
            .or_else(|_| Config::load("config.toml"))
            .unwrap_or_else(|_| Config::default())
    })
}

pub fn init_config(path: Option<&str>) -> Result<(), config::ConfigError> {
    let config = match path {
        Some(p) => Config::load(p),
        None => Config::load("app-config.toml")
            .or_else(|_| Config::load("config.toml"))
            .or_else(|_| Ok(Config::default())),
    }?;
    
    CONFIG.set(config).map_err(|_| {
        config::ConfigError::NotFound("Configuration already initialized".to_string())
    })?;
    
    Ok(())
} 