use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::type_two_handlers::register;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Type2Payload {
    REGISTER(register::RegisterPayload),
    // Add more payload types here
}

pub async fn handle_type2_message(
    frame_id: Uuid,
    json_payload: &str,
) -> (Uuid, String) {
    // Try to parse the incoming JSON as a Type2Payload
    let result: Result<Type2Payload, _> = serde_json::from_str(json_payload);
    match result {
        Ok(Type2Payload::REGISTER(register_payload)) => {
            register::handle_register(frame_id, register_payload).await
        }
        Err(e) => {
            let response = serde_json::json!({
                "status": 400,
                "message": format!("Invalid payload: {}", e)
            });
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "{\"status\":500}".to_string());
            (frame_id, response_json)
        }
    }
} 