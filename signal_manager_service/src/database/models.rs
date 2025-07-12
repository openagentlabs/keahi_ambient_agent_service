use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a registered client in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredClient {
    /// Unique identifier for the client
    pub id: String,
    /// Client name or identifier
    pub client_id: String,
    /// Authentication token for the client
    pub auth_token: String,
    /// Room identifier that the client is associated with
    pub room_id: Option<String>,
    /// Client capabilities or features
    pub capabilities: Vec<String>,
    /// When the client was registered
    pub registered_at: DateTime<Utc>,
    /// When the client was last seen
    pub last_seen: Option<DateTime<Utc>>,
    /// Client status (active, inactive, etc.)
    pub status: ClientStatus,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// Represents a terminated room in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminatedRoom {
    /// Unique identifier for the terminated room record
    pub id: String,
    /// Room identifier that was terminated
    pub room_id: String,
    /// When the room was terminated
    pub terminated_at: DateTime<Utc>,
    /// When the termination was recorded in the database
    pub termination_recorded_at: DateTime<Utc>,
    /// Room data at the time of termination
    pub room_data: serde_json::Value,
    /// Reason for termination (optional)
    pub termination_reason: Option<String>,
    /// Client ID that initiated the termination (optional)
    pub terminated_by: Option<String>,
    /// Additional metadata about the termination
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// Represents a room creation event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCreated {
    /// Unique identifier for the room creation record
    pub id: String,
    /// Room UUID that was created
    pub room_uuid: String,
    /// When the room was created
    pub created_at: DateTime<Utc>,
    /// Room data at the time of creation
    pub room_data: serde_json::Value,
    /// Client ID that created the room (optional)
    pub created_by: Option<String>,
    /// Additional metadata about the creation
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// Client status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ClientStatus {
    #[default]
    Active,
    Inactive,
    Suspended,
    Pending,
}

/// Registration payload for new clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationPayload {
    pub client_id: String,
    pub auth_token: String,
    pub room_id: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub status: u16,
    pub message: Option<String>,
    pub client_id: String,
    pub session_id: Option<String>,
}

/// Termination payload for rooms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationPayload {
    pub room_id: String,
    pub room_data: serde_json::Value,
    pub termination_reason: Option<String>,
    pub terminated_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Room creation payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCreationPayload {
    pub room_uuid: String,
    pub room_data: serde_json::Value,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Represents a client currently in a room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInRoom {
    /// Unique identifier for the client in room record
    pub id: String,
    /// Client identifier
    pub client_id: String,
    /// Room identifier that the client is in
    pub room_id: String,
    /// When the client joined the room
    pub joined_at: DateTime<Utc>,
    /// When the client was last active
    pub last_activity: DateTime<Utc>,
    /// Current status of the client in the room
    pub status: ClientInRoomStatus,
    /// Client capabilities
    pub capabilities: Vec<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// Represents a client that was in a room but is now in a terminated room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInTerminatedRoom {
    /// Unique identifier for the client in terminated room record
    pub id: String,
    /// Client identifier
    pub client_id: String,
    /// Room identifier that was terminated
    pub room_id: String,
    /// When the client joined the room
    pub joined_at: DateTime<Utc>,
    /// When the client left the room
    pub left_at: DateTime<Utc>,
    /// Reason for termination
    pub termination_reason: String,
    /// Who terminated the room
    pub terminated_by: String,
    /// Final status of the client
    pub final_status: ClientTerminationStatus,
    /// Client capabilities
    pub capabilities: Vec<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// Client status in room enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ClientInRoomStatus {
    #[default]
    Active,
    Inactive,
    Away,
    Busy,
}

/// Client termination status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ClientTerminationStatus {
    #[default]
    Disconnected,
    VoluntaryDisconnect,
    Kicked,
    Banned,
}

