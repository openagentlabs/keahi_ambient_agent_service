use crate::config::Config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

pub struct AuthManager {
    config: Arc<Config>,
    // In a real implementation, this would be replaced with a proper token store
    // or integration with an authentication service
    valid_tokens: Arc<RwLock<HashMap<String, String>>>,
}

impl AuthManager {
    pub fn new(config: Arc<Config>) -> Self {
        let mut valid_tokens = HashMap::new();
        
        // Load tokens from configuration
        let api_keys = config.parse_api_keys();
        for (client_id, token) in api_keys {
            valid_tokens.insert(client_id, token);
        }
        
        // For development/testing, add some sample tokens if none configured
        if valid_tokens.is_empty() {
            valid_tokens.insert("test_client_1".to_string(), "test_token_1".to_string());
            valid_tokens.insert("test_client_2".to_string(), "test_token_2".to_string());
        }
        
        Self {
            config,
            valid_tokens: Arc::new(RwLock::new(valid_tokens)),
        }
    }

    pub async fn authenticate(&self, client_id: &str, auth_token: &str) -> Result<bool, crate::Error> {
        debug!("Authenticating client: {} with method: {}", client_id, self.config.auth.auth_method);
        
        match self.config.auth.auth_method.as_str() {
            "token" => self.authenticate_with_token(client_id, auth_token).await,
            "api_key" => self.authenticate_with_api_key(client_id, auth_token).await,

            _ => {
                warn!("Unknown authentication method: {}", self.config.auth.auth_method);
                Ok(false)
            }
        }
    }

    async fn authenticate_with_token(&self, client_id: &str, auth_token: &str) -> Result<bool, crate::Error> {
        let tokens = self.valid_tokens.read().await;
        
        if let Some(expected_token) = tokens.get(client_id) {
            if expected_token == auth_token {
                debug!("Token authentication successful for client: {}", client_id);
                return Ok(true);
            } else {
                warn!("Invalid token for client: {}", client_id);
                return Ok(false);
            }
        }
        
        warn!("Unknown client: {}", client_id);
        Ok(false)
    }

    async fn authenticate_with_api_key(&self, client_id: &str, api_key: &str) -> Result<bool, crate::Error> {
        // API key authentication - same as token for now
        self.authenticate_with_token(client_id, api_key).await
    }



    pub async fn add_valid_token(&self, client_id: String, token: String) {
        let mut tokens = self.valid_tokens.write().await;
        tokens.insert(client_id, token);
    }

    pub async fn remove_token(&self, client_id: &str) {
        let mut tokens = self.valid_tokens.write().await;
        tokens.remove(client_id);
    }

    pub async fn validate_session(&self, client_id: &str, _session_id: &str) -> Result<bool, crate::Error> {
        // For now, we'll just check if the client exists
        let tokens = self.valid_tokens.read().await;
        Ok(tokens.contains_key(client_id))
    }

    pub fn get_auth_method(&self) -> &str {
        &self.config.auth.auth_method
    }
} 