use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::get_config;
use crate::database::{
    FirestoreRepositoryFactory, RepositoryFactory, WebRTCRoomRepository, WebRTCClientRepository,
    WebRTCRoomCreationPayload, WebRTCClientRegistrationPayload, ClientRole as DbClientRole,
};
use crate::cloudflare::{CloudflareSession, models::*};
use crate::config::Config;

pub const CURRENT_VERSION: &str = "1.0.0";

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
pub struct WebRTCRoomCreateResponse {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub room_id: Option<String>,
    pub session_id: Option<String>,
    pub app_id: Option<String>,
    pub stun_url: Option<String>,
    pub connection_info: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct WebRTCRoomCreateHandler {
    config: Arc<Config>,
}

impl WebRTCRoomCreateHandler {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn handle_room_create(&self, message: crate::message::Message) -> Result<crate::message::Message, Box<dyn std::error::Error + Send + Sync>> {
        let frame_id = message.uuid;
        let payload = match &message.payload {
            crate::message::Payload::WebRTCRoomCreate(payload) => payload,
            _ => return Err("Invalid message type".into()),
        };

        // Create repositories
        let factory = FirestoreRepositoryFactory::new(self.config.clone());
        let room_repository = match factory.create_webrtc_room_repository().await {
            Ok(repo) => repo,
            Err(e) => {
                error!("Failed to create room repository: {}", e);
                return Err("Database connection failed".into());
            }
        };

        let client_repository = match factory.create_webrtc_client_repository().await {
            Ok(repo) => repo,
            Err(e) => {
                error!("Failed to create client repository: {}", e);
                return Err("Database connection failed".into());
            }
        };

        let raw_payload = serde_json::to_value(payload)?;
        let (_, response_json) = handle_room_create_internal(
            frame_id, 
            raw_payload, 
            room_repository.clone(), 
            client_repository.clone()
        ).await;
        
        let response_payload: WebRTCRoomCreateResponse = serde_json::from_str(&response_json)?;
        
        // Debug logging for room creation
        if response_payload.status == 200 {
            info!("[WEBRTC_ROOM_CREATE] Room created: room_id={:?}, session_id={:?}, message={:?}", 
                response_payload.room_id, response_payload.session_id, response_payload.message);
        } else {
            warn!("[WEBRTC_ROOM_CREATE] Room creation failed: room_id={:?}, status={}, message={:?}", 
                response_payload.room_id, response_payload.status, response_payload.message);
        }

        let message_payload = if response_payload.status == 200 {
            crate::message::Payload::WebRTCRoomCreateAck(crate::message::WebRTCRoomCreateAckPayload {
                version: response_payload.version,
                status: response_payload.status,
                message: response_payload.message,
                room_id: response_payload.room_id,
                session_id: response_payload.session_id,
                app_id: response_payload.app_id,
                stun_url: response_payload.stun_url,
                connection_info: response_payload.connection_info,
            })
        } else {
            crate::message::Payload::Error(crate::message::ErrorPayload {
                error_code: response_payload.status as u8,
                error_message: response_payload.message.unwrap_or_else(|| "Unknown error".to_string()),
            })
        };

