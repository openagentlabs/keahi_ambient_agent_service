use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Retry intervals in milliseconds
pub const RETRY_INTERVALS: RetryIntervals = RetryIntervals {
    immediate: 0,
    short: 10000,    // 10 seconds
    medium: 30000,   // 30 seconds
    long: 60000,     // 60 seconds
    max: 60000,      // Maximum retry interval (60 seconds)
};

pub struct RetryIntervals {
    pub immediate: u64,
    pub short: u64,
    pub medium: u64,
    pub long: u64,
    pub max: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionStateType {
    #[serde(rename = "disconnected_not_to_connect")]
    DisconnectedNotToConnect,
    #[serde(rename = "trying_to_connect")]
    TryingToConnect,
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "was_connected_trying_to_reconnect")]
    WasConnectedTryingToReconnect,
    #[serde(rename = "disconnecting_disconnect_requested")]
    DisconnectingDisconnectRequested,
}

impl ConnectionStateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionStateType::DisconnectedNotToConnect => "disconnected_not_to_connect",
            ConnectionStateType::TryingToConnect => "trying_to_connect",
            ConnectionStateType::Connected => "connected",
            ConnectionStateType::WasConnectedTryingToReconnect => "was_connected_trying_to_reconnect",
            ConnectionStateType::DisconnectingDisconnectRequested => "disconnecting_disconnect_requested",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionState {
    pub state_type: ConnectionStateType,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub is_reconnecting: bool,
    pub last_heartbeat: u64,
    pub reconnect_attempts: u32,
    pub current_retry_interval: u64,
    pub next_retry_time: Option<u64>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            state_type: ConnectionStateType::DisconnectedNotToConnect,
            is_connected: false,
            is_connecting: false,
            is_reconnecting: false,
            last_heartbeat: 0,
            reconnect_attempts: 0,
            current_retry_interval: 0,
            next_retry_time: None,
        }
    }
}

// Protocol constants
pub const START_BYTE: u8 = 0xAA;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    Connect = 0x01,
    ConnectAck = 0x02,
    Disconnect = 0x03,
    Heartbeat = 0x04,
    HeartbeatAck = 0x05,
    SignalOffer = 0x10,
    SignalAnswer = 0x11,
    SignalIceCandidate = 0x12,
    Register = 0x20,
    RegisterAck = 0x21,
    Unregister = 0x22,
    UnregisterAck = 0x23,
    WebRTCRoomCreate = 0x30,
    WebRTCRoomCreateAck = 0x31,
    WebRTCRoomJoin = 0x32,
    WebRTCRoomJoinAck = 0x33,
    WebRTCRoomLeave = 0x34,
    WebRTCRoomLeaveAck = 0x35,
    Error = 0xFF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayloadType {
    Binary = 0x01,
    Json = 0x02,
    Text = 0x03,
    Protobuf = 0x04,
    Cbor = 0x05,
}

