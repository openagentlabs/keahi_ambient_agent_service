use async_trait::async_trait;
use crate::database::models::{WebRTCClient, WebRTCClientRegistrationPayload, WebRTCClientStatus, ClientRole};
use crate::database::error::DatabaseError;

/// Repository trait for WebRTC client operations
#[async_trait]
pub trait WebRTCClientRepository {
    /// Register a new WebRTC client
    async fn register_client(&self, payload: WebRTCClientRegistrationPayload) -> Result<WebRTCClient, DatabaseError>;
    
    /// Get a client by its ID
    async fn get_client_by_id(&self, client_id: &str) -> Result<Option<WebRTCClient>, DatabaseError>;
    
    /// Get clients by room ID
    async fn get_clients_by_room_id(&self, room_id: &str) -> Result<Vec<WebRTCClient>, DatabaseError>;
    
    /// Get clients by role in a room
    async fn get_clients_by_role(&self, room_id: &str, role: ClientRole) -> Result<Vec<WebRTCClient>, DatabaseError>;
    
    /// Update client status
    async fn update_client_status(&self, client_id: &str, status: WebRTCClientStatus) -> Result<(), DatabaseError>;
    
    /// Set session ID for client
    async fn set_session_id(&self, client_id: &str, session_id: &str) -> Result<(), DatabaseError>;
    
    /// Get client by session ID
    async fn get_client_by_session_id(&self, session_id: &str) -> Result<Option<WebRTCClient>, DatabaseError>;
    
    /// Get all active clients
    async fn get_active_clients(&self) -> Result<Vec<WebRTCClient>, DatabaseError>;
    
    /// Get active clients in a room
    async fn get_active_clients_in_room(&self, room_id: &str) -> Result<Vec<WebRTCClient>, DatabaseError>;
    
    /// Disconnect a client
    async fn disconnect_client(&self, client_id: &str, reason: &str) -> Result<(), DatabaseError>;
    
    /// Remove a client from a room
    async fn remove_client_from_room(&self, client_id: &str, room_id: &str) -> Result<(), DatabaseError>;
    
    /// Delete a client
    async fn delete_client(&self, client_id: &str) -> Result<(), DatabaseError>;
    
    /// Get client count
    async fn get_client_count(&self) -> Result<usize, DatabaseError>;
    
    /// Get client count in a room
    async fn get_client_count_in_room(&self, room_id: &str) -> Result<usize, DatabaseError>;
} 