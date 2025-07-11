use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub test: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub status: u16,
    pub message: Option<String>,
}

pub async fn handle_register(frame_id: Uuid, _payload: RegisterPayload) -> (Uuid, String) {
    // For REGISTER, always return 200 OK
    let response = RegisterResponse {
        status: 200,
        message: None,
    };
    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "{\"status\":500}".to_string());
    (frame_id, response_json)
} 