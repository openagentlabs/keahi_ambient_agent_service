use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDPOffer {
    pub sdp: String,
    pub type_: String,
}

impl SDPOffer {
    pub fn new(sdp: String, type_: String) -> Self {
        Self { sdp, type_ }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCreationParams {
    pub client_id: String,
    pub auth_token: String,
    pub role: String,
    pub offer_sdp: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl RoomCreationParams {
    pub fn new(client_id: String, auth_token: String, role: String) -> Self {
        Self {
            client_id,
            auth_token,
            role,
            offer_sdp: None,
            metadata: None,
        }
    }

    pub fn with_offer_sdp(mut self, offer_sdp: String) -> Self {
        self.offer_sdp = Some(offer_sdp);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
} 