        Ok(crate::message::Message::new(
            crate::message::MessageType::WebRTCRoomCreateAck,
            message_payload,
        ))
    }
}

async fn handle_room_create_internal(
    frame_id: Uuid, 
    raw_payload: serde_json::Value,
    room_repository: Arc<dyn WebRTCRoomRepository + Send + Sync>,
    client_repository: Arc<dyn WebRTCClientRepository + Send + Sync>,
) -> (Uuid, String) {
    // Validate and parse JSON payload
    let version = raw_payload.get("version");
    let client_id = raw_payload.get("client_id");
    let auth_token = raw_payload.get("auth_token");
    let role = raw_payload.get("role");

    // Check required fields and types
    if version.is_none() || !version.unwrap().is_string() {
        return error_response(frame_id, 400, "Missing or invalid 'version' field");
    }
    if client_id.is_none() || !client_id.unwrap().is_string() {
        return error_response(frame_id, 400, "Missing or invalid 'client_id' field");
    }
    if auth_token.is_none() || !auth_token.unwrap().is_string() {
        return error_response(frame_id, 400, "Missing or invalid 'auth_token' field");
    }
    if role.is_none() || !role.unwrap().is_string() {
        return error_response(frame_id, 400, "Missing or invalid 'role' field");
    }

    let version_str = version.unwrap().as_str().unwrap();
    if version_str > CURRENT_VERSION {
        return error_response(frame_id, 400, "Unsupported version: newer than server");
    }

    // Parse the payload into WebRTCRoomCreatePayload
    let payload: WebRTCRoomCreatePayload = match serde_json::from_value(raw_payload) {
        Ok(p) => p,
        Err(_) => return error_response(frame_id, 400, "Malformed room create payload"),
    };

    info!("Processing WebRTC room create request for client: {} with role: {}", payload.client_id, payload.role);

    // Validate role
    let role_str = payload.role.to_lowercase();
    let client_role = match role_str.as_str() {
        "sender" => DbClientRole::Sender,
        "receiver" => DbClientRole::Receiver,
        _ => return error_response(frame_id, 400, "Invalid role: must be 'sender' or 'receiver'"),
    };

    // Validate offer_sdp for sender
    if client_role == DbClientRole::Sender && payload.offer_sdp.is_none() {
        return error_response(frame_id, 400, "Offer SDP is required for sender role");
    }

    // Generate room ID
    let room_id = CloudflareSession::generate_room_id();
    
    // Create Cloudflare session if sender
    let mut session_id = None;
    let mut connection_info = None;
    
    if client_role == DbClientRole::Sender {
        match create_cloudflare_session(&room_id, &payload.client_id, payload.offer_sdp.unwrap()).await {
            Ok(info) => {
                session_id = info.session_id.clone();
                connection_info = Some(serde_json::to_value(info).unwrap());
            }
            Err(e) => {
                error!("Failed to create Cloudflare session: {}", e);
                return error_response(frame_id, 500, "Failed to create Cloudflare session");
            }
        }
    }

    // Create room in database
    let room_payload = WebRTCRoomCreationPayload {
        room_id: room_id.clone(),
        app_id: get_config().cloudflare.app_id.clone(),
        sender_client_id: if client_role == DbClientRole::Sender { Some(payload.client_id.clone()) } else { None },
        receiver_client_id: if client_role == DbClientRole::Receiver { Some(payload.client_id.clone()) } else { None },
        session_id: session_id.clone(),
        metadata: payload.metadata.clone(),
    };

    match room_repository.create_room(room_payload).await {
        Ok(_) => {
            info!("Created WebRTC room: {}", room_id);
        }
        Err(e) => {
            error!("Failed to create room in database: {}", e);
            return error_response(frame_id, 500, "Failed to create room in database");
        }
    }

    // Register client in database
    let client_payload = WebRTCClientRegistrationPayload {
        client_id: payload.client_id.clone(),
        room_id: room_id.clone(),
        role: client_role,
        session_id: session_id.clone(),
        metadata: payload.metadata,
    };

    match client_repository.register_client(client_payload).await {
        Ok(_) => {
            info!("Registered WebRTC client: {} in room: {}", payload.client_id, room_id);
        }
        Err(e) => {
            error!("Failed to register client in database: {}", e);
            return error_response(frame_id, 500, "Failed to register client in database");
        }
    }

    // Create success response
    let response = WebRTCRoomCreateResponse {
        version: CURRENT_VERSION.to_string(),
        status: 200,
        message: Some("Room created successfully".to_string()),
        room_id: Some(room_id),
        session_id,
        app_id: Some(get_config().cloudflare.app_id.clone()),
        stun_url: Some(get_config().cloudflare.stun_url.clone()),
        connection_info,
    };

    let response_json = serde_json::to_string(&response).unwrap();
    (frame_id, response_json)
}

async fn create_cloudflare_session(
    room_id: &str,
    client_id: &str,
    offer_sdp: String,
) -> Result<WebRTCConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
    let config = get_config();
    let session_manager = CloudflareSession::new(Arc::new(config.clone()))?;
    
    session_manager.create_room_with_sender(room_id, client_id, offer_sdp).await
}

fn error_response(frame_id: Uuid, status: u16, message: &str) -> (Uuid, String) {
    let response = WebRTCRoomCreateResponse {
        version: CURRENT_VERSION.to_string(),
        status,
        message: Some(message.to_string()),
        room_id: None,
        session_id: None,
        app_id: None,
        stun_url: None,
        connection_info: None,
    };
    
    let response_json = serde_json::to_string(&response).unwrap();
    (frame_id, response_json)
} 