use async_trait::async_trait;
use crate::database::{DatabaseResult, RoomCreated, RoomCreationPayload};

/// Repository trait for room creation database operations
/// This defines the interface that any room creation database implementation must follow
#[async_trait]
pub trait RoomCreatedRepository: Send + Sync {
    /// Create a new room creation record
    async fn create_room_created(&self, payload: RoomCreationPayload) -> DatabaseResult<RoomCreated>;
    
    /// Get a room creation record by room UUID
    async fn get_room_created(&self, room_uuid: &str) -> DatabaseResult<Option<RoomCreated>>;
    
    /// List all room creation records
    async fn list_rooms_created(&self, limit: Option<usize>) -> DatabaseResult<Vec<RoomCreated>>;
    
    /// Check if a room was created
    async fn room_was_created(&self, room_uuid: &str) -> DatabaseResult<bool>;
    
    /// Get room creation records by date range
    async fn get_rooms_created_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<RoomCreated>>;
} 