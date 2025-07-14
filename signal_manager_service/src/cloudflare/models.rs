use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a Cloudflare Realtime session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareSession {
    /// Unique session identifier from Cloudflare
    pub session_id: String,
    /// Room UUID that this session belongs to
    pub room_id: String,
    /// Client ID that owns this session
    pub client_id: String,
    /// Session description (SDP)
    pub session_description: SessionDescription,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// Session status
    pub status: SessionStatus,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Session description for WebRTC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    /// Type of session description (offer/answer)
    pub r#type: String,
    /// Session description protocol (SDP)
    pub sdp: String,
}

/// Track information for WebRTC streams
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Track {
    /// Track location (local/remote)
    pub location: String,
    /// Media ID for the track
    pub mid: Option<String>,
    /// Track name/identifier
    pub track_name: String,
    /// Session ID for remote tracks
    pub session_id: Option<String>,
}

/// Session status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum SessionStatus {
    Active,
    Inactive,
    Terminated,
    #[default]
    Pending,
}

/// WebRTC room information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoom {
    /// Unique room UUID
    pub room_id: String,
    /// Cloudflare app ID
    pub app_id: String,
    /// Room creation timestamp
    pub created_at: DateTime<Utc>,
    /// Room status
    pub status: RoomStatus,
    /// Sender client ID
    pub sender_client_id: Option<String>,
    /// Receiver client ID
    pub receiver_client_id: Option<String>,
    /// Room metadata
    pub metadata: serde_json::Value,
}

/// Room status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum RoomStatus {
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
    pub status: ClientStatus,
    /// Client metadata
    pub metadata: serde_json::Value,
}

/// Client status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ClientStatus {
    Active,
    Inactive,
    Disconnected,
    #[default]
    Pending,
}

/// Cloudflare API response for session creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareSessionResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "sessionDescription")]
    pub session_description: SessionDescription,
}

/// Cloudflare API response for track operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareTracksResponse {
    pub session_description: Option<SessionDescription>,
    pub tracks: Vec<Track>,
    pub requires_immediate_renegotiation: Option<bool>,
}

/// Cloudflare API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareError {
    pub error_code: Option<String>,
    pub error_description: String,
}

/// WebRTC connection information for clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCConnectionInfo {
    /// Room ID
    pub room_id: String,
    /// Client role
    pub role: ClientRole,
    /// Cloudflare app ID
    pub app_id: String,
    /// Session ID (if active)
    pub session_id: Option<String>,
    /// Connection status
    pub status: ConnectionStatus,
    /// Additional connection metadata
    pub metadata: serde_json::Value,
}

/// Connection status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    #[default]
    Disconnected,
    Failed,
}




 