use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalManagerConfig {
    pub url: String,
    pub port: u16,
    pub client_id: String,
    pub auth_token: String,
    pub version: String,
    pub heartbeat_interval: u64,
    pub timeout: u64,
    pub command_timeout: u64,
    pub reconnect_attempts: u32,
    pub reconnect_delay: u64,
}

impl Default for SignalManagerConfig {
    fn default() -> Self {
        Self {
            url: "127.0.0.1".to_string(),
            port: 8080,
            client_id: "user_agent_client".to_string(),
            auth_token: "test_token_1".to_string(),
            version: "1.0.0".to_string(),
            heartbeat_interval: 5,
            timeout: 10,
            command_timeout: 15,
            reconnect_attempts: 5,
            reconnect_delay: 1,
        }
    }
}

impl SignalManagerConfig {
    pub fn new(
        url: String,
        port: u16,
        client_id: String,
        auth_token: String,
    ) -> Self {
        Self {
            url,
            port,
            client_id,
            auth_token,
            version: "1.0.0".to_string(),
            heartbeat_interval: 5,
            timeout: 10,
            command_timeout: 15,
            reconnect_attempts: 5,
            reconnect_delay: 1,
        }
    }

    pub fn websocket_url(&self) -> String {
        format!("ws://{}:{}", self.url, self.port)
    }
} 