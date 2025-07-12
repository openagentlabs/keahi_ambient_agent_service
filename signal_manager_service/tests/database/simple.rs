use signal_manager_service::database::{
    RegisteredClient, ClientStatus, RegistrationPayload, RegistrationResponse
};
use serde_json::json;
use chrono::Utc;

#[test]
fn test_basic_client_creation() {
    let client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );

    assert_eq!(client.client_id, "test_client");
    assert_eq!(client.auth_token, "test_token");
    assert_eq!(client.capabilities, vec!["websocket"]);
    assert_eq!(client.metadata, json!({"version": "1.0"}));
    assert_eq!(client.status, ClientStatus::Active);
    assert!(client.last_seen.is_none());
    assert!(!client.id.is_empty());
}

#[test]
fn test_client_status() {
    let mut client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec![],
        json!({}),
    );

    // Default status should be Active
    assert!(client.is_active());

    // Test different statuses
    client.status = ClientStatus::Inactive;
    assert!(!client.is_active());

    client.status = ClientStatus::Suspended;
    assert!(!client.is_active());

    client.status = ClientStatus::Pending;
    assert!(!client.is_active());

    client.status = ClientStatus::Active;
    assert!(client.is_active());
}

#[test]
fn test_client_update_last_seen() {
    let mut client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec![],
        json!({}),
    );

    assert!(client.last_seen.is_none());
    
    let before_update = Utc::now();
    client.update_last_seen();
    let after_update = Utc::now();

    assert!(client.last_seen.is_some());
    let last_seen = client.last_seen.unwrap();
    assert!(last_seen >= before_update && last_seen <= after_update);
}

#[test]
fn test_registration_payload() {
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        room_id: None,
        capabilities: Some(vec!["websocket".to_string()]),
        metadata: Some(json!({"version": "1.0"})),
    };

    assert_eq!(payload.client_id, "test_client");
    assert_eq!(payload.auth_token, "test_token");
    assert_eq!(payload.room_id, None);
    assert_eq!(payload.capabilities, Some(vec!["websocket".to_string()]));
    assert_eq!(payload.metadata, Some(json!({"version": "1.0"})));
}

#[test]
fn test_registration_response() {
    let response = RegistrationResponse {
        status: 200,
        message: Some("Success".to_string()),
        client_id: "test_client".to_string(),
        session_id: Some("session_123".to_string()),
    };

    assert_eq!(response.status, 200);
    assert_eq!(response.message, Some("Success".to_string()));
    assert_eq!(response.client_id, "test_client");
    assert_eq!(response.session_id, Some("session_123".to_string()));
}

#[test]
fn test_client_serialization() {
    let client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );

    // Test serialization
    let serialized = serde_json::to_string(&client);
    assert!(serialized.is_ok());

    // Test deserialization
    let deserialized: RegisteredClient = serde_json::from_str(&serialized.unwrap()).unwrap();
    assert_eq!(deserialized.client_id, client.client_id);
    assert_eq!(deserialized.auth_token, client.auth_token);
    assert_eq!(deserialized.capabilities, client.capabilities);
    assert_eq!(deserialized.metadata, client.metadata);
    assert_eq!(deserialized.status, client.status);
}

#[test]
fn test_client_status_serialization() {
    let statuses = vec![
        ClientStatus::Active,
        ClientStatus::Inactive,
        ClientStatus::Suspended,
        ClientStatus::Pending,
    ];

    for status in statuses {
        let serialized = serde_json::to_string(&status);
        assert!(serialized.is_ok());

        let deserialized: ClientStatus = serde_json::from_str(&serialized.unwrap()).unwrap();
        assert_eq!(deserialized, status);
    }
}

#[test]
fn test_client_id_uniqueness() {
    let client1 = RegisteredClient::new(
        "client1".to_string(),
        "token1".to_string(),
        vec![],
        json!({}),
    );

    let client2 = RegisteredClient::new(
        "client2".to_string(),
        "token2".to_string(),
        vec![],
        json!({}),
    );

    // UUIDs should be unique
    assert_ne!(client1.id, client2.id);
    assert_ne!(client1.id, "");
    assert_ne!(client2.id, "");
}

#[test]
fn test_client_timestamp_ordering() {
    let client1 = RegisteredClient::new(
        "client1".to_string(),
        "token1".to_string(),
        vec![],
        json!({}),
    );

    std::thread::sleep(std::time::Duration::from_millis(10));

    let client2 = RegisteredClient::new(
        "client2".to_string(),
        "token2".to_string(),
        vec![],
        json!({}),
    );

    // Timestamps should be in chronological order
    assert!(client1.registered_at < client2.registered_at);
} 