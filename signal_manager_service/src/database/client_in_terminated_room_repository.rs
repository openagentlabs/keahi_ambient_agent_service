use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::database::models::{ClientInTerminatedRoom, ClientTerminationStatus};
use crate::database::error::DatabaseError;

/// Repository trait for managing clients in terminated rooms
#[async_trait]
pub trait ClientInTerminatedRoomRepository: Send + Sync {
    /// Create a new client in terminated room record
    async fn create_client_in_terminated_room(&self, client_in_terminated_room: ClientInTerminatedRoom) -> Result<ClientInTerminatedRoom, DatabaseError>;

    /// Get a client in terminated room by ID
    async fn get_client_in_terminated_room(&self, id: &str) -> Result<Option<ClientInTerminatedRoom>, DatabaseError>;

    /// Get all clients from a specific terminated room
    async fn get_clients_from_terminated_room(&self, room_id: &str) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Get all clients in terminated rooms
    async fn list_clients_in_terminated_rooms(&self) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Update client in terminated room
    async fn update_client_in_terminated_room(&self, id: &str, client_in_terminated_room: ClientInTerminatedRoom) -> Result<ClientInTerminatedRoom, DatabaseError>;

    /// Get clients by termination status
    async fn get_clients_by_termination_status(&self, status: ClientTerminationStatus) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Get clients terminated by specific user
    async fn get_clients_terminated_by(&self, terminated_by: &str) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Get clients by termination reason
    async fn get_clients_by_termination_reason(&self, reason: &str) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Check if client was in terminated room
    async fn client_was_in_terminated_room(&self, client_id: &str, room_id: &str) -> Result<bool, DatabaseError>;

    /// Get clients in terminated rooms by date range
    async fn get_clients_in_terminated_rooms_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Get clients from terminated room by date range
    async fn get_clients_from_terminated_room_by_date_range(
        &self,
        room_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;

    /// Get clients terminated between specific dates
    async fn get_clients_terminated_between(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ClientInTerminatedRoom>, DatabaseError>;
} 