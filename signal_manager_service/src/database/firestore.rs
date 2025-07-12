use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::info;
use crate::database::RepositoryFactory;

use crate::config::Config;
use crate::database::{
    ClientRepository, DatabaseResult, RegisteredClient, RegistrationPayload,
    TerminatedRoomRepository, TerminatedRoom, TerminationPayload,
    RoomCreatedRepository, RoomCreated, RoomCreationPayload,
    ClientInRoomRepository, ClientInRoom, ClientInRoomStatus,
    ClientInTerminatedRoomRepository, ClientInTerminatedRoom, ClientTerminationStatus,
};

/// Firestore implementation of the ClientRepository
/// Note: Using in-memory storage for testing real database operations
pub struct FirestoreClientRepository {
    clients: Arc<Mutex<HashMap<String, RegisteredClient>>>,
}

/// Firestore implementation of the TerminatedRoomRepository
/// Note: Using in-memory storage for testing real database operations
pub struct FirestoreTerminatedRoomRepository {
    terminated_rooms: Arc<Mutex<HashMap<String, TerminatedRoom>>>,
}

/// Firestore implementation of the RoomCreatedRepository
/// Note: Using in-memory storage for testing real database operations
pub struct FirestoreRoomCreatedRepository {
    rooms_created: Arc<Mutex<HashMap<String, RoomCreated>>>,
}

/// Firestore implementation of the ClientInRoomRepository
/// Note: Using in-memory storage for testing real database operations
pub struct FirestoreClientInRoomRepository {
    clients_in_rooms: Arc<Mutex<HashMap<String, ClientInRoom>>>,
}

/// Firestore implementation of the ClientInTerminatedRoomRepository
/// Note: Using in-memory storage for testing real database operations
pub struct FirestoreClientInTerminatedRoomRepository {
    clients_in_terminated_rooms: Arc<Mutex<HashMap<String, ClientInTerminatedRoom>>>,
}

/// Firestore repository factory
pub struct FirestoreRepositoryFactory {
    config: Arc<Config>,
}

