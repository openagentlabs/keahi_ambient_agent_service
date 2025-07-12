use async_trait::async_trait;
use crate::database::{DatabaseResult, RegisteredClient, RegistrationPayload};

/// Repository trait for client database operations
/// This defines the interface that any client database implementation must follow
#[async_trait]
pub trait ClientRepository: Send + Sync {
    /// Create a new client registration
    async fn create_client(&self, payload: RegistrationPayload) -> DatabaseResult<RegisteredClient>;
    
    /// Get a client by ID
    async fn get_client(&self, client_id: &str) -> DatabaseResult<Option<RegisteredClient>>;
    
    /// Get a client by authentication token
    async fn get_client_by_token(&self, auth_token: &str) -> DatabaseResult<Option<RegisteredClient>>;
    
    /// Update a client's information
    async fn update_client(&self, client: RegisteredClient) -> DatabaseResult<RegisteredClient>;
    
    /// Delete a client
    async fn delete_client(&self, client_id: &str) -> DatabaseResult<bool>;
    
    /// List all clients
    async fn list_clients(&self, limit: Option<usize>) -> DatabaseResult<Vec<RegisteredClient>>;
    
    /// Update client's last seen timestamp
    async fn update_last_seen(&self, client_id: &str) -> DatabaseResult<bool>;
    
    /// Check if a client exists
    async fn client_exists(&self, client_id: &str) -> DatabaseResult<bool>;
    
    /// Validate client authentication
    async fn validate_auth(&self, client_id: &str, auth_token: &str) -> DatabaseResult<bool>;
} 