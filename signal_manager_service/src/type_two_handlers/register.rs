use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use tracing::{error, info};

use crate::config::get_config;
use crate::database::{
    FirestoreRepositoryFactory, RegistrationPayload as DbRegistrationPayload, RepositoryFactory,
    ClientRepository,
};
use crate::config::Config;

pub const CURRENT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub room_id: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
}

// Test helper struct for integration tests
pub struct RegisterHandler {
    repository: Arc<dyn ClientRepository + Send + Sync>,
}

impl RegisterHandler {
    pub fn new(_config: Arc<Config>, repository: Arc<dyn ClientRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn handle_register(&self, message: crate::message::Message) -> Result<crate::message::Message, Box<dyn std::error::Error>> {
        let frame_id = message.uuid;
        let payload = match &message.payload {
            crate::message::Payload::Register(payload) => payload,
            _ => return Err("Invalid message type".into()),
        };

        let raw_payload = serde_json::to_value(payload)?;
        let (_, response_json) = handle_register_internal(frame_id, raw_payload, self.repository.clone()).await;
        
        let response_payload: RegisterResponse = serde_json::from_str(&response_json)?;
        
        let message_payload = if response_payload.status == 200 {
            crate::message::Payload::RegisterAck(crate::message::RegisterAckPayload {
                version: response_payload.version,
                status: response_payload.status,
                message: response_payload.message,
                client_id: response_payload.client_id,
                session_id: response_payload.session_id,
            })
        } else {
            crate::message::Payload::Error(crate::message::ErrorPayload {
                error_code: response_payload.status as u8,
                error_message: response_payload.message.unwrap_or_else(|| "Unknown error".to_string()),
            })
        };

        Ok(crate::message::Message::new(
            crate::message::MessageType::RegisterAck,
            message_payload,
        ))
    }

    pub async fn handle_unregister(&self, message: crate::message::Message) -> Result<crate::message::Message, Box<dyn std::error::Error>> {
        let frame_id = message.uuid;
        let payload = match &message.payload {
            crate::message::Payload::Unregister(payload) => payload,
            _ => return Err("Invalid message type".into()),
        };

        let raw_payload = serde_json::to_value(payload)?;
        let (_, response_json) = handle_unregister_internal(frame_id, raw_payload, self.repository.clone()).await;
        
        let response_payload: UnregisterResponse = serde_json::from_str(&response_json)?;
        
        let message_payload = if response_payload.status == 200 {
            crate::message::Payload::UnregisterAck(crate::message::UnregisterAckPayload {
                version: response_payload.version,
                status: response_payload.status,
                message: response_payload.message,
                client_id: response_payload.client_id,
            })
        } else {
            crate::message::Payload::Error(crate::message::ErrorPayload {
                error_code: response_payload.status as u8,
                error_message: response_payload.message.unwrap_or_else(|| "Unknown error".to_string()),
            })
        };

        Ok(crate::message::Message::new(
            crate::message::MessageType::UnregisterAck,
            message_payload,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub version: String,
    pub status: u16,
    pub message: Option<String>,
    pub client_id: Option<String>,
}

async fn handle_register_internal(
    frame_id: Uuid, 
    raw_payload: serde_json::Value,
    repository: Arc<dyn ClientRepository + Send + Sync>
) -> (Uuid, String) {
    // Validate and parse JSON payload
    let version = raw_payload.get("version");
    let client_id = raw_payload.get("client_id");
    let auth_token = raw_payload.get("auth_token");

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

    let version_str = version.unwrap().as_str().unwrap();
    if version_str > CURRENT_VERSION {
        return error_response(frame_id, 400, "Unsupported version: newer than server");
    }

    // Parse the payload into RegisterPayload
    let payload: RegisterPayload = match serde_json::from_value(raw_payload) {
        Ok(p) => p,
        Err(_) => return error_response(frame_id, 400, "Malformed register payload"),
    };

    info!("Processing register request for client: {}", payload.client_id);

    // Validate again for empty strings
    if payload.client_id.trim().is_empty() {
        return error_response(frame_id, 400, "Client ID is required");
    }
    if payload.auth_token.trim().is_empty() {
        return error_response(frame_id, 400, "Auth token is required");
    }

    let db_payload = DbRegistrationPayload {
        client_id: payload.client_id.clone(),
        auth_token: payload.auth_token,
        room_id: payload.room_id,
        capabilities: payload.capabilities,
        metadata: payload.metadata,
    };

    match repository.create_client(db_payload).await {
        Ok(client) => {
            info!("Successfully registered client: {}", client.client_id);
            let session_id = Uuid::new_v4().to_string();
            let response = RegisterResponse {
                version: CURRENT_VERSION.to_string(),
                status: 200,
                message: Some("Registration successful".to_string()),
                client_id: Some(client.client_id),
                session_id: Some(session_id),
            };
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| format!("{{\"version\":\"{}\",\"status\":500}}", CURRENT_VERSION));
            (frame_id, response_json)
        }
        Err(e) => {
            error!("Failed to register client: {}", e);
            let status = match e {
                crate::database::DatabaseError::Validation(_) => 409,
                crate::database::DatabaseError::Authentication(_) => 401,
                crate::database::DatabaseError::Connection(_) => 503,
                _ => 500,
            };
            let response = RegisterResponse {
                version: CURRENT_VERSION.to_string(),
                status,
                message: Some(format!("Registration failed: {}", e)),
                client_id: None,
                session_id: None,
            };
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| format!("{{\"version\":\"{}\",\"status\":500}}", CURRENT_VERSION));
            (frame_id, response_json)
        }
    }
}

async fn handle_unregister_internal(
    frame_id: Uuid, 
    raw_payload: serde_json::Value,
    repository: Arc<dyn ClientRepository + Send + Sync>
) -> (Uuid, String) {
    // Validate and parse JSON payload
    let version = raw_payload.get("version");
    let client_id = raw_payload.get("client_id");
    let auth_token = raw_payload.get("auth_token");

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

    let version_str = version.unwrap().as_str().unwrap();
    if version_str > CURRENT_VERSION {
        return error_response(frame_id, 400, "Unsupported version: newer than server");
    }

    // Parse the payload into UnregisterPayload
    let payload: crate::message::UnregisterPayload = match serde_json::from_value(raw_payload) {
        Ok(p) => p,
        Err(_) => return error_response(frame_id, 400, "Malformed unregister payload"),
    };

    info!("Processing unregister request for client: {}", payload.client_id);

    // Validate again for empty strings
    if payload.client_id.trim().is_empty() {
        return error_response(frame_id, 400, "Client ID is required");
    }
    if payload.auth_token.trim().is_empty() {
        return error_response(frame_id, 400, "Auth token is required");
    }

    // Validate auth before deleting
    match repository.validate_auth(&payload.client_id, &payload.auth_token).await {
        Ok(true) => {},
        Ok(false) => {
            return error_response(frame_id, 401, "Invalid client_id or auth_token");
        }
        Err(e) => {
            error!("Failed to validate auth: {}", e);
            return error_response(frame_id, 500, "Database error during auth validation");
        }
    }

    match repository.delete_client(&payload.client_id).await {
        Ok(true) => {
            info!("Successfully unregistered client: {}", payload.client_id);
            let response = UnregisterResponse {
                version: CURRENT_VERSION.to_string(),
                status: 200,
                message: Some("Unregistration successful".to_string()),
                client_id: Some(payload.client_id),
            };
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| format!("{{\"version\":\"{}\",\"status\":500}}", CURRENT_VERSION));
            (frame_id, response_json)
        }
        Ok(false) => {
            error!("Client not found for unregistration: {}", payload.client_id);
            error_response(frame_id, 404, "Client not found")
        }
        Err(e) => {
            error!("Failed to unregister client: {}", e);
            let response = UnregisterResponse {
                version: CURRENT_VERSION.to_string(),
                status: 500,
                message: Some(format!("Unregistration failed: {}", e)),
                client_id: None,
            };
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| format!("{{\"version\":\"{}\",\"status\":500}}", CURRENT_VERSION));
            (frame_id, response_json)
        }
    }
}

pub async fn handle_register(frame_id: Uuid, raw_payload: serde_json::Value) -> (Uuid, String) {
    // Get configuration
    let config = get_config();
    let config_arc = Arc::new(config.clone());
    let factory = FirestoreRepositoryFactory::new(config_arc);
    let repository = match factory.create_client_repository().await {
        Ok(repo) => repo,
        Err(e) => {
            error!("Failed to create repository: {}", e);
            return error_response(frame_id, 500, "Database connection failed");
        }
    };

    handle_register_internal(frame_id, raw_payload, repository).await
}

fn error_response(frame_id: Uuid, status: u16, message: &str) -> (Uuid, String) {
    let response = RegisterResponse {
        version: CURRENT_VERSION.to_string(),
        status,
        message: Some(message.to_string()),
        client_id: None,
        session_id: None,
    };
    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| format!("{{\"version\":\"{}\",\"status\":500}}", CURRENT_VERSION));
    (frame_id, response_json)
} 