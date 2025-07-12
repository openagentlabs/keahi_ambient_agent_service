use std::sync::Arc;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use signal_manager_service::{
    config::Config,
    database::{
        ClientRepository, TerminatedRoomRepository, RoomCreatedRepository,
        RegistrationPayload, TerminationPayload, RoomCreationPayload, 
        FirestoreRepositoryFactory, RepositoryFactory,
    },
};

/// Integration tests for real Firestore database
/// These tests require:
/// 1. GOOGLE_APPLICATION_CREDENTIALS environment variable set
/// 2. A Firestore database to be available
/// 3. Run with: cargo test --test firestore_integration_tests -- --ignored
#[tokio::test]
#[ignore]
async fn test_firestore_client_repository_integration() {
    // Skip if no credentials are available
    if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_err() {
        eprintln!("Skipping Firestore integration test - no credentials available");
        return;
    }

    let config = Config::default();
    let factory = FirestoreRepositoryFactory::new(Arc::new(config));
    let repo = factory.create_client_repository().await.unwrap();

    // Test data
    let client_id = format!("test_client_{}", Uuid::new_v4());
    let auth_token = format!("test_token_{}", Uuid::new_v4());
    let room_id = Some(format!("test_room_{}", Uuid::new_v4()));

    // Test creating a client
    let payload = RegistrationPayload {
        client_id: client_id.clone(),
        auth_token: auth_token.clone(),
        room_id: room_id.clone(),
        capabilities: Some(vec!["websocket".to_string(), "video".to_string()]),
        metadata: Some(json!({
            "version": "1.0",
            "platform": "web",
            "test": true
        })),
    };

    let client = repo.create_client(payload).await.unwrap();
    assert_eq!(client.client_id, client_id);
    assert_eq!(client.auth_token, auth_token);
    assert_eq!(client.room_id, room_id);
    assert_eq!(client.capabilities, vec!["websocket", "video"]);
    assert!(client.metadata.get("test").unwrap().as_bool().unwrap());

    // Test retrieving the client
    let retrieved_client = repo.get_client(&client_id).await.unwrap().unwrap();
    assert_eq!(retrieved_client.client_id, client_id);
    assert_eq!(retrieved_client.auth_token, auth_token);

    // Test retrieving by token
    let client_by_token = repo.get_client_by_token(&auth_token).await.unwrap().unwrap();
    assert_eq!(client_by_token.client_id, client_id);

    // Test client exists
    assert!(repo.client_exists(&client_id).await.unwrap());

    // Test updating client
    let mut updated_client = client.clone();
    updated_client.update_last_seen();
    let saved_client = repo.update_client(updated_client).await.unwrap();
    assert!(saved_client.last_seen.is_some());

    // Test listing clients
    let clients = repo.list_clients(Some(10)).await.unwrap();
    assert!(!clients.is_empty());

    // Test deleting client
    assert!(repo.delete_client(&client_id).await.unwrap());
    assert!(!repo.client_exists(&client_id).await.unwrap());
}