/// Client in room payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInRoomPayload {
    pub client_id: String,
    pub room_id: String,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Client in terminated room payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInTerminatedRoomPayload {
    pub client_id: String,
    pub room_id: String,
    pub termination_reason: String,
    pub terminated_by: String,
    pub final_status: ClientTerminationStatus,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

impl RegisteredClient {
    /// Create a new registered client
    pub fn new(
        client_id: String,
        auth_token: String,
        capabilities: Vec<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            auth_token,
            room_id: None,
            capabilities,
            registered_at: Utc::now(),
            last_seen: None,
            status: ClientStatus::Active,
            metadata,
            record_created_at: Utc::now(),
        }
    }

    /// Create a new registered client with room association
    pub fn new_with_room(
        client_id: String,
        auth_token: String,
        room_id: String,
        capabilities: Vec<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            auth_token,
            room_id: Some(room_id),
            capabilities,
            registered_at: Utc::now(),
            last_seen: None,
            status: ClientStatus::Active,
            metadata,
            record_created_at: Utc::now(),
        }
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Some(Utc::now());
    }

    /// Check if the client is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ClientStatus::Active)
    }

    /// Get the room ID if associated
    pub fn get_room_id(&self) -> Option<&str> {
        self.room_id.as_deref()
    }

    /// Associate the client with a room
    pub fn associate_with_room(&mut self, room_id: String) {
        self.room_id = Some(room_id);
    }

    /// Disassociate the client from any room
    pub fn disassociate_from_room(&mut self) {
        self.room_id = None;
    }
}

