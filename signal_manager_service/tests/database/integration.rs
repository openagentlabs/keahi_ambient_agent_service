use signal_manager_service::{
    database::{RegisteredClient, RegistrationPayload, ClientRepository, MockClientRepository},
    type_two_handlers::register::RegisterHandler,
    message::{Message, MessageType, Payload},
    config::Config,
};
use serde_json::json;
use std::sync::Arc;

/// Test configuration for integration tests
fn create_integration_test_config() -> Config {
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
            ],
        },
        firestore: signal_manager_service::config::FirestoreConfig {
            project_id: "test-project".to_string(),
            collection_name: "test_clients".to_string(),
        },
    }
}

#[tokio::test]
async fn test_register_handler_with_mock_repository() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test valid registration payload
    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket", "signaling"],
        "metadata": {
            "version": "1.0",
            "platform": "test"
        }
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::RegisterAck(ack) => {
            assert_eq!(ack.status, 200);
            assert_eq!(ack.client_id, "test_client");
            assert!(ack.session_id.is_some());
        }
        _ => panic!("Expected RegisterAck payload"),
    }
}

#[tokio::test]
async fn test_register_handler_duplicate_client() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload.clone()),
    );

    // First registration should succeed
    let response1 = handler.handle_register(message).await;
    assert!(response1.is_ok());

    // Second registration with same client_id should fail
    let message2 = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response2 = handler.handle_register(message2).await;
    assert!(response2.is_ok());

    let response_message = response2.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("already exists"));
        }
        _ => panic!("Expected Error payload for duplicate client"),
    }
}

#[tokio::test]
async fn test_register_handler_invalid_version() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with newer version
    let payload = json!({
        "version": "2.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("version"));
        }
        _ => panic!("Expected Error payload for invalid version"),
    }
}

#[tokio::test]
async fn test_register_handler_missing_version() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test without version field
    let payload = json!({
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("version"));
        }
        _ => panic!("Expected Error payload for missing version"),
    }
}

#[tokio::test]
async fn test_register_handler_missing_required_fields() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test missing client_id
    let payload = json!({
        "version": "1.0",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("client_id"));
        }
        _ => panic!("Expected Error payload for missing client_id"),
    }
}

#[tokio::test]
async fn test_register_handler_invalid_json() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with invalid JSON structure
    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": "invalid", // Should be array
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("validation"));
        }
        _ => panic!("Expected Error payload for invalid JSON"),
    }
}

#[tokio::test]
async fn test_register_handler_with_optional_fields() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with minimal required fields
    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token"
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::RegisterAck(ack) => {
            assert_eq!(ack.status, 200);
            assert_eq!(ack.client_id, "test_client");
            assert!(ack.session_id.is_some());
        }
        _ => panic!("Expected RegisterAck payload"),
    }
}

#[tokio::test]
async fn test_register_handler_repository_integration() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket", "signaling"],
        "metadata": {
            "version": "1.0",
            "platform": "test",
            "features": ["realtime", "p2p"]
        }
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    // Verify the client was actually stored in the repository
    let client = repository.get_client("test_client").await.unwrap();
    assert!(client.is_some());

    let stored_client = client.unwrap();
    assert_eq!(stored_client.client_id, "test_client");
    assert_eq!(stored_client.auth_token, "test_token");
    assert_eq!(stored_client.capabilities, vec!["websocket", "signaling"]);
    assert_eq!(stored_client.metadata, json!({
        "version": "1.0",
        "platform": "test",
        "features": ["realtime", "p2p"]
    }));
    assert!(stored_client.is_active());
}

#[tokio::test]
async fn test_register_handler_error_handling() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with empty client_id
    let payload = json!({
        "version": "1.0",
        "client_id": "",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("client_id"));
        }
        _ => panic!("Expected Error payload for empty client_id"),
    }
}

#[tokio::test]
async fn test_register_handler_session_id_generation() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let message = Message::new(
        MessageType::Register,
        Payload::Register(payload),
    );

    let response = handler.handle_register(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::RegisterAck(ack) => {
            assert_eq!(ack.status, 200);
            assert_eq!(ack.client_id, "test_client");
            assert!(ack.session_id.is_some());
            
            let session_id = ack.session_id.as_ref().unwrap();
            assert!(!session_id.is_empty());
            assert!(session_id.len() > 10); // Should be a reasonable UUID length
        }
        _ => panic!("Expected RegisterAck payload"),
    }
}

#[tokio::test]
async fn test_register_handler_multiple_clients() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Register multiple clients
    for i in 1..=3 {
        let payload = json!({
            "version": "1.0",
            "client_id": format!("client_{}", i),
            "auth_token": format!("token_{}", i),
            "capabilities": ["websocket"],
            "metadata": {"version": "1.0", "index": i}
        });

        let message = Message::new(
            MessageType::Register,
            Payload::Register(payload),
        );

        let response = handler.handle_register(message).await;
        assert!(response.is_ok());

        let response_message = response.unwrap();
        match &response_message.payload {
            Payload::RegisterAck(ack) => {
                assert_eq!(ack.status, 200);
                assert_eq!(ack.client_id, format!("client_{}", i));
                assert!(ack.session_id.is_some());
            }
            _ => panic!("Expected RegisterAck payload"),
        }
    }

    // Verify all clients were stored
    let clients = repository.list_clients(None).await.unwrap();
    assert_eq!(clients.len(), 3);

    for (i, client) in clients.iter().enumerate() {
        let expected_id = format!("client_{}", i + 1);
        assert_eq!(client.client_id, expected_id);
        assert_eq!(client.auth_token, format!("token_{}", i + 1));
    }
} 

