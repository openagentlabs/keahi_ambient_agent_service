use async_trait::async_trait;
use crate::database::{DatabaseResult, TerminatedRoom, TerminationPayload};

/// Repository trait for terminated room database operations
/// This defines the interface that any terminated room database implementation must follow
#[async_trait]
pub trait TerminatedRoomRepository: Send + Sync {
    /// Create a new terminated room record
    async fn create_terminated_room(&self, payload: TerminationPayload) -> DatabaseResult<TerminatedRoom>;
    
    /// Get a terminated room by room ID
    async fn get_terminated_room(&self, room_id: &str) -> DatabaseResult<Option<TerminatedRoom>>;
    
    /// List all terminated rooms
    async fn list_terminated_rooms(&self, limit: Option<usize>) -> DatabaseResult<Vec<TerminatedRoom>>;
    
    /// Check if a room was terminated
    async fn room_was_terminated(&self, room_id: &str) -> DatabaseResult<bool>;
    
    /// Get terminated rooms by date range
    async fn get_terminated_rooms_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> DatabaseResult<Vec<TerminatedRoom>>;
} 