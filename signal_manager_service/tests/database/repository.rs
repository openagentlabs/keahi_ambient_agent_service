use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use chrono::Utc;

use signal_manager_service::database::{
    ClientRepository, RegisteredClient, RegistrationPayload, DatabaseResult,
    TerminatedRoomRepository, TerminatedRoom, TerminationPayload,
    RoomCreatedRepository, RoomCreated, RoomCreationPayload,
    RepositoryFactory, ClientStatus,
    ClientInRoomRepository, ClientInRoom,
    ClientInTerminatedRoomRepository, ClientInTerminatedRoom,
    ClientInRoomStatus, ClientTerminationStatus,
};

/// Mock implementation of ClientRepository for testing
pub struct MockClientRepository {
    clients: Arc<Mutex<HashMap<String, RegisteredClient>>>,
}

/// Mock implementation of TerminatedRoomRepository for testing
pub struct MockTerminatedRoomRepository {
    terminated_rooms: Arc<Mutex<HashMap<String, TerminatedRoom>>>,
}

/// Mock implementation of RoomCreatedRepository for testing
pub struct MockRoomCreatedRepository {
    rooms_created: Arc<Mutex<HashMap<String, RoomCreated>>>,
}

/// Mock implementation of ClientInRoomRepository for testing
pub struct MockClientInRoomRepository {
    clients_in_room: Arc<Mutex<HashMap<String, ClientInRoom>>>,
}

/// Mock implementation of ClientInTerminatedRoomRepository for testing
pub struct MockClientInTerminatedRoomRepository {
    clients_in_terminated_room: Arc<Mutex<HashMap<String, ClientInTerminatedRoom>>>,
}

/// Mock repository factory for testing
pub struct MockRepositoryFactory;