#[tokio::test]
async fn test_unregister_handler_with_mock_repository() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // First register a client
    let register_payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket", "signaling"],
        "metadata": {
            "version": "1.0",
            "platform": "test"
        }
    });

    let register_message = Message::new(
        MessageType::Register,
        Payload::Register(register_payload),
    );

    let register_response = handler.handle_register(register_message).await;
    assert!(register_response.is_ok());

    // Now test unregister
    let unregister_payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token"
    });

    let unregister_message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(unregister_payload),
    );

    let response = handler.handle_unregister(unregister_message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::UnregisterAck(ack) => {
            assert_eq!(ack.status, 200);
            assert_eq!(ack.client_id, "test_client");
        }
        _ => panic!("Expected UnregisterAck payload"),
    }
}

#[tokio::test]
async fn test_unregister_handler_invalid_version() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with newer version
    let payload = json!({
        "version": "2.0",
        "client_id": "test_client",
        "auth_token": "test_token"
    });

    let message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(payload),
    );

    let response = handler.handle_unregister(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("version"));
        }
        _ => panic!("Expected Error payload for invalid version"),
    }
}

#[tokio::test]
async fn test_unregister_handler_missing_version() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test without version field
    let payload = json!({
        "client_id": "test_client",
        "auth_token": "test_token"
    });

    let message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(payload),
    );

    let response = handler.handle_unregister(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("version"));
        }
        _ => panic!("Expected Error payload for missing version"),
    }
}

#[tokio::test]
async fn test_unregister_handler_missing_client_id() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test missing client_id
    let payload = json!({
        "version": "1.0",
        "auth_token": "test_token"
    });

    let message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(payload),
    );

    let response = handler.handle_unregister(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("client_id"));
        }
        _ => panic!("Expected Error payload for missing client_id"),
    }
}

#[tokio::test]
async fn test_unregister_handler_invalid_json() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with invalid JSON structure
    let payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "extra_field": "should_not_be_here"
    });

    let message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(payload),
    );

    let response = handler.handle_unregister(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("unknown field"));
        }
        _ => panic!("Expected Error payload for invalid JSON"),
    }
}

#[tokio::test]
async fn test_unregister_handler_nonexistent_client() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Try to unregister a client that was never registered
    let payload = json!({
        "version": "1.0",
        "client_id": "nonexistent_client",
        "auth_token": "test_token"
    });

    let message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(payload),
    );

    let response = handler.handle_unregister(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 404);
            assert!(error.error_message.contains("not found"));
        }
        _ => panic!("Expected Error payload for nonexistent client"),
    }
}

#[tokio::test]
async fn test_unregister_handler_repository_integration() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Register a client first
    let register_payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token",
        "capabilities": ["websocket"],
        "metadata": {"version": "1.0"}
    });

    let register_message = Message::new(
        MessageType::Register,
        Payload::Register(register_payload),
    );

    let register_response = handler.handle_register(register_message).await;
    assert!(register_response.is_ok());

    // Verify client exists in repository
    let client = repository.get_client("test_client").await.unwrap();
    assert_eq!(client.client_id, "test_client");

    // Now unregister
    let unregister_payload = json!({
        "version": "1.0",
        "client_id": "test_client",
        "auth_token": "test_token"
    });

    let unregister_message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(unregister_payload),
    );

    let response = handler.handle_unregister(unregister_message).await;
    assert!(response.is_ok());

    // Verify client was removed from repository
    let client_result = repository.get_client("test_client").await;
    assert!(client_result.is_err());
}

#[tokio::test]
async fn test_unregister_handler_multiple_clients() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Register multiple clients
    let clients = vec!["client1", "client2", "client3"];
    
    for client_id in &clients {
        let register_payload = json!({
            "version": "1.0",
            "client_id": client_id,
            "auth_token": "test_token",
            "capabilities": ["websocket"],
            "metadata": {"version": "1.0"}
        });

        let register_message = Message::new(
            MessageType::Register,
            Payload::Register(register_payload),
        );

        let register_response = handler.handle_register(register_message).await;
        assert!(register_response.is_ok());
    }

    // Unregister one client
    let unregister_payload = json!({
        "version": "1.0",
        "client_id": "client2",
        "auth_token": "test_token"
    });

    let unregister_message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(unregister_payload),
    );

    let response = handler.handle_unregister(unregister_message).await;
    assert!(response.is_ok());

    // Verify only client2 was removed
    assert!(repository.get_client("client1").await.is_ok());
    assert!(repository.get_client("client2").await.is_err());
    assert!(repository.get_client("client3").await.is_ok());
}

#[tokio::test]
async fn test_unregister_handler_error_handling() {
    let config = Arc::new(create_integration_test_config());
    let repository = Arc::new(MockClientRepository::new());
    let handler = RegisterHandler::new(config, repository);

    // Test with malformed JSON
    let payload = json!({
        "version": "1.0",
        "client_id": null,  // Invalid client_id
        "auth_token": "test_token"
    });

    let message = Message::new(
        MessageType::Unregister,
        Payload::Unregister(payload),
    );

    let response = handler.handle_unregister(message).await;
    assert!(response.is_ok());

    let response_message = response.unwrap();
    match &response_message.payload {
        Payload::Error(error) => {
            assert_eq!(error.error_code, 400);
            assert!(error.error_message.contains("client_id"));
        }
        _ => panic!("Expected Error payload for invalid client_id"),
    }
} 