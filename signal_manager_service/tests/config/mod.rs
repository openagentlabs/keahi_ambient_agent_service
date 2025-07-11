use signal_manager_service::config::{Config, init_config, get_config};

#[test]
fn test_config_default() {
    let config = Config::default();
    
    // Test server config
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.max_connections, 1000);
    assert_eq!(config.server.heartbeat_interval, 30);
    assert_eq!(config.server.tls_enabled, false);
    assert_eq!(config.server.read_buffer_size, 8192);
    assert_eq!(config.server.write_buffer_size, 8192);
    assert_eq!(config.server.max_message_size, 1048576);
    

    
    // Test auth config
    assert_eq!(config.auth.auth_method, "token");
    assert_eq!(config.auth.token_expiry, 3600);
    assert_eq!(config.auth.api_keys.len(), 2);
    
    // Test logging config
    assert_eq!(config.logging.level, "info");
    assert_eq!(config.logging.format, "json");
    assert_eq!(config.logging.console_output, true);
    assert_eq!(config.logging.max_file_size, 10485760);
    assert_eq!(config.logging.max_files, 5);
    
    // Test metrics config
    assert_eq!(config.metrics.enabled, true);
    assert_eq!(config.metrics.port, 9090);
    assert_eq!(config.metrics.host, "127.0.0.1");
    assert_eq!(config.metrics.connection_stats_interval, 60);
    assert_eq!(config.metrics.message_stats_interval, 30);
    
    // Test session config
    assert_eq!(config.session.session_timeout, 3600);
    assert_eq!(config.session.cleanup_interval, 300);
    assert_eq!(config.session.max_sessions_per_client, 1);
    
    // Test security config
    assert_eq!(config.security.rate_limit_enabled, true);
    assert_eq!(config.security.max_messages_per_minute, 1000);
    assert_eq!(config.security.max_connections_per_ip, 10);
    assert_eq!(config.security.allowed_origins.len(), 1);
    assert_eq!(config.security.allowed_origins[0], "*");
}

#[test]
fn test_config_load_from_file() {
    // Test loading from app-config.toml
    let result = Config::load("app-config.toml");
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.auth.auth_method, "token");
}

#[test]
fn test_config_socket_addr() {
    let config = Config::default();
    let addr = config.socket_addr();
    assert_eq!(addr.to_string(), "127.0.0.1:8080");
}

#[test]
fn test_config_metrics_addr() {
    let config = Config::default();
    let addr = config.metrics_addr();
    assert_eq!(addr.to_string(), "127.0.0.1:9090");
}

#[test]
fn test_config_parse_api_keys() {
    let config = Config::default();
    let keys = config.parse_api_keys();
    
    assert_eq!(keys.len(), 2);
    assert_eq!(keys.get("test_client_1"), Some(&"test_token_1".to_string()));
    assert_eq!(keys.get("test_client_2"), Some(&"test_token_2".to_string()));
}

#[test]
fn test_global_config_access() {
    // Initialize config
    init_config(None).unwrap();
    
    // Get config from global accessor
    let config = get_config();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
}

#[test]
fn test_config_with_custom_path() {
    // Test with a custom config path
    let result = init_config(Some("app-config.toml"));
    assert!(result.is_ok());
    
    let config = get_config();
    assert_eq!(config.server.host, "127.0.0.1");
}

#[test]
fn test_tls_config() {
    let mut config = Config::default();
    
    // Test TLS disabled
    config.server.tls_enabled = false;
    assert_eq!(config.server.tls_enabled, false);
    
    // Test TLS enabled
    config.server.tls_enabled = true;
    config.server.tls_cert_path = "/path/to/cert.pem".to_string();
    config.server.tls_key_path = "/path/to/key.pem".to_string();
    assert_eq!(config.server.tls_enabled, true);
    assert_eq!(config.server.tls_cert_path, "/path/to/cert.pem");
    assert_eq!(config.server.tls_key_path, "/path/to/key.pem");
}

#[test]
fn test_auth_methods() {
    let mut config = Config::default();
    
    // Test different auth methods
    config.auth.auth_method = "token".to_string();
    assert_eq!(config.auth.auth_method, "token");
    
    config.auth.auth_method = "api_key".to_string();
    assert_eq!(config.auth.auth_method, "api_key");
    

}

#[test]
fn test_logging_config() {
    let mut config = Config::default();
    
    // Test file logging configuration
    config.logging.file_path = Some("/var/log/signal-manager.log".to_string());
    config.logging.console_output = false;
    
    assert_eq!(config.logging.file_path, Some("/var/log/signal-manager.log".to_string()));
    assert_eq!(config.logging.console_output, false);
}

#[test]
fn test_security_config() {
    let mut config = Config::default();
    
    // Test rate limiting
    config.security.rate_limit_enabled = true;
    config.security.max_messages_per_minute = 500;
    config.security.max_connections_per_ip = 5;
    
    assert_eq!(config.security.rate_limit_enabled, true);
    assert_eq!(config.security.max_messages_per_minute, 500);
    assert_eq!(config.security.max_connections_per_ip, 5);
    
    // Test CORS origins
    config.security.allowed_origins = vec![
        "https://example.com".to_string(),
        "https://app.example.com".to_string(),
    ];
    
    assert_eq!(config.security.allowed_origins.len(), 2);
    assert_eq!(config.security.allowed_origins[0], "https://example.com");
    assert_eq!(config.security.allowed_origins[1], "https://app.example.com");
}

#[test]
fn test_session_config() {
    let mut config = Config::default();
    
    // Test session timeout
    config.session.session_timeout = 7200; // 2 hours
    config.session.cleanup_interval = 600; // 10 minutes
    config.session.max_sessions_per_client = 3;
    
    assert_eq!(config.session.session_timeout, 7200);
    assert_eq!(config.session.cleanup_interval, 600);
    assert_eq!(config.session.max_sessions_per_client, 3);
}

#[test]
fn test_metrics_config() {
    let mut config = Config::default();
    
    // Test metrics configuration
    config.metrics.enabled = false;
    config.metrics.port = 9091;
    config.metrics.host = "0.0.0.0".to_string();
    config.metrics.connection_stats_interval = 120;
    config.metrics.message_stats_interval = 60;
    
    assert_eq!(config.metrics.enabled, false);
    assert_eq!(config.metrics.port, 9091);
    assert_eq!(config.metrics.host, "0.0.0.0");
    assert_eq!(config.metrics.connection_stats_interval, 120);
    assert_eq!(config.metrics.message_stats_interval, 60);
} 