// Payload structures matching server protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectPayload {
    pub client_id: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectAckPayload {
    pub status: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectPayload {
    pub client_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAckPayload {
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPayload {
    pub target_client_id: String,
    pub signal_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAckPayload {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterPayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterAckPayload {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub error_code: u8,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomCreatePayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub role: String,
    pub offer_sdp: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomCreateAckPayload {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub room_id: Option<String>,
    pub session_id: Option<String>,
    pub app_id: Option<String>,
    pub stun_url: Option<String>,
    pub connection_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    Connect(ConnectPayload),
    ConnectAck(ConnectAckPayload),
    Disconnect(DisconnectPayload),
    Heartbeat(HeartbeatPayload),
    HeartbeatAck(HeartbeatAckPayload),
    SignalOffer(SignalPayload),
    SignalAnswer(SignalPayload),
    SignalIceCandidate(SignalPayload),
    Register(RegisterPayload),
    RegisterAck(RegisterAckPayload),
    Unregister(UnregisterPayload),
    UnregisterAck(UnregisterAckPayload),
    WebRTCRoomCreate(WebRTCRoomCreatePayload),
    WebRTCRoomCreateAck(WebRTCRoomCreateAckPayload),
    Error(ErrorPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    #[serde(with = "uuid::serde::compact")]
    pub uuid: Uuid,
    pub payload_type: PayloadType,
    pub payload: Payload,
}

impl Message {
    pub fn new(message_type: MessageType, payload: Payload) -> Self {
        Self {
            message_type,
            uuid: Uuid::new_v4(),
            payload_type: PayloadType::Json,
            payload,
        }
    }

    pub fn connect(client_id: String, auth_token: String) -> Self {
        Self::new(
            MessageType::Connect,
            Payload::Connect(ConnectPayload {
                client_id,
                auth_token,
            })
        )
    }

    pub fn register(client_id: String, auth_token: String) -> Self {
        Self::new(
            MessageType::Register,
            Payload::Register(RegisterPayload {
                version: "1.0.0".to_string(),
                client_id,
                auth_token,
                capabilities: None,
                metadata: None,
            })
        )
    }

    pub fn unregister(client_id: String) -> Self {
        Self::new(
            MessageType::Unregister,
            Payload::Unregister(UnregisterPayload {
                version: "1.0.0".to_string(),
                client_id,
                auth_token: "".to_string(), // Not needed for unregister
            })
        )
    }

    pub fn heartbeat() -> Self {
        Self::new(
            MessageType::Heartbeat,
            Payload::Heartbeat(HeartbeatPayload {
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            })
        )
    }

    pub fn room_create(payload: WebRTCRoomCreatePayload) -> Self {
        Self::new(MessageType::WebRTCRoomCreate, Payload::WebRTCRoomCreate(payload))
    }

    // Binary serialization for server compatibility
    pub fn to_binary(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = Vec::new();
        
        // Start byte
        buffer.push(START_BYTE);
        
        // Message type
        buffer.push(self.message_type as u8);
        
        // UUID (16 bytes)
        buffer.extend_from_slice(self.uuid.as_bytes());
        
        // Payload type
        buffer.push(self.payload_type as u8);
        
        // Serialize payload as JSON
        let payload_bytes = serde_json::to_vec(&self.payload)?;
        
        // Payload length (2 bytes, big endian)
        let length = payload_bytes.len() as u16;
        buffer.extend_from_slice(&length.to_be_bytes());
        
        // Payload
        buffer.extend_from_slice(&payload_bytes);
        
        Ok(buffer)
    }

    pub fn from_binary(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if data.len() < 22 {
            return Err("Message too short".into());
        }

        if data[0] != START_BYTE {
            return Err("Invalid start byte".into());
        }

        let message_type = match data[1] {
            0x01 => MessageType::Connect,
            0x02 => MessageType::ConnectAck,
            0x03 => MessageType::Disconnect,
            0x04 => MessageType::Heartbeat,
            0x05 => MessageType::HeartbeatAck,
            0x10 => MessageType::SignalOffer,
            0x11 => MessageType::SignalAnswer,
            0x12 => MessageType::SignalIceCandidate,
            0x20 => MessageType::Register,
            0x21 => MessageType::RegisterAck,
            0x22 => MessageType::Unregister,
            0x23 => MessageType::UnregisterAck,
            0x30 => MessageType::WebRTCRoomCreate,
            0x31 => MessageType::WebRTCRoomCreateAck,
            0x32 => MessageType::WebRTCRoomJoin,
            0x33 => MessageType::WebRTCRoomJoinAck,
            0x34 => MessageType::WebRTCRoomLeave,
            0x35 => MessageType::WebRTCRoomLeaveAck,
            0xFF => MessageType::Error,
            _ => return Err("Unknown message type".into()),
        };

        let uuid = Uuid::from_slice(&data[2..18])?;
        let payload_type = match data[18] {
            0x01 => PayloadType::Binary,
            0x02 => PayloadType::Json,
            0x03 => PayloadType::Text,
            0x04 => PayloadType::Protobuf,
            0x05 => PayloadType::Cbor,
            _ => return Err("Unknown payload type".into()),
        };
        
        let length_bytes = [data[19], data[20]];
        let payload_length = u16::from_be_bytes(length_bytes) as usize;
        
        if data.len() < 21 + payload_length {
            return Err("Message length mismatch".into());
        }

        let payload_data = &data[21..21 + payload_length];
        let payload = serde_json::from_slice(payload_data)?;

        Ok(Self {
            message_type,
            uuid,
            payload_type,
            payload,
        })
    }
}

impl MessageType {
    pub fn from_u8(value: u8) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value {
            0x01 => Ok(MessageType::Connect),
            0x02 => Ok(MessageType::ConnectAck),
            0x03 => Ok(MessageType::Disconnect),
            0x04 => Ok(MessageType::Heartbeat),
            0x05 => Ok(MessageType::HeartbeatAck),
            0x10 => Ok(MessageType::SignalOffer),
            0x11 => Ok(MessageType::SignalAnswer),
            0x12 => Ok(MessageType::SignalIceCandidate),
            0x20 => Ok(MessageType::Register),
            0x21 => Ok(MessageType::RegisterAck),
            0x22 => Ok(MessageType::Unregister),
            0x23 => Ok(MessageType::UnregisterAck),
            0x30 => Ok(MessageType::WebRTCRoomCreate),
            0x31 => Ok(MessageType::WebRTCRoomCreateAck),
            0x32 => Ok(MessageType::WebRTCRoomJoin),
            0x33 => Ok(MessageType::WebRTCRoomJoinAck),
            0x34 => Ok(MessageType::WebRTCRoomLeave),
            0x35 => Ok(MessageType::WebRTCRoomLeaveAck),
            0xFF => Ok(MessageType::Error),
            _ => Err("Unknown message type".into()),
        }
    }
}

impl PayloadType {
    pub fn from_u8(value: u8) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value {
            0x01 => Ok(PayloadType::Binary),
            0x02 => Ok(PayloadType::Json),
            0x03 => Ok(PayloadType::Text),
            0x04 => Ok(PayloadType::Protobuf),
            0x05 => Ok(PayloadType::Cbor),
            _ => Err("Unknown payload type".into()),
        }
    }
} 