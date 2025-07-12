use crate::config::Config;
use crate::cloudflare::models::*;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use async_trait::async_trait;

#[async_trait]
pub trait CloudflareClientTrait: Send + Sync {
    async fn create_session(&self, offer_sdp: String) -> Result<CloudflareSessionResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn add_tracks(&self, session_id: &str, tracks: Vec<Track>, offer_sdp: Option<String>) -> Result<CloudflareTracksResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn send_answer_sdp(&self, session_id: &str, answer_sdp: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn terminate_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_session(&self, session_id: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
    async fn validate_credentials(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

/// Cloudflare Realtime API client
pub struct CloudflareClient {
    app_id: String,
    app_secret: String,
    base_url: String,
    http_client: Client,
    _config: Arc<Config>,
}

#[async_trait]
impl CloudflareClientTrait for CloudflareClient {
    async fn create_session(&self, offer_sdp: String) -> Result<CloudflareSessionResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.create_session_impl(offer_sdp).await
    }

    async fn add_tracks(&self, session_id: &str, tracks: Vec<Track>, offer_sdp: Option<String>) -> Result<CloudflareTracksResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.add_tracks_impl(session_id, tracks, offer_sdp).await
    }

    async fn send_answer_sdp(&self, session_id: &str, answer_sdp: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send_answer_sdp_impl(session_id, answer_sdp).await
    }

    async fn terminate_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.terminate_session_impl(session_id).await
    }

    async fn get_session(&self, session_id: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.get_session_impl(session_id).await
    }

    async fn validate_credentials(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.validate_credentials_impl().await
    }
}

impl CloudflareClient {
    /// Create a new Cloudflare client
    pub fn new(config: Arc<Config>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let app_id = config.cloudflare.app_id.clone();
        let app_secret = config.cloudflare.app_secret.clone();
        let base_url = config.cloudflare.base_url.clone();

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            app_id,
            app_secret,
            base_url,
            http_client,
            _config: config,
        })
    }

    /// Create a new WebRTC session with Cloudflare
    async fn create_session_impl(&self, offer_sdp: String) -> Result<CloudflareSessionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apps/{}/sessions/new", self.base_url, self.app_id);
        
        let body = serde_json::json!({
            "sessionDescription": {
                "type": "offer",
                "sdp": offer_sdp
            }
        });

        debug!("Creating Cloudflare session with URL: {}", url);
        
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.app_secret))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Cloudflare session creation failed: {}", error_text);
            return Err(format!("Cloudflare API error: {error_text}").into());
        }

        let result: CloudflareSessionResponse = response.json().await?;
        info!("Created Cloudflare session: {}", result.session_id);
        
        Ok(result)
    }

    /// Add tracks to an existing session
    async fn add_tracks_impl(
        &self, 
        session_id: &str, 
        tracks: Vec<Track>, 
        offer_sdp: Option<String>
    ) -> Result<CloudflareTracksResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apps/{}/sessions/{}/tracks/new", self.base_url, self.app_id, session_id);
        
        let mut body = serde_json::json!({
            "tracks": tracks
        });

        if let Some(sdp) = offer_sdp {
            body["sessionDescription"] = serde_json::json!({
                "type": "offer",
                "sdp": sdp
            });
        }

        debug!("Adding tracks to session {} with URL: {}", session_id, url);
        
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.app_secret))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Cloudflare track addition failed: {}", error_text);
            return Err(format!("Cloudflare API error: {error_text}").into());
        }

        let result: CloudflareTracksResponse = response.json().await?;
        info!("Added tracks to session: {}", session_id);
        
        Ok(result)
    }

    /// Send answer SDP for renegotiation
    async fn send_answer_sdp_impl(
        &self, 
        session_id: &str, 
        answer_sdp: String
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apps/{}/sessions/{}/renegotiate", self.base_url, self.app_id, session_id);
        
        let body = serde_json::json!({
            "sessionDescription": {
                "type": "answer",
                "sdp": answer_sdp
            }
        });

        debug!("Sending answer SDP to session {} with URL: {}", session_id, url);
        
        let response = self.http_client
            .put(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.app_secret))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Cloudflare answer SDP failed: {}", error_text);
            return Err(format!("Cloudflare API error: {error_text}").into());
        }

        info!("Sent answer SDP to session: {}", session_id);
        Ok(())
    }

    /// Terminate a session
    async fn terminate_session_impl(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apps/{}/sessions/{}", self.base_url, self.app_id, session_id);
        
        debug!("Terminating session {} with URL: {}", session_id, url);
        
        let response = self.http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.app_secret))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            warn!("Cloudflare session termination failed: {}", error_text);
            // Don't return error for termination failures as session might already be gone
        }

        info!("Terminated session: {}", session_id);
        Ok(())
    }

    /// Get session information
    async fn get_session_impl(&self, session_id: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apps/{}/sessions/{}", self.base_url, self.app_id, session_id);
        
        debug!("Getting session info for {} with URL: {}", session_id, url);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.app_secret))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Cloudflare get session failed: {}", error_text);
            return Err(format!("Cloudflare API error: {error_text}").into());
        }

        let result: Value = response.json().await?;
        Ok(result)
    }

    /// Validate Cloudflare credentials
    async fn validate_credentials_impl(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apps/{}", self.base_url, self.app_id);
        
        debug!("Validating Cloudflare credentials with URL: {}", url);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.app_secret))
            .send()
            .await?;

        let is_valid = response.status().is_success();
        
        if is_valid {
            info!("Cloudflare credentials validated successfully");
        } else {
            error!("Cloudflare credentials validation failed: {}", response.status());
        }
        
        Ok(is_valid)
    }
} 