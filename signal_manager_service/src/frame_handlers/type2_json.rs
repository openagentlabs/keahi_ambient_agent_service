use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;
use crate::type_two_handlers::{register, unregister};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Type2Payload {
    REGISTER(register::RegisterPayload),
    UNREGISTER(unregister::UnregisterPayload),
    // Add more payload types here
}

pub async fn handle_type2_message(
    frame_id: Uuid,
    json_payload: &str,
) -> (Uuid, String) {
    // Parse the incoming JSON as a Value
    let value_result: Result<Value, _> = serde_json::from_str(json_payload);
    match value_result {
        Ok(val) => {
            // Check for type field
            if let Some(type_field) = val.get("type") {
                if type_field == "REGISTER" {
                    return register::handle_register(frame_id, val).await;
                } else if type_field == "UNREGISTER" {
                    return unregister::handle_unregister(frame_id, val).await;
                }
            }
            let response = serde_json::json!({
                "status": 400,
                "message": "Unknown or missing type field"
            });
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "{\"status\":500}".to_string());
            (frame_id, response_json)
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