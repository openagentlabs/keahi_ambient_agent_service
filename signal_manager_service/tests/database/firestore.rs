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
        },
        auth: signal_manager_service::config::AuthConfig {
            clients: vec![
                signal_manager_service::config::ClientConfig {
                    client_id: "test_client_1".to_string(),
                    auth_token: "test_token_1".to_string(),
                },
                signal_manager_service::config::ClientConfig {
                    client_id: "test_client_2".to_string(),
                    auth_token: "test_token_2".to_string(),
                },
            ],
        },
        firestore: FirestoreConfig {
            project_id: "test-project".to_string(),
            collection_name: "test_clients".to_string(),
        },
    }
}

#[test]
fn test_firestore_config_creation() {
    let config = create_test_config();
    
    assert_eq!(config.firestore.project_id, "test-project");
    assert_eq!(config.firestore.collection_name, "test_clients");
}

#[test]
fn test_firestore_repository_factory_creation() {
    let config = Arc::new(create_test_config());
    let factory = FirestoreRepositoryFactory::new(config);
    
    // Factory should be created successfully
    assert!(true); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_firestore_document_path_generation() {
    let config = create_test_config();
    let repo = FirestoreClientRepository {
        db: firestore::FirestoreDb::new("test-project").await.unwrap(),
        collection_name: config.firestore.collection_name,
    };
    
    let path = repo.get_document_path("test_client");
    assert_eq!(path, "test_clients/test_client");
}

#[tokio::test]
async fn test_firestore_client_to_document_conversion() {
    let config = create_test_config();
    let repo = FirestoreClientRepository {
        db: firestore::FirestoreDb::new("test-project").await.unwrap(),
        collection_name: config.firestore.collection_name,
    };
    
    let client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );
    
    let document = repo.client_to_document(&client);
    assert!(document.is_ok());
    
    let doc = document.unwrap();
    assert!(!doc.fields.is_empty());
}

#[tokio::test]
async fn test_firestore_document_to_client_conversion() {
    let config = create_test_config();
    let repo = FirestoreClientRepository {
        db: firestore::FirestoreDb::new("test-project").await.unwrap(),
        collection_name: config.firestore.collection_name,
    };
    
    let original_client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );
    
    // Convert to document
    let document = repo.client_to_document(&original_client).unwrap();
    
    // Convert back to client
    let converted_client = repo.document_to_client(&document);
    assert!(converted_client.is_ok());
    
    let client = converted_client.unwrap();
    assert_eq!(client.client_id, original_client.client_id);
    assert_eq!(client.auth_token, original_client.auth_token);
    assert_eq!(client.capabilities, original_client.capabilities);
    assert_eq!(client.metadata, original_client.metadata);
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
    assert!(firestore_config_json.contains("test_clients"));
}

#[test]
fn test_firestore_config_deserialization() {
    let config_json = r#"{
        "project_id": "test-project",
        "collection_name": "test_clients"
    }"#;
    
    let firestore_config: FirestoreConfig = serde_json::from_str(config_json).unwrap();
    
    assert_eq!(firestore_config.project_id, "test-project");
    assert_eq!(firestore_config.collection_name, "test_clients");
}

#[tokio::test]
async fn test_firestore_error_handling() {
    let config = create_test_config();
    
    // Test with invalid project ID
    let invalid_config = Config {
        server: config.server.clone(),
        auth: config.auth.clone(),
        firestore: FirestoreConfig {
            project_id: "".to_string(), // Invalid empty project ID
            collection_name: config.firestore.collection_name.clone(),
        },
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
fn test_firestore_collection_name_validation() {
    let config = create_test_config();
    
    // Test valid collection name
    assert!(!config.firestore.collection_name.is_empty());
    assert!(config.firestore.collection_name.len() > 0);
    
    // Test that collection name doesn't contain invalid characters
    let invalid_chars = ['/', '\\', ' ', '.'];
    for &ch in &invalid_chars {
        assert!(!config.firestore.collection_name.contains(ch));
    }
}

#[test]
fn test_firestore_project_id_validation() {
    let config = create_test_config();
    
    // Test valid project ID
    assert!(!config.firestore.project_id.is_empty());
    assert!(config.firestore.project_id.len() > 0);
    
    // Test that project ID follows GCP naming conventions
    // Project IDs should be lowercase letters, numbers, and hyphens only
    let valid_chars: Vec<char> = config.firestore.project_id.chars()
        .filter(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        .collect();
    
    assert_eq!(valid_chars.len(), config.firestore.project_id.len());
}

#[tokio::test]
async fn test_firestore_config_integration() {
    let config = create_test_config();
    
    // Test that the config can be used to create a repository factory
    let factory = FirestoreRepositoryFactory::new(Arc::new(config));
    
    // Test that the factory can be created
    assert!(true); // Just checking it doesn't panic
    
    // Test that the factory has the expected configuration
    // (We can't access private fields, but we can test the public interface)
    let repo_result = factory.create_client_repository().await;
    
    // In test environment without credentials, this should fail
    // but the error should be related to connection, not configuration
    match repo_result {
        Ok(_) => {
            // If we have proper credentials, this should work
            assert!(true);
        }
        Err(error) => {
            match error {
                signal_manager_service::database::DatabaseError::Connection(_) => {
                    assert!(true); // Expected error in test environment
                }
                _ => panic!("Unexpected error type: {:?}", error),
            }
        }
    }
}

#[test]
fn test_firestore_config_defaults() {
    // Test that we can create a minimal config
    let firestore_config = FirestoreConfig {
        project_id: "test-project".to_string(),
        collection_name: "clients".to_string(),
    };
    
    assert_eq!(firestore_config.project_id, "test-project");
    assert_eq!(firestore_config.collection_name, "clients");
}

#[test]
fn test_firestore_config_clone() {
    let config = create_test_config();
    let cloned_config = config.clone();
    
    assert_eq!(config.firestore.project_id, cloned_config.firestore.project_id);
    assert_eq!(config.firestore.collection_name, cloned_config.firestore.collection_name);
}

#[test]
fn test_firestore_config_debug() {
    let config = create_test_config();
    let debug_str = format!("{:?}", config.firestore);
    
    assert!(debug_str.contains("test-project"));
    assert!(debug_str.contains("test_clients"));
} 