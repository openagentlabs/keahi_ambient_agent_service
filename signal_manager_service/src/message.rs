use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::frame_handlers::type2_json;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    #[serde(with = "uuid::serde::compact")]
    pub uuid: Uuid,
    pub payload_type: PayloadType,
    pub payload: Payload,
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
    WebRTCRoomJoin(WebRTCRoomJoinPayload),
    WebRTCRoomJoinAck(WebRTCRoomJoinAckPayload),
    WebRTCRoomLeave(WebRTCRoomLeavePayload),
    WebRTCRoomLeaveAck(WebRTCRoomLeaveAckPayload),
    Error(ErrorPayload),
}

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

// WebRTC Room Management Payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomCreatePayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub role: String, // "sender" or "receiver"
    pub offer_sdp: Option<String>, // Required for sender
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
pub struct WebRTCRoomJoinPayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub room_id: String,
    pub role: String, // "sender" or "receiver"
    pub offer_sdp: Option<String>, // Required for sender
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomJoinAckPayload {
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
pub struct WebRTCRoomLeavePayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub room_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomLeaveAckPayload {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub room_id: Option<String>,
    pub client_id: Option<String>,
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

    pub fn to_binary(&self) -> Result<Vec<u8>, crate::Error> {
        let mut buffer = Vec::new();
        
        // Start byte
        buffer.push(START_BYTE);
        
        // Message type
        buffer.push(self.message_type as u8);
        
        // UUID (16 bytes)
        buffer.extend_from_slice(self.uuid.as_bytes());
        
        // Payload type
        buffer.push(self.payload_type as u8);
        
        // Serialize payload based on type
        let payload_bytes = match &self.payload_type {
            PayloadType::Json => {
                
                serde_json::to_vec(&self.payload)?
            }
            PayloadType::Binary => {
                self.payload_to_binary()?
            }
            PayloadType::Text => {
                self.payload_to_text()?.into_bytes()
            }
            _ => return Err(crate::Error::MessageParse("Unsupported payload type".to_string())),
        };
        
        // Payload length (2 bytes, big endian)
        let length = payload_bytes.len() as u16;
        buffer.extend_from_slice(&length.to_be_bytes());
        
        // Payload
        buffer.extend_from_slice(&payload_bytes);
        
        Ok(buffer)
    }

    pub fn from_binary(data: &[u8]) -> Result<Self, crate::Error> {
        if data.len() < 22 {
            return Err(crate::Error::MessageParse("Message too short".to_string()));
        }

        if data[0] != START_BYTE {
            return Err(crate::Error::MessageParse("Invalid start byte".to_string()));
        }

        let message_type = MessageType::from_u8(data[1])?;
        let uuid = Uuid::from_slice(&data[2..18])?;
        let payload_type = PayloadType::from_u8(data[18])?;
        
        let length_bytes = [data[19], data[20]];
        let payload_length = u16::from_be_bytes(length_bytes) as usize;
        
        if data.len() < 21 + payload_length {
            return Err(crate::Error::PayloadLengthMismatch {
                expected: 21 + payload_length,
                actual: data.len(),
            });
        }

        let payload_data = &data[21..21 + payload_length];
        let payload = match payload_type {
            PayloadType::Json => {
                let payload: Payload = serde_json::from_slice(payload_data)?;
                payload
            }
            PayloadType::Binary => {
                Self::payload_from_binary(payload_data, message_type)?
            }
            PayloadType::Text => {
                let text = String::from_utf8_lossy(payload_data);
                Self::payload_from_text(&text, message_type)?
            }
            _ => return Err(crate::Error::MessageParse("Unsupported payload type".to_string())),
        };

        Ok(Self {
            message_type,
            uuid,
            payload_type,
            payload,
        })
    }

    fn payload_to_binary(&self) -> Result<Vec<u8>, crate::Error> {
        // Implement binary serialization for each payload type
        match &self.payload {
            Payload::Connect(p) => {
                let mut buffer = Vec::new();
                buffer.push(p.client_id.len() as u8);
                buffer.extend_from_slice(p.client_id.as_bytes());
                buffer.push(p.auth_token.len() as u8);
                buffer.extend_from_slice(p.auth_token.as_bytes());
                Ok(buffer)
            }
            Payload::Register(p) => {
                let mut buffer = Vec::new();
                buffer.push(p.version.len() as u8);
                buffer.extend_from_slice(p.version.as_bytes());
                buffer.push(p.client_id.len() as u8);
                buffer.extend_from_slice(p.client_id.as_bytes());
                buffer.push(p.auth_token.len() as u8);
                buffer.extend_from_slice(p.auth_token.as_bytes());
                if let Some(capabilities) = &p.capabilities {
                    buffer.push(capabilities.len() as u8);
                    for cap in capabilities {
                        buffer.extend_from_slice(cap.as_bytes());
                    }
                } else {
                    buffer.push(0);
                }
                if let Some(metadata) = &p.metadata {
                    let json = serde_json::to_vec(metadata)?;
                    buffer.push(json.len() as u8);
                    buffer.extend_from_slice(&json);
                } else {
                    buffer.push(0);
                }
                Ok(buffer)
            }
            Payload::Unregister(p) => {
                let mut buffer = Vec::new();
                buffer.push(p.version.len() as u8);
                buffer.extend_from_slice(p.version.as_bytes());
                buffer.push(p.client_id.len() as u8);
                buffer.extend_from_slice(p.client_id.as_bytes());
                buffer.push(p.auth_token.len() as u8);
                buffer.extend_from_slice(p.auth_token.as_bytes());
                Ok(buffer)
            }
            _ => Err(crate::Error::MessageParse("Binary serialization not implemented".to_string())),
        }
    }

    fn payload_to_text(&self) -> Result<String, crate::Error> {
        match &self.payload {
            Payload::Connect(p) => Ok(format!("{}:{}", p.client_id, p.auth_token)),
            Payload::ConnectAck(p) => Ok(format!("{}:{}", p.status, p.session_id)),
            Payload::SignalOffer(p) | Payload::SignalAnswer(p) | Payload::SignalIceCandidate(p) => {
                Ok(format!("{}:{}", p.target_client_id, p.signal_data))
            }
            Payload::Register(p) => Ok(format!("{}:{}:{}", p.version, p.client_id, p.auth_token)),
            Payload::RegisterAck(p) => Ok(format!("{}:{}:{}:{}:{}", p.version, p.status, p.message.as_deref().unwrap_or(""), p.client_id.as_deref().unwrap_or(""), p.session_id.as_deref().unwrap_or(""))),
            Payload::Unregister(p) => Ok(format!("{}:{}:{}", p.version, p.client_id, p.auth_token)),
            Payload::UnregisterAck(p) => Ok(format!("{}:{}:{}:{}", p.version, p.status, p.message.as_deref().unwrap_or(""), p.client_id.as_deref().unwrap_or(""))),
            Payload::Error(p) => Ok(format!("{}:{}", p.error_code, p.error_message)),
            _ => Err(crate::Error::MessageParse("Text serialization not implemented".to_string())),
        }
    }

    fn payload_from_binary(data: &[u8], message_type: MessageType) -> Result<Payload, crate::Error> {
        match message_type {
            MessageType::Connect => {
                if data.len() < 2 {
                    return Err(crate::Error::MessageParse("Invalid connect payload".to_string()));
                }
                let client_id_len = data[0] as usize;
                if data.len() < 1 + client_id_len + 1 {
                    return Err(crate::Error::MessageParse("Invalid connect payload".to_string()));
                }
                let client_id = String::from_utf8_lossy(&data[1..1 + client_id_len]).to_string();
                let auth_token_len = data[1 + client_id_len] as usize;
                if data.len() < 1 + client_id_len + 1 + auth_token_len {
                    return Err(crate::Error::MessageParse("Invalid connect payload".to_string()));
                }
                let auth_token = String::from_utf8_lossy(&data[1 + client_id_len + 1..1 + client_id_len + 1 + auth_token_len]).to_string();
                Ok(Payload::Connect(ConnectPayload { client_id, auth_token }))
            }
            MessageType::Register => {
                if data.len() < 2 {
                    return Err(crate::Error::MessageParse("Invalid register payload".to_string()));
                }
                let version_len = data[0] as usize;
                if data.len() < 1 + version_len + 1 {
                    return Err(crate::Error::MessageParse("Invalid register payload".to_string()));
                }
                let version = String::from_utf8_lossy(&data[1..1 + version_len]).to_string();
                let client_id_len = data[1 + version_len] as usize;
                if data.len() < 1 + version_len + 1 + client_id_len + 1 {
                    return Err(crate::Error::MessageParse("Invalid register payload".to_string()));
                }
                let client_id = String::from_utf8_lossy(&data[1 + version_len + 1..1 + version_len + 1 + client_id_len]).to_string();
                let auth_token_len = data[1 + version_len + 1 + client_id_len] as usize;
                if data.len() < 1 + version_len + 1 + client_id_len + 1 + auth_token_len {
                    return Err(crate::Error::MessageParse("Invalid register payload".to_string()));
                }
                let auth_token = String::from_utf8_lossy(&data[1 + version_len + 1 + client_id_len + 1..1 + version_len + 1 + client_id_len + 1 + auth_token_len]).to_string();
                let mut capabilities: Option<Vec<String>> = None;
                let mut metadata: Option<serde_json::Value> = None;

                let capabilities_start = 1 + version_len + 1 + client_id_len + 1 + auth_token_len;
                let mut capabilities_len = 0;
                if data.len() > capabilities_start {
                    capabilities_len = data[capabilities_start] as usize;
                    if data.len() < capabilities_start + 1 + capabilities_len {
                        return Err(crate::Error::MessageParse("Invalid register payload".to_string()));
                    }
                    let mut caps = Vec::new();
                    for i in 0..capabilities_len {
                        caps.push(String::from_utf8_lossy(&data[capabilities_start + 1 + i..capabilities_start + 1 + i + 1]).to_string());
                    }
                    capabilities = Some(caps);
                }

                let metadata_start = capabilities_start + 1 + capabilities_len;
                if data.len() > metadata_start {
                    let metadata_len = data[metadata_start] as usize;
                    if data.len() < metadata_start + 1 + metadata_len {
                        return Err(crate::Error::MessageParse("Invalid register payload".to_string()));
                    }
                    let json: serde_json::Value = serde_json::from_slice(&data[metadata_start + 1..metadata_start + 1 + metadata_len])?;
                    metadata = Some(json);
                }

                Ok(Payload::Register(RegisterPayload { version, client_id, auth_token, capabilities, metadata }))
            }
            MessageType::Unregister => {
                if data.len() < 2 {
                    return Err(crate::Error::MessageParse("Invalid unregister payload".to_string()));
                }
                let version_len = data[0] as usize;
                if data.len() < 1 + version_len + 1 {
                    return Err(crate::Error::MessageParse("Invalid unregister payload".to_string()));
                }
                let version = String::from_utf8_lossy(&data[1..1 + version_len]).to_string();
                let client_id_len = data[1 + version_len] as usize;
                if data.len() < 1 + version_len + 1 + client_id_len + 1 {
                    return Err(crate::Error::MessageParse("Invalid unregister payload".to_string()));
                }
                let client_id = String::from_utf8_lossy(&data[1 + version_len + 1..1 + version_len + 1 + client_id_len]).to_string();
                let auth_token_len = data[1 + version_len + 1 + client_id_len] as usize;
                if data.len() < 1 + version_len + 1 + client_id_len + 1 + auth_token_len {
                    return Err(crate::Error::MessageParse("Invalid unregister payload".to_string()));
                }
                let auth_token = String::from_utf8_lossy(&data[1 + version_len + 1 + client_id_len + 1..1 + version_len + 1 + client_id_len + 1 + auth_token_len]).to_string();
                Ok(Payload::Unregister(UnregisterPayload { version, client_id, auth_token }))
            }
            _ => Err(crate::Error::MessageParse("Binary deserialization not implemented".to_string())),
        }
    }

    fn payload_from_text(text: &str, message_type: MessageType) -> Result<Payload, crate::Error> {
        let parts: Vec<&str> = text.split(':').collect();
        if parts.len() < 2 {
            return Err(crate::Error::MessageParse("Invalid text format".to_string()));
        }

        match message_type {
            MessageType::Connect => {
                Ok(Payload::Connect(ConnectPayload {
                    client_id: parts[0].to_string(),
                    auth_token: parts[1].to_string(),
                }))
            }
            MessageType::ConnectAck => {
                Ok(Payload::ConnectAck(ConnectAckPayload {
                    status: parts[0].to_string(),
                    session_id: parts[1].to_string(),
                }))
            }
            MessageType::SignalOffer => {
                Ok(Payload::SignalOffer(SignalPayload {
                    target_client_id: parts[0].to_string(),
                    signal_data: parts[1].to_string(),
                }))
            }
            MessageType::SignalAnswer => {
                Ok(Payload::SignalAnswer(SignalPayload {
                    target_client_id: parts[0].to_string(),
                    signal_data: parts[1].to_string(),
                }))
            }
            MessageType::SignalIceCandidate => {
                Ok(Payload::SignalIceCandidate(SignalPayload {
                    target_client_id: parts[0].to_string(),
                    signal_data: parts[1].to_string(),
                }))
            }
            MessageType::Register => {
                Ok(Payload::Register(RegisterPayload {
                    version: parts[0].to_string(),
                    client_id: parts[1].to_string(),
                    auth_token: parts[2].to_string(),
                    capabilities: None,
                    metadata: None,
                }))
            }
            MessageType::RegisterAck => {
                let status = parts[0].parse::<u16>().map_err(|_| crate::Error::MessageParse("Invalid status".to_string()))?;
                let message = if parts.len() > 1 { Some(parts[1].to_string()) } else { None };
                let client_id = if parts.len() > 2 { Some(parts[2].to_string()) } else { None };
                let session_id = if parts.len() > 3 { Some(parts[3].to_string()) } else { None };
                Ok(Payload::RegisterAck(RegisterAckPayload { version: parts[0].to_string(), status, message, client_id, session_id }))
            }
            MessageType::Unregister => {
                Ok(Payload::Unregister(UnregisterPayload {
                    version: parts[0].to_string(),
                    client_id: parts[1].to_string(),
                    auth_token: parts[2].to_string(),
                }))
            }
            MessageType::UnregisterAck => {
                let status = parts[0].parse::<u16>().map_err(|_| crate::Error::MessageParse("Invalid status".to_string()))?;
                let message = if parts.len() > 1 { Some(parts[1].to_string()) } else { None };
                let client_id = if parts.len() > 2 { Some(parts[2].to_string()) } else { None };
                Ok(Payload::UnregisterAck(UnregisterAckPayload { version: parts[0].to_string(), status, message, client_id }))
            }
            MessageType::Error => {
                let error_code = parts[0].parse::<u8>().map_err(|_| crate::Error::MessageParse("Invalid error code".to_string()))?;
                Ok(Payload::Error(ErrorPayload {
                    error_code,
                    error_message: parts[1].to_string(),
                }))
            }
            _ => Err(crate::Error::MessageParse("Text deserialization not implemented".to_string())),
        }
    }

    /// If this is a type 2 (JSON) message, process it using the type2 handler
    pub async fn process_type2_if_applicable(&self) -> Option<(uuid::Uuid, String)> {
        if self.payload_type == PayloadType::Json {
            // Try to extract the raw JSON string from the payload
            // For now, we assume the payload is a serde_json::Value or struct serializable to JSON
            let json = serde_json::to_string(&self.payload).ok()?;
            Some(type2_json::handle_type2_message(self.uuid, &json).await)
        } else {
            None
        }
    }
}

impl MessageType {
    pub fn from_u8(value: u8) -> Result<Self, crate::Error> {
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
            _ => Err(crate::Error::InvalidMessageType(value)),
        }
    }
}

impl PayloadType {
    pub fn from_u8(value: u8) -> Result<Self, crate::Error> {
        match value {
            0x01 => Ok(PayloadType::Binary),
            0x02 => Ok(PayloadType::Json),
            0x03 => Ok(PayloadType::Text),
            0x04 => Ok(PayloadType::Protobuf),
            0x05 => Ok(PayloadType::Cbor),
            _ => Err(crate::Error::InvalidPayloadType(value)),
        }
    }
} 