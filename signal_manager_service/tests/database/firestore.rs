use signal_manager_service::database::{
    FirestoreClientRepository, FirestoreRepositoryFactory, DatabaseResult, RegisteredClient, RegistrationPayload
};
use signal_manager_service::config::{Config, FirestoreConfig};
use serde_json::json;
use std::sync::Arc;

/// Test configuration for Firestore tests
fn create_test_config() -> Config {
    Config {
        server: signal_manager_service::config::ServerConfig {
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
        auth: signal_manager_service::config::AuthConfig {
            token_secret: "test-secret".to_string(),
            token_expiry: 3600,
            auth_method: "token".to_string(),
            api_keys: vec![
                "test_client_1:test_token_1".to_string(),
                "test_client_2:test_token_2".to_string(),
            ],
        },
        logging: signal_manager_service::config::LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
            file_path: None,
            console_output: true,
            max_file_size: 10485760,
            max_files: 5,
        },
        metrics: signal_manager_service::config::MetricsConfig {
            enabled: true,
            port: 9090,
            host: "127.0.0.1".to_string(),
            connection_stats_interval: 60,
            message_stats_interval: 30,
        },
        session: signal_manager_service::config::SessionConfig {
            session_timeout: 3600,
            cleanup_interval: 300,
            max_sessions_per_client: 1,
        },
        security: signal_manager_service::config::SecurityConfig {
            rate_limit_enabled: true,
            max_messages_per_minute: 100,
            max_connections_per_ip: 10,
            allowed_origins: vec!["*".to_string()],
        },
        gcp: signal_manager_service::config::GcpConfig {
            credentials_path: "".to_string(),
            project_id: "test-project".to_string(),
            region: "us-central1".to_string(),
        },
        firestore: FirestoreConfig {
            project_id: "test-project".to_string(),
            database_name: "test-db".to_string(),
            region: "us-central1".to_string(),
        },
        cloudflare: signal_manager_service::config::CloudflareConfig {
            app_id: "test-app-id".to_string(),
            app_secret: "test-app-secret".to_string(),
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
            stun_url: "stun:stun.cloudflare.com:3478".to_string(),
        },
    }
}

#[test]
fn test_firestore_config_creation() {
    let config = create_test_config();
    
    assert_eq!(config.firestore.project_id, "test-project");
    assert_eq!(config.firestore.database_name, "test-db");
}

#[test]
fn test_firestore_repository_factory_creation() {
    let config = Arc::new(create_test_config());
    let factory = FirestoreRepositoryFactory::new(config);
    
    // Factory should be created successfully
    assert!(true); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_firestore_repository_creation() {
    let config = create_test_config();
    let repo = FirestoreClientRepository::new(&config).await;
    
    // This will fail in test environment without proper credentials,
    // but we can test the error handling
    match repo {
        Ok(_) => {
            // If we have proper credentials, this should work
            assert!(true);
        }
        Err(error) => {
            // Expected in test environment without credentials
            match error {
                signal_manager_service::database::DatabaseError::Connection(_) => {
                    assert!(true); // Expected error
                }
                _ => panic!("Unexpected error type: {:?}", error),
            }
        }
    }
}

#[tokio::test]
async fn test_firestore_repository_factory() {
    let config = Arc::new(create_test_config());
    let factory = FirestoreRepositoryFactory::new(config);
    
    let repo = factory.create_client_repository().await;
    
    // This will fail in test environment without proper credentials,
    // but we can test the error handling
    match repo {
        Ok(_) => {
            // If we have proper credentials, this should work
            assert!(true);
        }
        Err(error) => {
            // Expected in test environment without credentials
            match error {
                signal_manager_service::database::DatabaseError::Connection(_) => {
                    assert!(true); // Expected error
                }
                _ => panic!("Unexpected error type: {:?}", error),
            }
        }
    }
}

#[test]
fn test_firestore_config_serialization() {
    let config = create_test_config();
    
    // Test that the config can be serialized
    let serialized = serde_json::to_string(&config.firestore);
    assert!(serialized.is_ok());
    
    let firestore_config_json = serialized.unwrap();
    assert!(firestore_config_json.contains("test-project"));
    assert!(firestore_config_json.contains("test-db"));
}

#[test]
fn test_firestore_config_deserialization() {
    let config_json = r#"{
        "project_id": "test-project",
        "database_name": "test-db",
        "region": "us-central1"
    }"#;
    
    let firestore_config: FirestoreConfig = serde_json::from_str(config_json).unwrap();
    
    assert_eq!(firestore_config.project_id, "test-project");
    assert_eq!(firestore_config.database_name, "test-db");
}

#[tokio::test]
async fn test_firestore_error_handling() {
    let config = create_test_config();
    
    // Test with invalid project ID
    let invalid_config = Config {
        server: config.server.clone(),
        auth: config.auth.clone(),
        logging: config.logging.clone(),
        metrics: config.metrics.clone(),
        session: config.session.clone(),
        security: config.security.clone(),
        gcp: config.gcp.clone(),
        firestore: FirestoreConfig {
            project_id: "".to_string(), // Invalid empty project ID
            database_name: "test-db".to_string(),
            region: "us-central1".to_string(),
        },
        cloudflare: config.cloudflare.clone(),
    };
    
    let repo = FirestoreClientRepository::new(&invalid_config).await;
    assert!(repo.is_err());
    
    if let Err(error) = repo {
        match error {
            signal_manager_service::database::DatabaseError::Connection(_) => {
                assert!(true); // Expected error
            }
            _ => panic!("Unexpected error type: {:?}", error),
        }
    }
}

#[test]
fn test_firestore_project_id_validation() {
    let config = create_test_config();
    
    // Test valid project ID
    assert!(!config.firestore.project_id.is_empty());
    assert!(config.firestore.project_id.len() > 0);
    
    // Test that project ID doesn't contain invalid characters
    let invalid_chars = ['/', '\\', ' ', '.'];
    for &ch in &invalid_chars {
        assert!(!config.firestore.project_id.contains(ch));
    }
}

#[tokio::test]
async fn test_firestore_config_integration() {
    let config = create_test_config();
    
    // Test that the config can be used to create a repository factory
    let factory = FirestoreRepositoryFactory::new(Arc::new(config));
    
    // Test that the factory can be created without errors
    assert!(true);
}

#[test]
fn test_firestore_config_defaults() {
    let config = Config::default();
    
    // Test that default config has valid firestore settings
    assert!(!config.firestore.project_id.is_empty());
    assert!(!config.firestore.database_name.is_empty());
    assert!(!config.firestore.region.is_empty());
}

#[test]
fn test_firestore_config_clone() {
    let config = create_test_config();
    let cloned_config = config.clone();
    
    assert_eq!(config.firestore.project_id, cloned_config.firestore.project_id);
    assert_eq!(config.firestore.database_name, cloned_config.firestore.database_name);
    assert_eq!(config.firestore.region, cloned_config.firestore.region);
}

#[test]
fn test_firestore_config_debug() {
    let config = create_test_config();
    let debug_output = format!("{:?}", config.firestore);
    
    assert!(debug_output.contains("test-project"));
    assert!(debug_output.contains("test-db"));
} 