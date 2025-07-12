use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::get_config;
use crate::database::{
    FirestoreRepositoryFactory, RepositoryFactory, WebRTCRoomRepository, WebRTCClientRepository,
};
use crate::cloudflare::CloudflareSession;
use crate::config::Config;

pub const CURRENT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomLeavePayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub room_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCRoomLeaveResponse {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub room_id: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Clone)]
pub struct WebRTCRoomLeaveHandler {
    config: Arc<Config>,
}

impl WebRTCRoomLeaveHandler {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn handle_room_leave(&self, message: crate::message::Message) -> Result<crate::message::Message, Box<dyn std::error::Error + Send + Sync>> {
        let frame_id = message.uuid;
        let payload = match &message.payload {
            crate::message::Payload::WebRTCRoomLeave(payload) => payload,
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
        let (_, response_json) = handle_room_leave_internal(
            frame_id, 
            raw_payload, 
            room_repository.clone(), 
            client_repository.clone()
        ).await;
        
        let response_payload: WebRTCRoomLeaveResponse = serde_json::from_str(&response_json)?;
        
        // Debug logging for room leave
        if response_payload.status == 200 {
            info!("[WEBRTC_ROOM_LEAVE] Room left: room_id={:?}, client_id={:?}, message={:?}", 
                response_payload.room_id, response_payload.client_id, response_payload.message);
        } else {
            warn!("[WEBRTC_ROOM_LEAVE] Room leave failed: room_id={:?}, client_id={:?}, status={}, message={:?}", 
                response_payload.room_id, response_payload.client_id, response_payload.status, response_payload.message);
        }

        let message_payload = if response_payload.status == 200 {
            crate::message::Payload::WebRTCRoomLeaveAck(crate::message::WebRTCRoomLeaveAckPayload {
                version: response_payload.version,
                status: response_payload.status,
                message: response_payload.message,
                room_id: response_payload.room_id,
                client_id: response_payload.client_id,
            })
        } else {
            crate::message::Payload::Error(crate::message::ErrorPayload {
                error_code: response_payload.status as u8,
                error_message: response_payload.message.unwrap_or_else(|| "Unknown error".to_string()),
            })
        };

        Ok(crate::message::Message::new(
            crate::message::MessageType::WebRTCRoomLeaveAck,
            message_payload,
        ))
    }
}

async fn handle_room_leave_internal(
    frame_id: Uuid, 
    raw_payload: serde_json::Value,
    room_repository: Arc<dyn WebRTCRoomRepository + Send + Sync>,
    client_repository: Arc<dyn WebRTCClientRepository + Send + Sync>,
) -> (Uuid, String) {
    // Validate and parse JSON payload
    let version = raw_payload.get("version");
    let client_id = raw_payload.get("client_id");
    let auth_token = raw_payload.get("auth_token");
    let room_id = raw_payload.get("room_id");

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
    if room_id.is_none() || !room_id.unwrap().is_string() {
        return error_response(frame_id, 400, "Missing or invalid 'room_id' field");
    }

    let version_str = version.unwrap().as_str().unwrap();
    if version_str > CURRENT_VERSION {
        return error_response(frame_id, 400, "Unsupported version: newer than server");
    }

    // Parse the payload into WebRTCRoomLeavePayload
    let payload: WebRTCRoomLeavePayload = match serde_json::from_value(raw_payload) {
        Ok(p) => p,
        Err(_) => return error_response(frame_id, 400, "Malformed room leave payload"),
    };

    info!("Processing WebRTC room leave request for client: {} from room: {}", 
        payload.client_id, payload.room_id);

    // Check if room exists
    let _room = match room_repository.get_room_by_id(&payload.room_id).await {
        Ok(Some(room)) => room,
        Ok(None) => return error_response(frame_id, 404, "Room not found"),
        Err(e) => {
            error!("Failed to get room from database: {}", e);
            return error_response(frame_id, 500, "Database error");
        }
    };

    // Check if client is in the room
    let client = match client_repository.get_client_by_id(&payload.client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => return error_response(frame_id, 404, "Client not found"),
        Err(e) => {
            error!("Failed to get client from database: {}", e);
            return error_response(frame_id, 500, "Database error");
        }
    };

    if client.get_room_id() != payload.room_id {
        return error_response(frame_id, 400, "Client is not in the specified room");
    }

    // Terminate Cloudflare session if client has one
    if let Some(session_id) = client.get_session_id() {
        match terminate_cloudflare_session(session_id, &payload.room_id).await {
            Ok(_) => {
                info!("Terminated Cloudflare session: {} for room: {}", session_id, payload.room_id);
            }
            Err(e) => {
                warn!("Failed to terminate Cloudflare session: {}", e);
                // Continue with cleanup even if Cloudflare termination fails
            }
        }
    }

    // Remove client from room
    match client_repository.remove_client_from_room(&payload.client_id, &payload.room_id).await {
        Ok(_) => {
            info!("Removed client: {} from room: {}", payload.client_id, payload.room_id);
        }
        Err(e) => {
            error!("Failed to remove client from room: {}", e);
            return error_response(frame_id, 500, "Failed to remove client from room");
        }
    }

    // Check if room is now empty and terminate it
    let remaining_clients = match client_repository.get_clients_by_room_id(&payload.room_id).await {
        Ok(clients) => clients,
        Err(e) => {
            error!("Failed to get remaining clients: {}", e);
            return error_response(frame_id, 500, "Database error");
        }
    };

    if remaining_clients.is_empty() {
        // Terminate the room
        match room_repository.terminate_room(&payload.room_id, "Room empty").await {
            Ok(_) => {
                info!("Terminated empty room: {}", payload.room_id);
            }
            Err(e) => {
                error!("Failed to terminate room: {}", e);
                // Continue with response even if room termination fails
            }
        }
    }

    // Create success response
    let response = WebRTCRoomLeaveResponse {
        version: CURRENT_VERSION.to_string(),
        status: 200,
        message: Some("Left room successfully".to_string()),
        room_id: Some(payload.room_id),
        client_id: Some(payload.client_id),
    };

    let response_json = serde_json::to_string(&response).unwrap();
    (frame_id, response_json)
}

async fn terminate_cloudflare_session(
    session_id: &str,
    room_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = get_config();
    let session_manager = CloudflareSession::new(Arc::new(config.clone()))?;
    
    session_manager.terminate_session(session_id, room_id).await
}

fn error_response(frame_id: Uuid, status: u16, message: &str) -> (Uuid, String) {
    let response = WebRTCRoomLeaveResponse {
        version: CURRENT_VERSION.to_string(),
        status,
        message: Some(message.to_string()),
        room_id: None,
        client_id: None,
    };
    
    let response_json = serde_json::to_string(&response).unwrap();
    (frame_id, response_json)
} 