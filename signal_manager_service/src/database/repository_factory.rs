use async_trait::async_trait;
use std::sync::Arc;
use crate::database::{DatabaseResult, ClientRepository, TerminatedRoomRepository, RoomCreatedRepository, ClientInRoomRepository, ClientInTerminatedRoomRepository, WebRTCRoomRepository, WebRTCClientRepository};

/// Repository factory trait for creating repository instances
/// This defines the interface for creating different types of repositories
#[async_trait]
pub trait RepositoryFactory: Send + Sync {
    /// Create a new client repository instance
    async fn create_client_repository(&self) -> DatabaseResult<Arc<dyn ClientRepository + Send + Sync>>;
    
    /// Create a new terminated room repository instance
    async fn create_terminated_room_repository(&self) -> DatabaseResult<Arc<dyn TerminatedRoomRepository + Send + Sync>>;
    
    /// Create a new room created repository instance
    async fn create_room_created_repository(&self) -> DatabaseResult<Arc<dyn RoomCreatedRepository + Send + Sync>>;

    /// Create a new client in room repository instance
    async fn create_client_in_room_repository(&self) -> DatabaseResult<Arc<dyn ClientInRoomRepository + Send + Sync>>;

    /// Create a new client in terminated room repository instance
    async fn create_client_in_terminated_room_repository(&self) -> DatabaseResult<Arc<dyn ClientInTerminatedRoomRepository + Send + Sync>>;

    /// Create a new WebRTC room repository instance
    async fn create_webrtc_room_repository(&self) -> DatabaseResult<Arc<dyn WebRTCRoomRepository + Send + Sync>>;

    /// Create a new WebRTC client repository instance
    async fn create_webrtc_client_repository(&self) -> DatabaseResult<Arc<dyn WebRTCClientRepository + Send + Sync>>;
} 