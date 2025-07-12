use signal_manager_service::database::{RegisteredClient, ClientStatus, TerminatedRoom, TerminationPayload, RoomCreated, RoomCreationPayload};
use serde_json::json;
use chrono::{DateTime, Utc};

#[test]
fn test_registered_client_creation() {
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
    assert_eq!(client.room_id, None);
}

#[test]
fn test_registered_client_with_room() {
    let client = RegisteredClient::new_with_room(
        "test_client".to_string(),
        "test_token".to_string(),
        "room_123".to_string(),
        vec!["websocket".to_string(), "video".to_string()],
        json!({"version": "1.0", "platform": "web"}),
    );

    assert_eq!(client.client_id, "test_client");
    assert_eq!(client.auth_token, "test_token");
    assert_eq!(client.room_id, Some("room_123".to_string()));
    assert_eq!(client.capabilities, vec!["websocket", "video"]);
    assert_eq!(client.metadata, json!({"version": "1.0", "platform": "web"}));
    assert_eq!(client.status, ClientStatus::Active);
    assert!(client.last_seen.is_none());
}

#[test]
fn test_registered_client_room_association() {
    let mut client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );

    // Initially no room association
    assert_eq!(client.get_room_id(), None);

    // Associate with a room
    client.associate_with_room("room_456".to_string());
    assert_eq!(client.get_room_id(), Some("room_456"));

    // Disassociate from room
    client.disassociate_from_room();
    assert_eq!(client.get_room_id(), None);
}

#[test]
fn test_registered_client_update_last_seen() {
    let mut client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );

    assert!(client.last_seen.is_none());
    client.update_last_seen();
    assert!(client.last_seen.is_some());
}

#[test]
fn test_registered_client_is_active() {
    let client = RegisteredClient::new(
        "test_client".to_string(),
        "test_token".to_string(),
        vec!["websocket".to_string()],
        json!({"version": "1.0"}),
    );

    assert!(client.is_active());
}

#[test]
fn test_terminated_room_creation() {
    let room_data = json!({
        "room_name": "test_room",
        "participants": ["user1", "user2"],
        "settings": {"max_participants": 10}
    });

    let terminated_room = TerminatedRoom::new(
        "room_123".to_string(),
        room_data.clone(),
        Some("Room was inactive for too long".to_string()),
        Some("admin_user".to_string()),
        Some(json!({"termination_type": "automatic"})),
    );

    assert_eq!(terminated_room.room_id, "room_123");
    assert_eq!(terminated_room.room_data, room_data);
    assert_eq!(terminated_room.termination_reason, Some("Room was inactive for too long".to_string()));
    assert_eq!(terminated_room.terminated_by, Some("admin_user".to_string()));
    assert_eq!(terminated_room.metadata, json!({"termination_type": "automatic"}));
    assert!(terminated_room.terminated_at <= Utc::now());
    assert!(terminated_room.termination_recorded_at <= Utc::now());
    assert!(terminated_room.termination_recorded_at >= terminated_room.terminated_at);
}

#[test]
fn test_terminated_room_getters() {
    let room_data = json!({"room_name": "test_room"});
    let terminated_room = TerminatedRoom::new(
        "room_123".to_string(),
        room_data.clone(),
        None,
        None,
        None,
    );

    assert_eq!(terminated_room.get_room_id(), "room_123");
    assert_eq!(terminated_room.get_room_data(), &room_data);
    assert!(terminated_room.get_terminated_at() <= Utc::now());
    assert!(terminated_room.get_termination_recorded_at() <= Utc::now());
}

#[test]
fn test_terminated_room_with_custom_timestamps() {
    let room_data = json!({"room_name": "test_room"});
    let custom_terminated_at = Utc::now() - chrono::Duration::hours(2);
    
    let terminated_room = TerminatedRoom::new_with_timestamps(
        "room_456".to_string(),
        room_data.clone(),
        custom_terminated_at,
        Some("Manual termination".to_string()),
        Some("user123".to_string()),
        Some(json!({"reason": "manual"})),
    );

    assert_eq!(terminated_room.room_id, "room_456");
    assert_eq!(terminated_room.terminated_at, custom_terminated_at);
    assert!(terminated_room.termination_recorded_at > custom_terminated_at);
    assert!(terminated_room.termination_recorded_at <= Utc::now());
    assert_eq!(terminated_room.termination_reason, Some("Manual termination".to_string()));
    assert_eq!(terminated_room.terminated_by, Some("user123".to_string()));
}