impl MockClientRepository {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockTerminatedRoomRepository {
    pub fn new() -> Self {
        Self {
            terminated_rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockRoomCreatedRepository {
    pub fn new() -> Self {
        Self {
            rooms_created: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockClientInRoomRepository {
    pub fn new() -> Self {
        Self {
            clients_in_room: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockClientInTerminatedRoomRepository {
    pub fn new() -> Self {
        Self {
            clients_in_terminated_room: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ClientRepository for MockClientRepository {
    async fn create_client(&self, payload: RegistrationPayload) -> DatabaseResult<RegisteredClient> {
        let mut clients = self.clients.lock().await;
        
        // Check if client already exists
        if clients.contains_key(&payload.client_id) {
            return Err(signal_manager_service::database::DatabaseError::Validation(
                format!("Client {} already exists", payload.client_id)
            ));
        }

        let client = if let Some(room_id) = payload.room_id {
            RegisteredClient::new_with_room(
                payload.client_id.clone(),
                payload.auth_token,
                room_id,
                payload.capabilities.unwrap_or_default(),
                payload.metadata.unwrap_or_default(),
            )
        } else {
            RegisteredClient::new(
                payload.client_id.clone(),
                payload.auth_token,
                payload.capabilities.unwrap_or_default(),
                payload.metadata.unwrap_or_default(),
            )
        };

        clients.insert(payload.client_id, client.clone());
        Ok(client)
    }

    async fn get_client(&self, client_id: &str) -> DatabaseResult<Option<RegisteredClient>> {
        let clients = self.clients.lock().await;
        Ok(clients.get(client_id).cloned())
    }

    async fn get_client_by_token(&self, auth_token: &str) -> DatabaseResult<Option<RegisteredClient>> {
        let clients = self.clients.lock().await;
        Ok(clients.values().find(|c| c.auth_token == auth_token).cloned())
    }

    async fn update_client(&self, client: RegisteredClient) -> DatabaseResult<RegisteredClient> {
        let mut clients = self.clients.lock().await;
        let mut updated_client = client;
        updated_client.update_last_seen();
        clients.insert(updated_client.client_id.clone(), updated_client.clone());
        Ok(updated_client)
    }

    async fn delete_client(&self, client_id: &str) -> DatabaseResult<bool> {
        let mut clients = self.clients.lock().await;
        Ok(clients.remove(client_id).is_some())
    }

    async fn list_clients(&self, limit: Option<usize>) -> DatabaseResult<Vec<RegisteredClient>> {
        let clients = self.clients.lock().await;
        let mut result: Vec<_> = clients.values().cloned().collect();
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }

    async fn update_last_seen(&self, client_id: &str) -> DatabaseResult<bool> {
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.update_last_seen();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn client_exists(&self, client_id: &str) -> DatabaseResult<bool> {
        let clients = self.clients.lock().await;
        Ok(clients.contains_key(client_id))
    }

    async fn validate_auth(&self, client_id: &str, auth_token: &str) -> DatabaseResult<bool> {
        let clients = self.clients.lock().await;
        Ok(clients.get(client_id)
            .map(|c| c.auth_token == auth_token && c.is_active())
            .unwrap_or(false))
    }
}

#[async_trait]
impl TerminatedRoomRepository for MockTerminatedRoomRepository {
    async fn create_terminated_room(&self, payload: TerminationPayload) -> DatabaseResult<TerminatedRoom> {
        let mut rooms = self.terminated_rooms.lock().await;
        
        // Check if room was already terminated
        if rooms.contains_key(&payload.room_id) {
            return Err(signal_manager_service::database::DatabaseError::Validation(
                format!("Room {} was already terminated", payload.room_id)
            ));
        }

        let terminated_room = TerminatedRoom::new(
            payload.room_id.clone(),
            payload.room_data,
            payload.termination_reason,
            payload.terminated_by,
            payload.metadata,
        );

        rooms.insert(payload.room_id, terminated_room.clone());
        Ok(terminated_room)
    }

    async fn get_terminated_room(&self, room_id: &str) -> DatabaseResult<Option<TerminatedRoom>> {
        let rooms = self.terminated_rooms.lock().await;
        Ok(rooms.get(room_id).cloned())
    }

    async fn list_terminated_rooms(&self, limit: Option<usize>) -> DatabaseResult<Vec<TerminatedRoom>> {
        let rooms = self.terminated_rooms.lock().await;
        let mut result: Vec<_> = rooms.values().cloned().collect();
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }

    async fn room_was_terminated(&self, room_id: &str) -> DatabaseResult<bool> {
        let rooms = self.terminated_rooms.lock().await;
        Ok(rooms.contains_key(room_id))
    }

    async fn get_terminated_rooms_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<TerminatedRoom>> {
        let rooms = self.terminated_rooms.lock().await;
        let result: Vec<_> = rooms.values()
            .filter(|room| room.terminated_at >= start_date && room.terminated_at <= end_date)
            .cloned()
            .collect();
        
        Ok(result)
    }
}

#[async_trait]
impl RoomCreatedRepository for MockRoomCreatedRepository {
    async fn create_room_created(&self, payload: RoomCreationPayload) -> DatabaseResult<RoomCreated> {
        let mut rooms = self.rooms_created.lock().await;
        
        // Check if room was already created
        if rooms.contains_key(&payload.room_uuid) {
            return Err(signal_manager_service::database::DatabaseError::Validation(
                format!("Room {} was already created", payload.room_uuid)
            ));
        }

        let room_created = RoomCreated::new(
            payload.room_uuid.clone(),
            payload.room_data,
            payload.created_by,
            payload.metadata,
        );

        rooms.insert(payload.room_uuid, room_created.clone());
        Ok(room_created)
    }

    async fn get_room_created(&self, room_uuid: &str) -> DatabaseResult<Option<RoomCreated>> {
        let rooms = self.rooms_created.lock().await;
        Ok(rooms.get(room_uuid).cloned())
    }

    async fn list_rooms_created(&self, limit: Option<usize>) -> DatabaseResult<Vec<RoomCreated>> {
        let rooms = self.rooms_created.lock().await;
        let mut result: Vec<_> = rooms.values().cloned().collect();
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }

    async fn room_was_created(&self, room_uuid: &str) -> DatabaseResult<bool> {
        let rooms = self.rooms_created.lock().await;
        Ok(rooms.contains_key(room_uuid))
    }

    async fn get_rooms_created_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<RoomCreated>> {
        let rooms = self.rooms_created.lock().await;
        let result: Vec<_> = rooms.values()
            .filter(|room| room.created_at >= start_date && room.created_at <= end_date)
            .cloned()
            .collect();
        
        Ok(result)
    }
}

#[async_trait]
impl ClientInRoomRepository for MockClientInRoomRepository {
    async fn create_client_in_room(&self, client_in_room: ClientInRoom) -> DatabaseResult<ClientInRoom> {
        let mut clients = self.clients_in_room.lock().await;
        
        // Check if client is already in the room
        if clients.contains_key(&client_in_room.id) {
            return Err(signal_manager_service::database::DatabaseError::Validation(
                format!("Client in room {} already exists", client_in_room.id)
            ));
        }

        clients.insert(client_in_room.id.clone(), client_in_room.clone());
        Ok(client_in_room)
    }

    async fn get_client_in_room(&self, id: &str) -> DatabaseResult<Option<ClientInRoom>> {
        let clients = self.clients_in_room.lock().await;
        Ok(clients.get(id).cloned())
    }

    async fn get_clients_in_room(&self, room_id: &str) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients = self.clients_in_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.room_id == room_id)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn list_clients_in_rooms(&self) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients = self.clients_in_room.lock().await;
        Ok(clients.values().cloned().collect())
    }

    async fn update_client_in_room(&self, id: &str, client_in_room: ClientInRoom) -> DatabaseResult<ClientInRoom> {
        let mut clients = self.clients_in_room.lock().await;
        clients.insert(id.to_string(), client_in_room.clone());
        Ok(client_in_room)
    }

    async fn update_client_status(&self, id: &str, status: ClientInRoomStatus) -> DatabaseResult<ClientInRoom> {
        let mut clients = self.clients_in_room.lock().await;
        if let Some(client) = clients.get_mut(id) {
            client.update_status(status);
            Ok(client.clone())
        } else {
            Err(signal_manager_service::database::DatabaseError::NotFound(format!("Client in room {} not found", id)))
        }
    }

    async fn update_client_last_activity(&self, id: &str) -> DatabaseResult<ClientInRoom> {
        let mut clients = self.clients_in_room.lock().await;
        if let Some(client) = clients.get_mut(id) {
            client.update_last_activity();
            Ok(client.clone())
        } else {
            Err(signal_manager_service::database::DatabaseError::NotFound(format!("Client in room {} not found", id)))
        }
    }

    async fn remove_client_from_room(&self, id: &str) -> DatabaseResult<()> {
        let mut clients = self.clients_in_room.lock().await;
        clients.remove(id);
        Ok(())
    }

    async fn client_exists_in_room(&self, client_id: &str, room_id: &str) -> DatabaseResult<bool> {
        let clients = self.clients_in_room.lock().await;
        Ok(clients.values().any(|c| c.client_id == client_id && c.room_id == room_id))
    }

    async fn get_clients_by_status(&self, room_id: &str, status: ClientInRoomStatus) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients = self.clients_in_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.room_id == room_id && c.status == status)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_active_clients_in_room(&self, room_id: &str) -> DatabaseResult<Vec<ClientInRoom>> {
        self.get_clients_by_status(room_id, ClientInRoomStatus::Active).await
    }

    async fn get_clients_in_room_by_date_range(
        &self,
        room_id: &str,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients = self.clients_in_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.room_id == room_id && c.joined_at >= start_date && c.joined_at <= end_date)
            .cloned()
            .collect();
        Ok(result)
    }
}

#[async_trait]
impl ClientInTerminatedRoomRepository for MockClientInTerminatedRoomRepository {
    async fn create_client_in_terminated_room(&self, client_in_terminated_room: ClientInTerminatedRoom) -> DatabaseResult<ClientInTerminatedRoom> {
        let mut clients = self.clients_in_terminated_room.lock().await;
        
        // Check if client is already in the terminated room
        if clients.contains_key(&client_in_terminated_room.id) {
            return Err(signal_manager_service::database::DatabaseError::Validation(
                format!("Client in terminated room {} already exists", client_in_terminated_room.id)
            ));
        }

        clients.insert(client_in_terminated_room.id.clone(), client_in_terminated_room.clone());
        Ok(client_in_terminated_room)
    }

    async fn get_client_in_terminated_room(&self, id: &str) -> DatabaseResult<Option<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        Ok(clients.get(id).cloned())
    }

    async fn get_clients_from_terminated_room(&self, room_id: &str) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.room_id == room_id)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn list_clients_in_terminated_rooms(&self) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        Ok(clients.values().cloned().collect())
    }

    async fn update_client_in_terminated_room(&self, id: &str, client_in_terminated_room: ClientInTerminatedRoom) -> DatabaseResult<ClientInTerminatedRoom> {
        let mut clients = self.clients_in_terminated_room.lock().await;
        clients.insert(id.to_string(), client_in_terminated_room.clone());
        Ok(client_in_terminated_room)
    }

    async fn get_clients_by_termination_status(&self, status: ClientTerminationStatus) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.final_status == status)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_clients_terminated_by(&self, terminated_by: &str) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.terminated_by == terminated_by)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_clients_by_termination_reason(&self, reason: &str) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.termination_reason == reason)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn client_was_in_terminated_room(&self, client_id: &str, room_id: &str) -> DatabaseResult<bool> {
        let clients = self.clients_in_terminated_room.lock().await;
        Ok(clients.values().any(|c| c.client_id == client_id && c.room_id == room_id))
    }

    async fn get_clients_in_terminated_rooms_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.left_at >= start_date && c.left_at <= end_date)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_clients_from_terminated_room_by_date_range(
        &self,
        room_id: &str,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.room_id == room_id && c.left_at >= start_date && c.left_at <= end_date)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_clients_terminated_between(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients = self.clients_in_terminated_room.lock().await;
        let result: Vec<_> = clients.values()
            .filter(|c| c.left_at >= start_date && c.left_at <= end_date)
            .cloned()
            .collect();
        Ok(result)
    }
}

#[async_trait]
impl RepositoryFactory for MockRepositoryFactory {
    async fn create_client_repository(&self) -> DatabaseResult<Arc<dyn ClientRepository + Send + Sync>> {
        Ok(Arc::new(MockClientRepository::new()))
    }

    async fn create_terminated_room_repository(&self) -> DatabaseResult<Arc<dyn TerminatedRoomRepository + Send + Sync>> {
        Ok(Arc::new(MockTerminatedRoomRepository::new()))
    }

    async fn create_room_created_repository(&self) -> DatabaseResult<Arc<dyn RoomCreatedRepository + Send + Sync>> {
        Ok(Arc::new(MockRoomCreatedRepository::new()))
    }

    async fn create_client_in_room_repository(&self) -> DatabaseResult<Arc<dyn ClientInRoomRepository + Send + Sync>> {
        Ok(Arc::new(MockClientInRoomRepository::new()))
    }

    async fn create_client_in_terminated_room_repository(&self) -> DatabaseResult<Arc<dyn ClientInTerminatedRoomRepository + Send + Sync>> {
        Ok(Arc::new(MockClientInTerminatedRoomRepository::new()))
    }
}

#[tokio::test]
async fn test_mock_repository_create_client() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: Some(vec!["websocket".to_string()]),
        metadata: Some(serde_json::json!({"version": "1.0"})),
        room_id: None,
    };

