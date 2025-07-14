use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCConfig {
    pub stun_url: String,
    pub app_id: String,
    pub app_secret: String,
}

impl Default for WebRTCConfig {
    fn default() -> Self {
        Self {
            stun_url: "stun:stun.cloudflare.com:3478".to_string(),
            app_id: "bffd14dc10f70248bbcf42d3c5ef4307".to_string(),
            app_secret: "98468ea69f92fc7cb75c436bbfb4155296f4a29a0dc0b642247a124dc328420a".to_string(),
        }
    }
}

impl WebRTCConfig {
    pub fn new(stun_url: String, app_id: String, app_secret: String) -> Self {
        Self {
            stun_url,
            app_id,
            app_secret,
        }
    }
} 