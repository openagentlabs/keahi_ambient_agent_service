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
pub enum ClientStatus {
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
pub enum ClientInRoomStatus {
    Active,
    Inactive,
    Away,
    Busy,
}

/// Client termination status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientTerminationStatus {
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

impl Default for ClientStatus {
    fn default() -> Self {
        ClientStatus::Active
    }
}

impl Default for ClientInRoomStatus {
    fn default() -> Self {
        ClientInRoomStatus::Active
    }
}

impl Default for ClientTerminationStatus {
    fn default() -> Self {
        ClientTerminationStatus::Disconnected
    }
} 