    let result = repo.create_client(payload).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.client_id, "test_client");
    assert_eq!(client.auth_token, "test_token");
    assert_eq!(client.capabilities, vec!["websocket"]);
    assert_eq!(client.metadata, serde_json::json!({"version": "1.0"}));
}

#[tokio::test]
async fn test_mock_repository_create_duplicate_client() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // First creation should succeed
    let result1 = repo.create_client(payload.clone()).await;
    assert!(result1.is_ok());

    // Second creation with same client_id should fail
    let result2 = repo.create_client(payload).await;
    assert!(result2.is_err());
    
    if let Err(error) = result2 {
        match error {
            signal_manager_service::database::DatabaseError::Validation(msg) => {
                assert!(msg.contains("already exists"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

#[tokio::test]
async fn test_mock_repository_get_client() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    repo.create_client(payload).await.unwrap();

    // Get existing client
    let result = repo.get_client("test_client").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Get non-existing client
    let result = repo.get_client("non_existent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_mock_repository_get_client_by_token() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    repo.create_client(payload).await.unwrap();

    // Get by correct token
    let result = repo.get_client_by_token("test_token").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Get by wrong token
    let result = repo.get_client_by_token("wrong_token").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_mock_repository_update_client() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    let client = repo.create_client(payload).await.unwrap();
    let original_last_seen = client.last_seen;

    // Update client
    let updated_client = repo.update_client(client).await.unwrap();
    assert!(updated_client.last_seen.is_some());
    assert_ne!(updated_client.last_seen, original_last_seen);
}

#[tokio::test]
async fn test_mock_repository_delete_client() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    repo.create_client(payload).await.unwrap();

    // Delete existing client
    let result = repo.delete_client("test_client").await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Delete non-existing client
    let result = repo.delete_client("non_existent").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_mock_repository_list_clients() {
    let repo = MockClientRepository::new();
    
    // Create multiple clients
    for i in 1..=5 {
        let payload = RegistrationPayload {
            client_id: format!("client_{}", i),
            auth_token: format!("token_{}", i),
            capabilities: None,
            metadata: None,
            room_id: None,
        };
        repo.create_client(payload).await.unwrap();
    }

    // List all clients
    let result = repo.list_clients(None).await;
    assert!(result.is_ok());
    let clients = result.unwrap();
    assert_eq!(clients.len(), 5);

    // List with limit
    let result = repo.list_clients(Some(3)).await;
    assert!(result.is_ok());
    let clients = result.unwrap();
    assert_eq!(clients.len(), 3);
}

#[tokio::test]
async fn test_mock_repository_update_last_seen() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    repo.create_client(payload).await.unwrap();

    // Update last seen for existing client
    let result = repo.update_last_seen("test_client").await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Update last seen for non-existing client
    let result = repo.update_last_seen("non_existent").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_mock_repository_client_exists() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Check non-existing client
    let result = repo.client_exists("test_client").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Create client
    repo.create_client(payload).await.unwrap();

    // Check existing client
    let result = repo.client_exists("test_client").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_mock_repository_validate_auth() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    repo.create_client(payload).await.unwrap();

    // Validate with correct credentials
    let result = repo.validate_auth("test_client", "test_token").await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Validate with wrong token
    let result = repo.validate_auth("test_client", "wrong_token").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Validate with non-existing client
    let result = repo.validate_auth("non_existent", "test_token").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_mock_repository_factory() {
    let factory = MockRepositoryFactory;
    let repo = factory.create_client_repository().await;
    assert!(repo.is_ok());
}

#[tokio::test]
async fn test_mock_repository_with_inactive_client() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
        capabilities: None,
        metadata: None,
        room_id: None,
    };

    // Create client
    let mut client = repo.create_client(payload).await.unwrap();
    
    // Make client inactive
    client.status = signal_manager_service::database::ClientStatus::Inactive;
    repo.update_client(client).await.unwrap();

    // Validate auth should fail for inactive client
    let result = repo.validate_auth("test_client", "test_token").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
} 

#[tokio::test]
async fn test_mock_repository_create_client_with_room() {
    let repo = MockClientRepository::new();
    
    let payload = RegistrationPayload {
        client_id: "test_client_with_room".to_string(),
        auth_token: "test_token_with_room".to_string(),
        room_id: Some("room_123".to_string()),
        capabilities: Some(vec!["websocket".to_string(), "video".to_string()]),
        metadata: Some(serde_json::json!({"version": "1.0", "platform": "web"})),
    };

    let client = repo.create_client(payload).await.unwrap();
    
    assert_eq!(client.client_id, "test_client_with_room");
    assert_eq!(client.auth_token, "test_token_with_room");
    assert_eq!(client.room_id, Some("room_123".to_string()));
    assert_eq!(client.capabilities, vec!["websocket", "video"]);
    assert_eq!(client.metadata, serde_json::json!({"version": "1.0", "platform": "web"}));
    assert_eq!(client.status, ClientStatus::Active);
    assert!(client.last_seen.is_none());
    
    // Verify the client was stored
    let retrieved = repo.get_client("test_client_with_room").await.unwrap().unwrap();
    assert_eq!(retrieved.room_id, Some("room_123".to_string()));
}

#[tokio::test]
async fn test_terminated_room_repository_create() {
    let repo = MockTerminatedRoomRepository::new();
    
    let payload = TerminationPayload {
        room_id: "room_123".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        termination_reason: Some("Room was inactive".to_string()),
        terminated_by: Some("admin".to_string()),
        metadata: Some(serde_json::json!({"reason": "inactivity"})),
    };

    let result = repo.create_terminated_room(payload).await;
    assert!(result.is_ok());

    let terminated_room = result.unwrap();
    assert_eq!(terminated_room.room_id, "room_123");
    assert_eq!(terminated_room.termination_reason, Some("Room was inactive".to_string()));
    assert_eq!(terminated_room.terminated_by, Some("admin".to_string()));
}

#[tokio::test]
async fn test_terminated_room_repository_duplicate_creation() {
    let repo = MockTerminatedRoomRepository::new();
    
    let payload = TerminationPayload {
        room_id: "room_123".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        termination_reason: None,
        terminated_by: None,
        metadata: None,
    };

    // First creation should succeed
    let result1 = repo.create_terminated_room(payload.clone()).await;
    assert!(result1.is_ok());

    // Second creation with same room_id should fail
    let result2 = repo.create_terminated_room(payload).await;
    assert!(result2.is_err());

    if let Err(error) = result2 {
        match error {
            signal_manager_service::database::DatabaseError::Validation(msg) => {
                assert!(msg.contains("already terminated"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

#[tokio::test]
async fn test_terminated_room_repository_get() {
    let repo = MockTerminatedRoomRepository::new();
    
    let payload = TerminationPayload {
        room_id: "room_123".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        termination_reason: None,
        terminated_by: None,
        metadata: None,
    };

    // Create a terminated room
    repo.create_terminated_room(payload).await.unwrap();

    // Get the terminated room
    let result = repo.get_terminated_room("room_123").await;
    assert!(result.is_ok());

    let terminated_room = result.unwrap();
    assert!(terminated_room.is_some());
    assert_eq!(terminated_room.unwrap().room_id, "room_123");

    // Get non-existent room
    let result = repo.get_terminated_room("non_existent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_terminated_room_repository_list() {
    let repo = MockTerminatedRoomRepository::new();
    
    // Create multiple terminated rooms
    let rooms = vec![
        ("room_1", "Room 1"),
        ("room_2", "Room 2"),
        ("room_3", "Room 3"),
    ];

    for (room_id, room_name) in rooms {
        let payload = TerminationPayload {
            room_id: room_id.to_string(),
            room_data: serde_json::json!({"room_name": room_name}),
            termination_reason: None,
            terminated_by: None,
            metadata: None,
        };
        repo.create_terminated_room(payload).await.unwrap();
    }

    // List all terminated rooms
    let result = repo.list_terminated_rooms(None).await;
    assert!(result.is_ok());

    let terminated_rooms = result.unwrap();
    assert_eq!(terminated_rooms.len(), 3);

    // List with limit
    let result = repo.list_terminated_rooms(Some(2)).await;
    assert!(result.is_ok());

    let terminated_rooms = result.unwrap();
    assert_eq!(terminated_rooms.len(), 2);
}

#[tokio::test]
async fn test_terminated_room_repository_was_terminated() {
    let repo = MockTerminatedRoomRepository::new();
    
    let payload = TerminationPayload {
        room_id: "room_123".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        termination_reason: None,
        terminated_by: None,
        metadata: None,
    };

    // Create a terminated room
    repo.create_terminated_room(payload).await.unwrap();

    // Check if room was terminated
    let result = repo.room_was_terminated("room_123").await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Check non-existent room
    let result = repo.room_was_terminated("non_existent").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_terminated_room_repository_date_range() {
    let repo = MockTerminatedRoomRepository::new();
    
    // Create terminated rooms
    let payload = TerminationPayload {
        room_id: "room_123".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        termination_reason: None,
        terminated_by: None,
        metadata: None,
    };

    repo.create_terminated_room(payload).await.unwrap();

    // Get rooms by date range
    let start_date = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(1);

    let result = repo.get_terminated_rooms_by_date_range(start_date, end_date).await;
    assert!(result.is_ok());

    let terminated_rooms = result.unwrap();
    assert_eq!(terminated_rooms.len(), 1);
    assert_eq!(terminated_rooms[0].room_id, "room_123");

    // Get rooms outside date range
    let start_date = chrono::Utc::now() + chrono::Duration::hours(2);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(3);

    let result = repo.get_terminated_rooms_by_date_range(start_date, end_date).await;
    assert!(result.is_ok());

    let terminated_rooms = result.unwrap();
    assert_eq!(terminated_rooms.len(), 0);
}

#[tokio::test]
async fn test_repository_factory_terminated_room() {
    let factory = MockRepositoryFactory;
    
    let result = factory.create_terminated_room_repository().await;
    assert!(result.is_ok());

    let repo = result.unwrap();
    
    // Test that the repository works
    let payload = TerminationPayload {
        room_id: "room_123".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        termination_reason: None,
        terminated_by: None,
        metadata: None,
    };

    let result = repo.create_terminated_room(payload).await;
    assert!(result.is_ok());
} 

#[tokio::test]
async fn test_room_created_repository_create() {
    let repo = MockRoomCreatedRepository::new();
    
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        created_by: Some("user123".to_string()),
        metadata: Some(serde_json::json!({"creation_type": "manual"})),
    };

    let result = repo.create_room_created(payload).await;
    assert!(result.is_ok());

    let room_created = result.unwrap();
    assert_eq!(room_created.room_uuid, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(room_created.created_by, Some("user123".to_string()));
    assert_eq!(room_created.metadata, serde_json::json!({"creation_type": "manual"}));
}

#[tokio::test]
async fn test_room_created_repository_duplicate_creation() {
    let repo = MockRoomCreatedRepository::new();
    
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        created_by: None,
        metadata: None,
    };

    // First creation should succeed
    let result1 = repo.create_room_created(payload.clone()).await;
    assert!(result1.is_ok());

    // Second creation with same room_uuid should fail
    let result2 = repo.create_room_created(payload).await;
    assert!(result2.is_err());

    if let Err(error) = result2 {
        match error {
            signal_manager_service::database::DatabaseError::Validation(msg) => {
                assert!(msg.contains("already created"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

#[tokio::test]
async fn test_room_created_repository_get() {
    let repo = MockRoomCreatedRepository::new();
    
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        created_by: None,
        metadata: None,
    };

    // Create a room creation record
    repo.create_room_created(payload).await.unwrap();

    // Get the room creation record
    let result = repo.get_room_created("550e8400-e29b-41d4-a716-446655440000").await;
    assert!(result.is_ok());

    let room_created = result.unwrap();
    assert!(room_created.is_some());
    assert_eq!(room_created.unwrap().room_uuid, "550e8400-e29b-41d4-a716-446655440000");

    // Get non-existent room
    let result = repo.get_room_created("non_existent_uuid").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_room_created_repository_list() {
    let repo = MockRoomCreatedRepository::new();
    
    // Create multiple room creation records
    let rooms = vec![
        ("550e8400-e29b-41d4-a716-446655440001", "Room 1"),
        ("550e8400-e29b-41d4-a716-446655440002", "Room 2"),
        ("550e8400-e29b-41d4-a716-446655440003", "Room 3"),
    ];

    for (room_uuid, room_name) in rooms {
        let payload = RoomCreationPayload {
            room_uuid: room_uuid.to_string(),
            room_data: serde_json::json!({"room_name": room_name}),
            created_by: None,
            metadata: None,
        };
        repo.create_room_created(payload).await.unwrap();
    }

    // List all room creation records
    let result = repo.list_rooms_created(None).await;
    assert!(result.is_ok());

    let rooms_created = result.unwrap();
    assert_eq!(rooms_created.len(), 3);

    // List with limit
    let result = repo.list_rooms_created(Some(2)).await;
    assert!(result.is_ok());

    let rooms_created = result.unwrap();
    assert_eq!(rooms_created.len(), 2);
}

#[tokio::test]
async fn test_room_created_repository_was_created() {
    let repo = MockRoomCreatedRepository::new();
    
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        created_by: None,
        metadata: None,
    };

    // Create a room creation record
    repo.create_room_created(payload).await.unwrap();

    // Check if room was created
    let result = repo.room_was_created("550e8400-e29b-41d4-a716-446655440000").await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Check non-existent room
    let result = repo.room_was_created("non_existent_uuid").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_room_created_repository_date_range() {
    let repo = MockRoomCreatedRepository::new();
    
    // Create room creation records
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        created_by: None,
        metadata: None,
    };

    repo.create_room_created(payload).await.unwrap();

    // Get rooms by date range
    let start_date = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(1);

    let result = repo.get_rooms_created_by_date_range(start_date, end_date).await;
    assert!(result.is_ok());

    let rooms_created = result.unwrap();
    assert_eq!(rooms_created.len(), 1);
    assert_eq!(rooms_created[0].room_uuid, "550e8400-e29b-41d4-a716-446655440000");

    // Get rooms outside date range
    let start_date = chrono::Utc::now() + chrono::Duration::hours(2);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(3);

    let result = repo.get_rooms_created_by_date_range(start_date, end_date).await;
    assert!(result.is_ok());

    let rooms_created = result.unwrap();
    assert_eq!(rooms_created.len(), 0);
}

#[tokio::test]
async fn test_repository_factory_room_created() {
    let factory = MockRepositoryFactory;
    
    let result = factory.create_room_created_repository().await;
    assert!(result.is_ok());

    let repo = result.unwrap();
    
    // Test that the repository works
    let payload = RoomCreationPayload {
        room_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        room_data: serde_json::json!({"room_name": "test_room"}),
        created_by: None,
        metadata: None,
    };

    let result = repo.create_room_created(payload).await;
    assert!(result.is_ok());
} 

#[tokio::test]
async fn test_client_in_room_repository_create() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    let result = repo.create_client_in_room(client_in_room.clone()).await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.client_id, "client_123");
    assert_eq!(created.room_id, "room_456");
    assert_eq!(created.status, ClientInRoomStatus::Active);
    assert_eq!(created.metadata, serde_json::json!({}));
}

#[tokio::test]
async fn test_client_in_room_repository_duplicate_creation() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // First creation should succeed
    let result1 = repo.create_client_in_room(client_in_room.clone()).await;
    assert!(result1.is_ok());

    // Second creation with same id should fail
    let result2 = repo.create_client_in_room(client_in_room).await;
    assert!(result2.is_err());

    if let Err(error) = result2 {
        match error {
            signal_manager_service::database::DatabaseError::Validation(msg) => {
                assert!(msg.contains("already exists"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

#[tokio::test]
async fn test_client_in_room_repository_get() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in room
    let created_client = repo.create_client_in_room(client_in_room).await.unwrap();

    // Get existing client in room
    let result = repo.get_client_in_room(&created_client.id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Get non-existing client in room
    let result = repo.get_client_in_room("non_existent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_client_in_room_repository_get_clients_in_room() {
    let repo = MockClientInRoomRepository::new();
    
    // Create multiple clients in the same room
    let clients = vec![
        ("client_1", "room_123"),
        ("client_2", "room_123"),
        ("client_3", "room_456"),
    ];

    for (client_id, room_id) in clients {
        let client_in_room = ClientInRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_room(client_in_room).await.unwrap();
    }

    // Get clients in room_123
    let result = repo.get_clients_in_room("room_123").await;
    assert!(result.is_ok());
    let clients_in_room = result.unwrap();
    assert_eq!(clients_in_room.len(), 2);

    // Get clients in room_456
    let result = repo.get_clients_in_room("room_456").await;
    assert!(result.is_ok());
    let clients_in_room = result.unwrap();
    assert_eq!(clients_in_room.len(), 1);
}

#[tokio::test]
async fn test_client_in_room_repository_list_all() {
    let repo = MockClientInRoomRepository::new();
    
    // Create multiple clients in rooms
    let clients = vec![
        ("client_1", "room_123"),
        ("client_2", "room_123"),
        ("client_3", "room_456"),
    ];

    for (client_id, room_id) in clients {
        let client_in_room = ClientInRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_room(client_in_room).await.unwrap();
    }

    // List all clients in rooms
    let result = repo.list_clients_in_rooms().await;
    assert!(result.is_ok());
    let all_clients = result.unwrap();
    assert_eq!(all_clients.len(), 3);
}

#[tokio::test]
async fn test_client_in_room_repository_update() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in room
    let created_client = repo.create_client_in_room(client_in_room).await.unwrap();

    // Update client in room
    let updated_client = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({"updated": true})),
    );

    let result = repo.update_client_in_room(&created_client.id, updated_client.clone()).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.status, ClientInRoomStatus::Active);
    assert_eq!(updated.metadata, serde_json::json!({"updated": true}));
}

#[tokio::test]
async fn test_client_in_room_repository_update_status() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in room
    let created_client = repo.create_client_in_room(client_in_room).await.unwrap();

    // Update status
    let result = repo.update_client_status(&created_client.id, ClientInRoomStatus::Inactive).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.status, ClientInRoomStatus::Inactive);

    // Update status for non-existing client
    let result = repo.update_client_status("non_existent", ClientInRoomStatus::Active).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_in_room_repository_update_last_activity() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in room
    let created = repo.create_client_in_room(client_in_room).await.unwrap();
    let original_last_activity = created.last_activity;

    // Update last activity
    let result = repo.update_client_last_activity(&created.id).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert!(updated.last_activity > original_last_activity);

    // Update last activity for non-existing client
    let result = repo.update_client_last_activity("non_existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_in_room_repository_remove() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in room
    let created_client = repo.create_client_in_room(client_in_room).await.unwrap();

    // Remove client from room
    let result = repo.remove_client_from_room(&created_client.id).await;
    assert!(result.is_ok());

    // Verify client was removed
    let result = repo.get_client_in_room(&created_client.id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_client_in_room_repository_exists() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Check non-existing client in room
    let result = repo.client_exists_in_room("client_123", "room_456").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Create client in room
    repo.create_client_in_room(client_in_room).await.unwrap();

    // Check existing client in room
    let result = repo.client_exists_in_room("client_123", "room_456").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_client_in_room_repository_get_by_status() {
    let repo = MockClientInRoomRepository::new();
    
    // Create clients with different statuses
    let clients = vec![
        ("client_1", "room_123"),
        ("client_2", "room_123"),
        ("client_3", "room_123"),
    ];

    for (i, (client_id, room_id)) in clients.iter().enumerate() {
        let status = if i == 1 { ClientInRoomStatus::Inactive } else { ClientInRoomStatus::Active };
        let mut client_in_room = ClientInRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        client_in_room.status = status;
        repo.create_client_in_room(client_in_room).await.unwrap();
    }

    // Get active clients in room
    let result = repo.get_active_clients_in_room("room_123").await;
    assert!(result.is_ok());
    let active_clients = result.unwrap();
    assert_eq!(active_clients.len(), 2);

    // Get inactive clients in room
    let result = repo.get_clients_by_status("room_123", ClientInRoomStatus::Inactive).await;
    assert!(result.is_ok());
    let inactive_clients = result.unwrap();
    assert_eq!(inactive_clients.len(), 1);
}

#[tokio::test]
async fn test_client_in_room_repository_date_range() {
    let repo = MockClientInRoomRepository::new();
    
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in room
    repo.create_client_in_room(client_in_room).await.unwrap();

    // Get clients by date range
    let start_date = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(1);

    let result = repo.get_clients_in_room_by_date_range("room_456", start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].client_id, "client_123");

    // Get clients outside date range
    let start_date = chrono::Utc::now() + chrono::Duration::hours(2);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(3);

    let result = repo.get_clients_in_room_by_date_range("room_456", start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 0);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_create() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    let result = repo.create_client_in_terminated_room(client_in_terminated_room.clone()).await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.client_id, "client_123");
    assert_eq!(created.room_id, "room_456");
    assert_eq!(created.final_status, ClientTerminationStatus::Disconnected);
    assert_eq!(created.terminated_by, "admin");
    assert_eq!(created.termination_reason, "Room was terminated");
    assert_eq!(created.metadata, serde_json::json!({}));
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_duplicate_creation() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // First creation should succeed
    let result1 = repo.create_client_in_terminated_room(client_in_terminated_room.clone()).await;
    assert!(result1.is_ok());

    // Second creation with same id should fail
    let result2 = repo.create_client_in_terminated_room(client_in_terminated_room).await;
    assert!(result2.is_err());

    if let Err(error) = result2 {
        match error {
            signal_manager_service::database::DatabaseError::Validation(msg) => {
                assert!(msg.contains("already exists"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_get() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in terminated room
    let created_client = repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();

    // Get existing client in terminated room
    let result = repo.get_client_in_terminated_room(&created_client.id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Get non-existing client in terminated room
    let result = repo.get_client_in_terminated_room("non_existent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_get_from_terminated_room() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    // Create multiple clients from the same terminated room
    let clients = vec![
        ("client_1", "room_123", "Normal"),
        ("client_2", "room_123", "Abnormal"),
        ("client_3", "room_456", "Normal"),
    ];

    for (client_id, room_id, status_str) in clients {
        let status = match status_str {
            "Abnormal" => ClientTerminationStatus::Kicked,
            _ => ClientTerminationStatus::Disconnected,
        };
        let client_in_terminated_room = ClientInTerminatedRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            Utc::now(),
            "Room was terminated".to_string(),
            "admin".to_string(),
            status,
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();
    }

    // Get clients from room_123
    let result = repo.get_clients_from_terminated_room("room_123").await;
    assert!(result.is_ok());
    let clients_from_room = result.unwrap();
    assert_eq!(clients_from_room.len(), 2);

    // Get clients from room_456
    let result = repo.get_clients_from_terminated_room("room_456").await;
    assert!(result.is_ok());
    let clients_from_room = result.unwrap();
    assert_eq!(clients_from_room.len(), 1);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_list_all() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    // Create multiple clients in terminated rooms
    let clients = vec![
        ("client_1", "room_123"),
        ("client_2", "room_123"),
        ("client_3", "room_456"),
    ];

    for (client_id, room_id) in clients {
        let client_in_terminated_room = ClientInTerminatedRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            Utc::now(),
            "Room was terminated".to_string(),
            "admin".to_string(),
            ClientTerminationStatus::Disconnected,
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();
    }

    // List all clients in terminated rooms
    let result = repo.list_clients_in_terminated_rooms().await;
    assert!(result.is_ok());
    let all_clients = result.unwrap();
    assert_eq!(all_clients.len(), 3);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_update() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in terminated room
    repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();

    // Update client in terminated room
    let updated_client = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was forcefully terminated".to_string(),
        "system".to_string(),
        ClientTerminationStatus::Kicked,
        vec!["websocket".to_string()],
        Some(serde_json::json!({"updated": true})),
    );

    let result = repo.update_client_in_terminated_room("client_123_room_456", updated_client.clone()).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.final_status, ClientTerminationStatus::Kicked);
    assert_eq!(updated.terminated_by, "system");
    assert_eq!(updated.termination_reason, "Room was forcefully terminated");
    assert_eq!(updated.metadata, serde_json::json!({"updated": true}));
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_get_by_termination_status() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    // Create clients with different termination statuses
    let clients = vec![
        ("client_1", "room_123", "Normal"),
        ("client_2", "room_123", "Abnormal"),
        ("client_3", "room_456", "Normal"),
    ];

    for (client_id, room_id, status_str) in clients {
        let status = match status_str {
            "Abnormal" => ClientTerminationStatus::Kicked,
            _ => ClientTerminationStatus::Disconnected,
        };
        let client_in_terminated_room = ClientInTerminatedRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            Utc::now(),
            "Room was terminated".to_string(),
            "admin".to_string(),
            status,
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();
    }

    // Get clients with normal termination status
    let result = repo.get_clients_by_termination_status(ClientTerminationStatus::Disconnected).await;
    assert!(result.is_ok());
    let normal_clients = result.unwrap();
    assert_eq!(normal_clients.len(), 2);

    // Get clients with abnormal termination status
    let result = repo.get_clients_by_termination_status(ClientTerminationStatus::Kicked).await;
    assert!(result.is_ok());
    let abnormal_clients = result.unwrap();
    assert_eq!(abnormal_clients.len(), 1);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_get_terminated_by() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    // Create clients terminated by different users
    let clients = vec![
        ("client_1", "room_123", "admin"),
        ("client_2", "room_123", "system"),
        ("client_3", "room_456", "admin"),
    ];

    for (client_id, room_id, terminated_by) in clients {
        let client_in_terminated_room = ClientInTerminatedRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            Utc::now(),
            "Room was terminated".to_string(),
            terminated_by.to_string(),
            if terminated_by == "Abnormal" { ClientTerminationStatus::Kicked } else { ClientTerminationStatus::Disconnected },
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();
    }

    // Get clients terminated by admin
    let result = repo.get_clients_terminated_by("admin").await;
    assert!(result.is_ok());
    let admin_terminated = result.unwrap();
    assert_eq!(admin_terminated.len(), 2);

    // Get clients terminated by system
    let result = repo.get_clients_terminated_by("system").await;
    assert!(result.is_ok());
    let system_terminated = result.unwrap();
    assert_eq!(system_terminated.len(), 1);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_get_by_termination_reason() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    // Create clients with different termination reasons
    let clients = vec![
        ("client_1", "room_123", "Room was inactive"),
        ("client_2", "room_123", "Room was inactive"),
        ("client_3", "room_456", "Room was forcefully terminated"),
    ];

    for (client_id, room_id, reason) in clients {
        let client_in_terminated_room = ClientInTerminatedRoom::new(
            client_id.to_string(),
            room_id.to_string(),
            Utc::now(),
            reason.to_string(),
            "admin".to_string(),
            if reason == "Room was forcefully terminated" { ClientTerminationStatus::Kicked } else { ClientTerminationStatus::Disconnected },
            vec!["websocket".to_string()],
            Some(serde_json::json!({})),
        );
        repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();
    }

    // Get clients terminated due to inactivity
    let result = repo.get_clients_by_termination_reason("Room was inactive").await;
    assert!(result.is_ok());
    let inactive_clients = result.unwrap();
    assert_eq!(inactive_clients.len(), 2);

    // Get clients terminated forcefully
    let result = repo.get_clients_by_termination_reason("Room was forcefully terminated").await;
    assert!(result.is_ok());
    let forceful_clients = result.unwrap();
    assert_eq!(forceful_clients.len(), 1);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_was_in_terminated_room() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Check non-existing client in terminated room
    let result = repo.client_was_in_terminated_room("client_123", "room_456").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Create client in terminated room
    repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();

    // Check existing client in terminated room
    let result = repo.client_was_in_terminated_room("client_123", "room_456").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_date_range() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in terminated room
    repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();

    // Get clients by date range
    let start_date = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(1);

    let result = repo.get_clients_in_terminated_rooms_by_date_range(start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].client_id, "client_123");

    // Get clients outside date range
    let start_date = chrono::Utc::now() + chrono::Duration::hours(2);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(3);

    let result = repo.get_clients_in_terminated_rooms_by_date_range(start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 0);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_from_room_date_range() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in terminated room
    repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();

    // Get clients from specific room by date range
    let start_date = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(1);

    let result = repo.get_clients_from_terminated_room_by_date_range("room_456", start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].client_id, "client_123");

    // Get clients from different room by date range
    let result = repo.get_clients_from_terminated_room_by_date_range("room_789", start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 0);
}

#[tokio::test]
async fn test_client_in_terminated_room_repository_terminated_between() {
    let repo = MockClientInTerminatedRoomRepository::new();
    
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    // Create client in terminated room
    repo.create_client_in_terminated_room(client_in_terminated_room).await.unwrap();

    // Get clients terminated between dates
    let start_date = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(1);

    let result = repo.get_clients_terminated_between(start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].client_id, "client_123");

    // Get clients terminated outside date range
    let start_date = chrono::Utc::now() + chrono::Duration::hours(2);
    let end_date = chrono::Utc::now() + chrono::Duration::hours(3);

    let result = repo.get_clients_terminated_between(start_date, end_date).await;
    assert!(result.is_ok());

    let clients = result.unwrap();
    assert_eq!(clients.len(), 0);
}

#[tokio::test]
async fn test_repository_factory_client_in_room() {
    let factory = MockRepositoryFactory;
    
    let result = factory.create_client_in_room_repository().await;
    assert!(result.is_ok());

    let repo = result.unwrap();
    
    // Test that the repository works
    let client_in_room = ClientInRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    let result = repo.create_client_in_room(client_in_room).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_repository_factory_client_in_terminated_room() {
    let factory = MockRepositoryFactory;
    
    let result = factory.create_client_in_terminated_room_repository().await;
    assert!(result.is_ok());

    let repo = result.unwrap();
    
    // Test that the repository works
    let client_in_terminated_room = ClientInTerminatedRoom::new(
        "client_123".to_string(),
        "room_456".to_string(),
        Utc::now(),
        "Room was terminated".to_string(),
        "admin".to_string(),
        ClientTerminationStatus::Disconnected,
        vec!["websocket".to_string()],
        Some(serde_json::json!({})),
    );

    let result = repo.create_client_in_terminated_room(client_in_terminated_room).await;
    assert!(result.is_ok());
} 