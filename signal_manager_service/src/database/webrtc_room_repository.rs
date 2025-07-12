use async_trait::async_trait;
use crate::database::models::{WebRTCRoom, WebRTCRoomCreationPayload, WebRTCRoomStatus};
use crate::database::error::DatabaseError;

/// Repository trait for WebRTC room operations
#[async_trait]
pub trait WebRTCRoomRepository {
    /// Create a new WebRTC room
    async fn create_room(&self, payload: WebRTCRoomCreationPayload) -> Result<WebRTCRoom, DatabaseError>;
    
    /// Get a room by its ID
    async fn get_room_by_id(&self, room_id: &str) -> Result<Option<WebRTCRoom>, DatabaseError>;
    
    /// Get a room by its UUID
    async fn get_room_by_uuid(&self, room_uuid: &str) -> Result<Option<WebRTCRoom>, DatabaseError>;
    
    /// Update room status
    async fn update_room_status(&self, room_id: &str, status: WebRTCRoomStatus) -> Result<(), DatabaseError>;
    
    /// Set sender client ID
    async fn set_sender_client_id(&self, room_id: &str, client_id: &str) -> Result<(), DatabaseError>;
    
    /// Set receiver client ID
    async fn set_receiver_client_id(&self, room_id: &str, client_id: &str) -> Result<(), DatabaseError>;
    
    /// Set session ID
    async fn set_session_id(&self, room_id: &str, session_id: &str) -> Result<(), DatabaseError>;
    
    /// Get all active rooms
    async fn get_active_rooms(&self) -> Result<Vec<WebRTCRoom>, DatabaseError>;
    
    /// Get rooms by client ID
    async fn get_rooms_by_client_id(&self, client_id: &str) -> Result<Vec<WebRTCRoom>, DatabaseError>;
    
    /// Terminate a room
    async fn terminate_room(&self, room_id: &str, reason: &str) -> Result<(), DatabaseError>;
    
    /// Delete a room
    async fn delete_room(&self, room_id: &str) -> Result<(), DatabaseError>;
    
    /// Get room count
    async fn get_room_count(&self) -> Result<usize, DatabaseError>;
} 