impl TerminatedRoom {
    /// Create a new terminated room record
    pub fn new(
        room_id: String,
        room_data: serde_json::Value,
        termination_reason: Option<String>,
        terminated_by: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            room_id,
            terminated_at: Utc::now(),
            termination_recorded_at: Utc::now(),
            room_data,
            termination_reason,
            terminated_by,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Create a new terminated room record with custom timestamps
    pub fn new_with_timestamps(
        room_id: String,
        room_data: serde_json::Value,
        terminated_at: DateTime<Utc>,
        termination_reason: Option<String>,
        terminated_by: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            room_id,
            terminated_at,
            termination_recorded_at: Utc::now(),
            room_data,
            termination_reason,
            terminated_by,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Get the room ID
    pub fn get_room_id(&self) -> &str {
        &self.room_id
    }

    /// Get the termination timestamp
    pub fn get_terminated_at(&self) -> DateTime<Utc> {
        self.terminated_at
    }

    /// Get the termination recorded timestamp
    pub fn get_termination_recorded_at(&self) -> DateTime<Utc> {
        self.termination_recorded_at
    }

    /// Get the room data
    pub fn get_room_data(&self) -> &serde_json::Value {
        &self.room_data
    }
}

impl RoomCreated {
    /// Create a new room creation record
    pub fn new(
        room_uuid: String,
        room_data: serde_json::Value,
        created_by: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            room_uuid,
            created_at: Utc::now(),
            room_data,
            created_by,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Get the room UUID
    pub fn get_room_uuid(&self) -> &str {
        &self.room_uuid
    }

    /// Get the creation timestamp
    pub fn get_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get the room data
    pub fn get_room_data(&self) -> &serde_json::Value {
        &self.room_data
    }
}

impl ClientInRoom {
    /// Create a new client in room record
    pub fn new(
        client_id: String,
        room_id: String,
        capabilities: Vec<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            room_id,
            joined_at: Utc::now(),
            last_activity: Utc::now(),
            status: ClientInRoomStatus::Active,
            capabilities,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Update the last activity timestamp
    pub fn update_last_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Update the client status
    pub fn update_status(&mut self, status: ClientInRoomStatus) {
        self.status = status;
    }

    /// Check if the client is active in the room
    pub fn is_active(&self) -> bool {
        matches!(self.status, ClientInRoomStatus::Active)
    }

    /// Get the client ID
    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    /// Get the room ID
    pub fn get_room_id(&self) -> &str {
        &self.room_id
    }

    /// Get the joined timestamp
    pub fn get_joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }

    /// Get the last activity timestamp
    pub fn get_last_activity(&self) -> DateTime<Utc> {
        self.last_activity
    }
}

impl ClientInTerminatedRoom {
    /// Create a new client in terminated room record
    pub fn new(
        client_id: String,
        room_id: String,
        joined_at: DateTime<Utc>,
        termination_reason: String,
        terminated_by: String,
        final_status: ClientTerminationStatus,
        capabilities: Vec<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            room_id,
            joined_at,
            left_at: Utc::now(),
            termination_reason,
            terminated_by,
            final_status,
            capabilities,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Create a new client in terminated room record with custom left timestamp
    pub fn new_with_left_at(
        client_id: String,
        room_id: String,
        joined_at: DateTime<Utc>,
        left_at: DateTime<Utc>,
        termination_reason: String,
        terminated_by: String,
        final_status: ClientTerminationStatus,
        capabilities: Vec<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            room_id,
            joined_at,
            left_at,
            termination_reason,
            terminated_by,
            final_status,
            capabilities,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Builder for creating ClientInTerminatedRoom
    pub fn builder() -> ClientInTerminatedRoomBuilder {
        ClientInTerminatedRoomBuilder::default()
    }
}

/// Builder for ClientInTerminatedRoom
#[derive(Default)]
pub struct ClientInTerminatedRoomBuilder {
    client_id: Option<String>,
    room_id: Option<String>,
    joined_at: Option<DateTime<Utc>>,
    left_at: Option<DateTime<Utc>>,
    termination_reason: Option<String>,
    terminated_by: Option<String>,
    final_status: Option<ClientTerminationStatus>,
    capabilities: Vec<String>,
    metadata: Option<serde_json::Value>,
}

impl ClientInTerminatedRoomBuilder {
    pub fn client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn room_id(mut self, room_id: String) -> Self {
        self.room_id = Some(room_id);
        self
    }

    pub fn joined_at(mut self, joined_at: DateTime<Utc>) -> Self {
        self.joined_at = Some(joined_at);
        self
    }

    pub fn left_at(mut self, left_at: DateTime<Utc>) -> Self {
        self.left_at = Some(left_at);
        self
    }

    pub fn termination_reason(mut self, termination_reason: String) -> Self {
        self.termination_reason = Some(termination_reason);
        self
    }

    pub fn terminated_by(mut self, terminated_by: String) -> Self {
        self.terminated_by = Some(terminated_by);
        self
    }

    pub fn final_status(mut self, final_status: ClientTerminationStatus) -> Self {
        self.final_status = Some(final_status);
        self
    }

    pub fn capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> Result<ClientInTerminatedRoom, String> {
        let client_id = self.client_id.ok_or("client_id is required")?;
        let room_id = self.room_id.ok_or("room_id is required")?;
        let joined_at = self.joined_at.ok_or("joined_at is required")?;
        let termination_reason = self.termination_reason.ok_or("termination_reason is required")?;
        let terminated_by = self.terminated_by.ok_or("terminated_by is required")?;
        let final_status = self.final_status.ok_or("final_status is required")?;

        Ok(ClientInTerminatedRoom {
            id: Uuid::new_v4().to_string(),
            client_id,
            room_id,
            joined_at,
            left_at: self.left_at.unwrap_or_else(Utc::now),
            termination_reason,
            terminated_by,
            final_status,
            capabilities: self.capabilities,
            metadata: self.metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        })
    }
}

impl ClientInTerminatedRoom {
    /// Get the client ID
    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    /// Get the room ID
    pub fn get_room_id(&self) -> &str {
        &self.room_id
    }

    /// Get the joined timestamp
    pub fn get_joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }

    /// Get the left timestamp
    pub fn get_left_at(&self) -> DateTime<Utc> {
        self.left_at
    }

    /// Get the termination reason
    pub fn get_termination_reason(&self) -> &str {
        &self.termination_reason
    }

    /// Get who terminated the room
    pub fn get_terminated_by(&self) -> &str {
        &self.terminated_by
    }

    /// Get the final status
    pub fn get_final_status(&self) -> ClientTerminationStatus {
        self.final_status.clone()
    }

    /// Calculate session duration
    pub fn get_session_duration(&self) -> std::time::Duration {
        self.left_at.signed_duration_since(self.joined_at).to_std().unwrap_or_default()
    }
}




/// WebRTC room information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoom {
    /// Unique room UUID
    pub id: String,
    /// Room UUID
    pub room_id: String,
    /// Cloudflare app ID
    pub app_id: String,
    /// Room creation timestamp
    pub created_at: DateTime<Utc>,
    /// Room status
    pub status: WebRTCRoomStatus,
    /// Sender client ID
    pub sender_client_id: Option<String>,
    /// Receiver client ID
    pub receiver_client_id: Option<String>,
    /// Cloudflare session ID
    pub session_id: Option<String>,
    /// Room metadata
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// WebRTC room status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum WebRTCRoomStatus {
    Active,
    Inactive,
    Terminated,
    #[default]
    Pending,
}

/// Client role in WebRTC room
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientRole {
    Sender,
    Receiver,
}

/// WebRTC client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCClient {
    /// Unique identifier for the WebRTC client record
    pub id: String,
    /// Client ID
    pub client_id: String,
    /// Room ID the client is in
    pub room_id: String,
    /// Client role (sender/receiver)
    pub role: ClientRole,
    /// Session ID if active
    pub session_id: Option<String>,
    /// When the client joined
    pub joined_at: DateTime<Utc>,
    /// Client status
    pub status: WebRTCClientStatus,
    /// Client metadata
    pub metadata: serde_json::Value,
    /// When the record was created in the database
    pub record_created_at: DateTime<Utc>,
}

/// WebRTC client status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum WebRTCClientStatus {
    Active,
    Inactive,
    Disconnected,
    #[default]
    Pending,
}

/// WebRTC room creation payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomCreationPayload {
    pub room_id: String,
    pub app_id: String,
    pub sender_client_id: Option<String>,
    pub receiver_client_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// WebRTC client registration payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCClientRegistrationPayload {
    pub client_id: String,
    pub room_id: String,
    pub role: ClientRole,
    pub session_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl WebRTCRoom {
    /// Create a new WebRTC room
    pub fn new(
        room_id: String,
        app_id: String,
        sender_client_id: Option<String>,
        receiver_client_id: Option<String>,
        session_id: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            room_id,
            app_id,
            created_at: Utc::now(),
            status: WebRTCRoomStatus::Pending,
            sender_client_id,
            receiver_client_id,
            session_id,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Get the room ID
    pub fn get_room_id(&self) -> &str {
        &self.room_id
    }

    /// Get the app ID
    pub fn get_app_id(&self) -> &str {
        &self.app_id
    }

    /// Get the session ID
    pub fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Update the room status
    pub fn update_status(&mut self, status: WebRTCRoomStatus) {
        self.status = status;
    }

    /// Set the sender client ID
    pub fn set_sender_client_id(&mut self, client_id: String) {
        self.sender_client_id = Some(client_id);
    }

    /// Set the receiver client ID
    pub fn set_receiver_client_id(&mut self, client_id: String) {
        self.receiver_client_id = Some(client_id);
    }

    /// Set the session ID
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Check if the room is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, WebRTCRoomStatus::Active)
    }
}

impl WebRTCClient {
    /// Create a new WebRTC client
    pub fn new(
        client_id: String,
        room_id: String,
        role: ClientRole,
        session_id: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            room_id,
            role,
            session_id,
            joined_at: Utc::now(),
            status: WebRTCClientStatus::Pending,
            metadata: metadata.unwrap_or_default(),
            record_created_at: Utc::now(),
        }
    }

    /// Get the client ID
    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    /// Get the room ID
    pub fn get_room_id(&self) -> &str {
        &self.room_id
    }

    /// Get the role
    pub fn get_role(&self) -> &ClientRole {
        &self.role
    }

    /// Get the session ID
    pub fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Update the client status
    pub fn update_status(&mut self, status: WebRTCClientStatus) {
        self.status = status;
    }

    /// Set the session ID
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Check if the client is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, WebRTCClientStatus::Active)
    }
}


 