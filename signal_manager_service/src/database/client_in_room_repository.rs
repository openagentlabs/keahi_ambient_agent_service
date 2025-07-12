use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::database::models::{ClientInRoom, ClientInRoomStatus};
use crate::database::error::DatabaseError;

/// Repository trait for managing clients in rooms
#[async_trait]
pub trait ClientInRoomRepository: Send + Sync {
    /// Create a new client in room record
    async fn create_client_in_room(&self, client_in_room: ClientInRoom) -> Result<ClientInRoom, DatabaseError>;

    /// Get a client in room by ID
    async fn get_client_in_room(&self, id: &str) -> Result<Option<ClientInRoom>, DatabaseError>;

    /// Get all clients in a specific room
    async fn get_clients_in_room(&self, room_id: &str) -> Result<Vec<ClientInRoom>, DatabaseError>;

    /// Get all clients in rooms
    async fn list_clients_in_rooms(&self) -> Result<Vec<ClientInRoom>, DatabaseError>;

    /// Update client in room
    async fn update_client_in_room(&self, id: &str, client_in_room: ClientInRoom) -> Result<ClientInRoom, DatabaseError>;

    /// Update client status in room
    async fn update_client_status(&self, id: &str, status: ClientInRoomStatus) -> Result<ClientInRoom, DatabaseError>;

    /// Update client last activity
    async fn update_client_last_activity(&self, id: &str) -> Result<ClientInRoom, DatabaseError>;

    /// Remove client from room
    async fn remove_client_from_room(&self, id: &str) -> Result<(), DatabaseError>;

    /// Check if client is in room
    async fn client_exists_in_room(&self, client_id: &str, room_id: &str) -> Result<bool, DatabaseError>;

    /// Get clients by status in room
    async fn get_clients_by_status(&self, room_id: &str, status: ClientInRoomStatus) -> Result<Vec<ClientInRoom>, DatabaseError>;

    /// Get active clients in room
    async fn get_active_clients_in_room(&self, room_id: &str) -> Result<Vec<ClientInRoom>, DatabaseError>;

    /// Get clients in room by date range
    async fn get_clients_in_room_by_date_range(
        &self,
        room_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ClientInRoom>, DatabaseError>;
} 