#[test]
fn test_terminated_room_with_optional_fields() {
    let terminated_room = TerminatedRoom::new(
        "room_456".to_string(),
        json!({"room_name": "test_room"}),
        None,
        None,
        None,
    );

    assert_eq!(terminated_room.room_id, "room_456");
    assert_eq!(terminated_room.termination_reason, None);
    assert_eq!(terminated_room.terminated_by, None);
    assert_eq!(terminated_room.metadata, json!({}));
    assert!(terminated_room.terminated_at <= Utc::now());
    assert!(terminated_room.termination_recorded_at <= Utc::now());
}

#[test]
fn test_termination_payload_creation() {
    let payload = TerminationPayload {
        room_id: "room_789".to_string(),
        room_data: json!({"room_name": "test_room"}),
        termination_reason: Some("Manual termination".to_string()),
        terminated_by: Some("user123".to_string()),
        metadata: Some(json!({"reason": "manual"})),
    };

    assert_eq!(payload.room_id, "room_789");
    assert_eq!(payload.termination_reason, Some("Manual termination".to_string()));
    assert_eq!(payload.terminated_by, Some("user123".to_string()));
    assert_eq!(payload.metadata, Some(json!({"reason": "manual"})));
}

#[test]
fn test_room_created_creation() {
    let room_data = json!({
        "room_name": "test_room",
        "max_participants": 10,
        "settings": {"allow_screen_sharing": true}
    });

    let room_created = RoomCreated::new(
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data.clone(),
        Some("user123".to_string()),
        Some(json!({"creation_type": "manual"})),
    );

    assert_eq!(room_created.room_uuid, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(room_created.room_data, room_data);
    assert_eq!(room_created.created_by, Some("user123".to_string()));
    assert_eq!(room_created.metadata, json!({"creation_type": "manual"}));
    assert!(room_created.created_at <= Utc::now());
}

#[test]
fn test_room_created_getters() {
    let room_data = json!({"room_name": "test_room"});
    let room_created = RoomCreated::new(
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data.clone(),
        None,
        None,
    );

    assert_eq!(room_created.get_room_uuid(), "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(room_created.get_room_data(), &room_data);
    assert!(room_created.get_created_at() <= Utc::now());
}

#[test]
fn test_room_created_with_optional_fields() {
    let room_created = RoomCreated::new(
        "550e8400-e29b-41d4-a716-446655440001".to_string(),
        json!({"room_name": "test_room"}),
        None,
        None,
    );

    assert_eq!(room_created.room_uuid, "550e8400-e29b-41d4-a716-446655440001");
    assert_eq!(room_created.created_by, None);
    assert_eq!(room_created.metadata, json!({}));
}

#[test]
fn test_room_creation_payload_creation() {
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440002".to_string(),
        room_data: json!({"room_name": "test_room"}),
        created_by: Some("user456".to_string()),
        metadata: Some(json!({"reason": "manual_creation"})),
    };

    assert_eq!(payload.room_uuid, "550e8400-e29b-41d4-a716-446655440002");
    assert_eq!(payload.created_by, Some("user456".to_string()));
    assert_eq!(payload.metadata, Some(json!({"reason": "manual_creation"})));
}

#[test]
fn test_client_status_default() {
    let status: ClientStatus = Default::default();
    assert_eq!(status, ClientStatus::Active);
}

#[test]
fn test_registration_payload_with_room() {
    use signal_manager_service::database::RegistrationPayload;
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        room_id: Some("room_123".to_string()),
        capabilities: Some(vec!["websocket".to_string(), "video".to_string()]),
        metadata: Some(json!({"version": "1.0", "platform": "web"})),
    };

    assert_eq!(payload.client_id, "test_client");
    assert_eq!(payload.auth_token, "test_token");
    assert_eq!(payload.room_id, Some("room_123".to_string()));
    assert_eq!(payload.capabilities, Some(vec!["websocket".to_string(), "video".to_string()]));
    assert_eq!(payload.metadata, Some(json!({"version": "1.0", "platform": "web"})));
}

#[test]
fn test_registration_payload_without_room() {
    use signal_manager_service::database::RegistrationPayload;
    
    let payload = RegistrationPayload {
        client_id: "test_client_no_room".to_string(),
        auth_token: "test_token_no_room".to_string(),
        room_id: None,
        capabilities: Some(vec!["websocket".to_string()]),
        metadata: Some(json!({"version": "1.0", "platform": "mobile"})),
    };

    assert_eq!(payload.client_id, "test_client_no_room");
    assert_eq!(payload.auth_token, "test_token_no_room");
    assert_eq!(payload.room_id, None);
    assert_eq!(payload.capabilities, Some(vec!["websocket".to_string()]));
    assert_eq!(payload.metadata, Some(json!({"version": "1.0", "platform": "mobile"})));
} 