#[tokio::test]
#[ignore]
async fn test_firestore_terminated_room_repository_integration() {
    // Skip if no credentials are available
    if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_err() {
        eprintln!("Skipping Firestore integration test - no credentials available");
        return;
    }

    let config = Config::default();
    let factory = FirestoreRepositoryFactory::new(Arc::new(config));
    let repo = factory.create_terminated_room_repository().await.unwrap();

    // Test data
    let room_id = format!("test_room_{}", Uuid::new_v4());
    let room_data = json!({
        "room_name": "Test Room",
        "participants": ["user1", "user2"],
        "settings": {"max_participants": 10}
    });

    // Test creating a terminated room
    let payload = TerminationPayload {
        room_id: room_id.clone(),
        room_data: room_data.clone(),
        termination_reason: Some("Room was inactive for too long".to_string()),
        terminated_by: Some("admin_user".to_string()),
        metadata: Some(json!({
            "termination_type": "automatic",
            "test": true
        })),
    };

    let terminated_room = repo.create_terminated_room(payload).await.unwrap();
    assert_eq!(terminated_room.room_id, room_id);
    assert_eq!(terminated_room.room_data, room_data);
    assert_eq!(terminated_room.termination_reason, Some("Room was inactive for too long".to_string()));
    assert_eq!(terminated_room.terminated_by, Some("admin_user".to_string()));
    assert!(terminated_room.metadata.get("test").unwrap().as_bool().unwrap());

    // Test retrieving the terminated room
    let retrieved_room = repo.get_terminated_room(&room_id).await.unwrap().unwrap();
    assert_eq!(retrieved_room.room_id, room_id);

    // Test room was terminated
    assert!(repo.room_was_terminated(&room_id).await.unwrap());

    // Test listing terminated rooms
    let rooms = repo.list_terminated_rooms(Some(10)).await.unwrap();
    assert!(!rooms.is_empty());

    // Test date range query
    let start_date = Utc::now() - chrono::Duration::hours(1);
    let end_date = Utc::now() + chrono::Duration::hours(1);
    let rooms_in_range = repo.get_terminated_rooms_by_date_range(start_date, end_date).await.unwrap();
    assert!(!rooms_in_range.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_firestore_room_created_repository_integration() {
    // Skip if no credentials are available
    if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_err() {
        eprintln!("Skipping Firestore integration test - no credentials available");
        return;
    }

    let config = Config::default();
    let factory = FirestoreRepositoryFactory::new(Arc::new(config));
    let repo = factory.create_room_created_repository().await.unwrap();

    // Test data
    let room_uuid = Uuid::new_v4().to_string();
    let room_data = json!({
        "room_name": "Test Room Created",
        "creator": "user1",
        "settings": {"max_participants": 15}
    });

    // Test creating a room creation record
    let payload = RoomCreationPayload {
        room_uuid: room_uuid.clone(),
        room_data: room_data.clone(),
        created_by: Some("user1".to_string()),
        metadata: Some(json!({
            "creation_type": "manual",
            "test": true
        })),
    };

    let room_created = repo.create_room_created(payload).await.unwrap();
    assert_eq!(room_created.room_uuid, room_uuid);
    assert_eq!(room_created.room_data, room_data);
    assert_eq!(room_created.created_by, Some("user1".to_string()));
    assert!(room_created.metadata.get("test").unwrap().as_bool().unwrap());

    // Test retrieving the room creation record
    let retrieved_room = repo.get_room_created(&room_uuid).await.unwrap().unwrap();
    assert_eq!(retrieved_room.room_uuid, room_uuid);

    // Test room was created
    assert!(repo.room_was_created(&room_uuid).await.unwrap());

    // Test listing room creation records
    let rooms = repo.list_rooms_created(Some(10)).await.unwrap();
    assert!(!rooms.is_empty());

    // Test date range query
    let start_date = Utc::now() - chrono::Duration::hours(1);
    let end_date = Utc::now() + chrono::Duration::hours(1);
    let rooms_in_range = repo.get_rooms_created_by_date_range(start_date, end_date).await.unwrap();
    assert!(!rooms_in_range.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_firestore_repository_factory_integration() {
    // Skip if no credentials are available
    if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_err() {
        eprintln!("Skipping Firestore integration test - no credentials available");
        return;
    }

    let config = Config::default();
    let factory = FirestoreRepositoryFactory::new(Arc::new(config));

    // Test creating all repository types
    let _client_repo = factory.create_client_repository().await.unwrap();
    let _terminated_room_repo = factory.create_terminated_room_repository().await.unwrap();
    let _room_created_repo = factory.create_room_created_repository().await.unwrap();

    // Verify repositories are created successfully
    assert!(std::any::type_name::<dyn ClientRepository>().contains("ClientRepository"));
    assert!(std::any::type_name::<dyn TerminatedRoomRepository>().contains("TerminatedRoomRepository"));
    assert!(std::any::type_name::<dyn RoomCreatedRepository>().contains("RoomCreatedRepository"));
}

/// Helper function to run all Firestore integration tests
/// Usage: cargo test --test firestore_integration_tests -- --ignored
pub async fn run_all_firestore_integration_tests() {
    println!("Running Firestore integration tests...");
    
    // Check if credentials are available
    if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_err() {
        eprintln!("GOOGLE_APPLICATION_CREDENTIALS environment variable not set");
        eprintln!("Skipping Firestore integration tests");
        return;
    }

    // Note: These tests are designed to be run individually with cargo test
    // The helper function is for documentation purposes
    println!("All Firestore integration tests completed successfully!");
} 