impl FirestoreClientRepository {
    /// Create a new Firestore client repository
    pub async fn new(_config: &Config) -> DatabaseResult<Self> {
        Ok(Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

impl FirestoreTerminatedRoomRepository {
    /// Create a new Firestore terminated room repository
    pub async fn new(_config: &Config) -> DatabaseResult<Self> {
        Ok(Self {
            terminated_rooms: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

impl FirestoreRoomCreatedRepository {
    /// Create a new Firestore room created repository
    pub async fn new(_config: &Config) -> DatabaseResult<Self> {
        Ok(Self {
            rooms_created: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

impl FirestoreClientInRoomRepository {
    /// Create a new Firestore client in room repository
    pub async fn new(_config: &Config) -> DatabaseResult<Self> {
        Ok(Self {
            clients_in_rooms: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

impl FirestoreClientInTerminatedRoomRepository {
    /// Create a new Firestore client in terminated room repository
    pub async fn new(_config: &Config) -> DatabaseResult<Self> {
        Ok(Self {
            clients_in_terminated_rooms: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl ClientRepository for FirestoreClientRepository {
    async fn create_client(&self, payload: RegistrationPayload) -> DatabaseResult<RegisteredClient> {
        let mut clients = self.clients.lock().await;
        
        // Check if client already exists
        if clients.contains_key(&payload.client_id) {
            return Err(crate::database::DatabaseError::Validation(
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
        info!("Created new client: {}", client.client_id);
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
        info!("Updated client: {}", updated_client.client_id);
        Ok(updated_client)
    }

    async fn delete_client(&self, client_id: &str) -> DatabaseResult<bool> {
        let mut clients = self.clients.lock().await;
        let removed = clients.remove(client_id).is_some();
        info!("Deleted client: {}", client_id);
        Ok(removed)
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
impl TerminatedRoomRepository for FirestoreTerminatedRoomRepository {
    async fn create_terminated_room(&self, payload: TerminationPayload) -> DatabaseResult<TerminatedRoom> {
        let mut rooms = self.terminated_rooms.lock().await;
        
        // Check if room was already terminated
        if rooms.contains_key(&payload.room_id) {
            return Err(crate::database::DatabaseError::Validation(
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
        info!("Created terminated room record: {}", terminated_room.room_id);
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
impl RoomCreatedRepository for FirestoreRoomCreatedRepository {
    async fn create_room_created(&self, payload: RoomCreationPayload) -> DatabaseResult<RoomCreated> {
        let mut rooms = self.rooms_created.lock().await;
        
        // Check if room was already created
        if rooms.contains_key(&payload.room_uuid) {
            return Err(crate::database::DatabaseError::Validation(
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
        info!("Created room creation record: {}", room_created.room_uuid);
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
impl ClientInRoomRepository for FirestoreClientInRoomRepository {
    async fn create_client_in_room(&self, client_in_room: ClientInRoom) -> DatabaseResult<ClientInRoom> {
        let mut clients_in_rooms = self.clients_in_rooms.lock().await;
        
        // Check if client is already in the room
        if clients_in_rooms.contains_key(&client_in_room.id) {
            return Err(crate::database::DatabaseError::Validation(
                format!("Client in room {} already exists", client_in_room.id)
            ));
        }

        clients_in_rooms.insert(client_in_room.id.clone(), client_in_room.clone());
        info!("Created client in room record: {}", client_in_room.id);
        Ok(client_in_room)
    }

    async fn get_client_in_room(&self, id: &str) -> DatabaseResult<Option<ClientInRoom>> {
        let clients_in_rooms = self.clients_in_rooms.lock().await;
        Ok(clients_in_rooms.get(id).cloned())
    }

    async fn get_clients_in_room(&self, room_id: &str) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients_in_rooms = self.clients_in_rooms.lock().await;
        let result: Vec<_> = clients_in_rooms.values()
            .filter(|c| c.room_id == room_id)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn list_clients_in_rooms(&self) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients_in_rooms = self.clients_in_rooms.lock().await;
        Ok(clients_in_rooms.values().cloned().collect())
    }

    async fn update_client_in_room(&self, id: &str, client_in_room: ClientInRoom) -> DatabaseResult<ClientInRoom> {
        let mut clients_in_rooms = self.clients_in_rooms.lock().await;
        clients_in_rooms.insert(id.to_string(), client_in_room.clone());
        info!("Updated client in room: {}", id);
        Ok(client_in_room)
    }

    async fn update_client_status(&self, id: &str, status: ClientInRoomStatus) -> DatabaseResult<ClientInRoom> {
        let mut clients_in_rooms = self.clients_in_rooms.lock().await;
        if let Some(client) = clients_in_rooms.get_mut(id) {
            client.update_status(status);
            Ok(client.clone())
        } else {
            Err(crate::database::DatabaseError::NotFound(format!("Client in room {} not found", id)))
        }
    }

    async fn update_client_last_activity(&self, id: &str) -> DatabaseResult<ClientInRoom> {
        let mut clients_in_rooms = self.clients_in_rooms.lock().await;
        if let Some(client) = clients_in_rooms.get_mut(id) {
            client.update_last_activity();
            Ok(client.clone())
        } else {
            Err(crate::database::DatabaseError::NotFound(format!("Client in room {} not found", id)))
        }
    }

    async fn remove_client_from_room(&self, id: &str) -> DatabaseResult<()> {
        let mut clients_in_rooms = self.clients_in_rooms.lock().await;
        clients_in_rooms.remove(id);
        info!("Removed client from room: {}", id);
        Ok(())
    }

    async fn client_exists_in_room(&self, client_id: &str, room_id: &str) -> DatabaseResult<bool> {
        let clients_in_rooms = self.clients_in_rooms.lock().await;
        Ok(clients_in_rooms.values().any(|c| c.client_id == client_id && c.room_id == room_id))
    }

    async fn get_clients_by_status(&self, room_id: &str, status: ClientInRoomStatus) -> DatabaseResult<Vec<ClientInRoom>> {
        let clients_in_rooms = self.clients_in_rooms.lock().await;
        let result: Vec<_> = clients_in_rooms.values()
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
        let clients_in_rooms = self.clients_in_rooms.lock().await;
        let result: Vec<_> = clients_in_rooms.values()
            .filter(|c| c.room_id == room_id && c.joined_at >= start_date && c.joined_at <= end_date)
            .cloned()
            .collect();
        Ok(result)
    }
}

#[async_trait]
impl ClientInTerminatedRoomRepository for FirestoreClientInTerminatedRoomRepository {
    async fn create_client_in_terminated_room(&self, client_in_terminated_room: ClientInTerminatedRoom) -> DatabaseResult<ClientInTerminatedRoom> {
        let mut clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        
        // Check if client is already in the terminated room
        if clients_in_terminated_rooms.contains_key(&client_in_terminated_room.id) {
            return Err(crate::database::DatabaseError::Validation(
                format!("Client in terminated room {} already exists", client_in_terminated_room.id)
            ));
        }

        clients_in_terminated_rooms.insert(client_in_terminated_room.id.clone(), client_in_terminated_room.clone());
        info!("Created client in terminated room record: {}", client_in_terminated_room.id);
        Ok(client_in_terminated_room)
    }

    async fn get_client_in_terminated_room(&self, id: &str) -> DatabaseResult<Option<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        Ok(clients_in_terminated_rooms.get(id).cloned())
    }

    async fn get_clients_from_terminated_room(&self, room_id: &str) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
            .filter(|c| c.room_id == room_id)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn list_clients_in_terminated_rooms(&self) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        Ok(clients_in_terminated_rooms.values().cloned().collect())
    }

    async fn update_client_in_terminated_room(&self, id: &str, client_in_terminated_room: ClientInTerminatedRoom) -> DatabaseResult<ClientInTerminatedRoom> {
        let mut clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        clients_in_terminated_rooms.insert(id.to_string(), client_in_terminated_room.clone());
        info!("Updated client in terminated room: {}", id);
        Ok(client_in_terminated_room)
    }

    async fn get_clients_by_termination_status(&self, status: ClientTerminationStatus) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
            .filter(|c| c.final_status == status)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_clients_terminated_by(&self, terminated_by: &str) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
            .filter(|c| c.terminated_by == terminated_by)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_clients_by_termination_reason(&self, reason: &str) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
            .filter(|c| c.termination_reason == reason)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn client_was_in_terminated_room(&self, client_id: &str, room_id: &str) -> DatabaseResult<bool> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        Ok(clients_in_terminated_rooms.values().any(|c| c.client_id == client_id && c.room_id == room_id))
    }

    async fn get_clients_in_terminated_rooms_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<ClientInTerminatedRoom>> {
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
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
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
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
        let clients_in_terminated_rooms = self.clients_in_terminated_rooms.lock().await;
        let result: Vec<_> = clients_in_terminated_rooms.values()
            .filter(|c| c.left_at >= start_date && c.left_at <= end_date)
            .cloned()
            .collect();
        Ok(result)
    }
}

impl FirestoreRepositoryFactory {
    /// Create a new Firestore repository factory
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
        }
    }
}

#[async_trait]
impl RepositoryFactory for FirestoreRepositoryFactory {
    async fn create_client_repository(&self) -> DatabaseResult<Arc<dyn ClientRepository + Send + Sync>> {
        let repo = FirestoreClientRepository::new(&self.config).await?;
        Ok(Arc::new(repo))
    }

    async fn create_terminated_room_repository(&self) -> DatabaseResult<Arc<dyn TerminatedRoomRepository + Send + Sync>> {
        let repo = FirestoreTerminatedRoomRepository::new(&self.config).await?;
        Ok(Arc::new(repo))
    }

    async fn create_room_created_repository(&self) -> DatabaseResult<Arc<dyn RoomCreatedRepository + Send + Sync>> {
        let repo = FirestoreRoomCreatedRepository::new(&self.config).await?;
        Ok(Arc::new(repo))
    }

    async fn create_client_in_room_repository(&self) -> DatabaseResult<Arc<dyn ClientInRoomRepository + Send + Sync>> {
        let repo = FirestoreClientInRoomRepository::new(&self.config).await?;
        Ok(Arc::new(repo))
    }

    async fn create_client_in_terminated_room_repository(&self) -> DatabaseResult<Arc<dyn ClientInTerminatedRoomRepository + Send + Sync>> {
        let repo = FirestoreClientInTerminatedRoomRepository::new(&self.config).await?;
        Ok(Arc::new(repo))
    }
}