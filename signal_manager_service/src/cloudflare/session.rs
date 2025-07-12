use crate::cloudflare::{CloudflareClient, CloudflareClientTrait, models::*};
use crate::config::Config;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::Utc;

/// WebRTC session manager
pub struct CloudflareSession {
    client: Box<dyn CloudflareClientTrait>,
    config: Arc<Config>,
}

impl CloudflareSession {
    /// Create a new session manager
    pub fn new(config: Arc<Config>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = CloudflareClient::new(config.clone())?;
        
        Ok(Self {
            client: Box::new(client),
            config,
        })
    }

    /// Create a new session manager with a custom client (for testing)
    #[cfg(test)]
    pub fn new_with_client(config: Arc<Config>, client: Box<dyn CloudflareClientTrait>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            client,
            config,
        })
    }

    /// Create a new WebRTC room with sender session
    pub async fn create_room_with_sender(
        &self,
        room_id: &str,
        client_id: &str,
        offer_sdp: String,
    ) -> Result<WebRTCConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Creating room {} with sender {}", room_id, client_id);

        // Create Cloudflare session
        let session_response = self.client.create_session(offer_sdp).await?;
        
        let connection_info = WebRTCConnectionInfo {
            room_id: room_id.to_string(),
            role: ClientRole::Sender,
            app_id: self.config.cloudflare.app_id.clone(),
            session_id: Some(session_response.session_id.clone()),
            status: ConnectionStatus::Connecting,
            metadata: serde_json::json!({
                "session_id": session_response.session_id,
                "created_at": Utc::now(),
                "client_id": client_id,
            }),
        };

        info!("Created room {} with sender session {}", room_id, session_response.session_id);
        
        Ok(connection_info)
    }

    /// Join a room as receiver
    pub async fn join_room_as_receiver(
        &self,
        room_id: &str,
        client_id: &str,
        sender_session_id: &str,
    ) -> Result<WebRTCConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Joining room {} as receiver {}", room_id, client_id);

        // For receiver, we need to get tracks from the sender's session
        let tracks = vec![
            Track {
                location: "remote".to_string(),
                mid: None,
                track_name: "video".to_string(),
                session_id: Some(sender_session_id.to_string()),
            },
            Track {
                location: "remote".to_string(),
                mid: None,
                track_name: "audio".to_string(),
                session_id: Some(sender_session_id.to_string()),
            },
        ];

        // Add tracks to get the receiver session
        let tracks_response = self.client.add_tracks(sender_session_id, tracks, None).await?;
        
        let connection_info = WebRTCConnectionInfo {
            room_id: room_id.to_string(),
            role: ClientRole::Receiver,
            app_id: self.config.cloudflare.app_id.clone(),
            session_id: Some(sender_session_id.to_string()), // Same session for receiver
            status: ConnectionStatus::Connecting,
            metadata: serde_json::json!({
                "sender_session_id": sender_session_id,
                "joined_at": Utc::now(),
                "client_id": client_id,
                "requires_renegotiation": tracks_response.requires_immediate_renegotiation,
            }),
        };

        info!("Joined room {} as receiver with session {}", room_id, sender_session_id);
        
        Ok(connection_info)
    }

    /// Add tracks to an existing session
    pub async fn add_tracks_to_session(
        &self,
        session_id: &str,
        tracks: Vec<Track>,
        offer_sdp: Option<String>,
    ) -> Result<CloudflareTracksResponse, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Adding tracks to session {}", session_id);
        let tracks_len = tracks.len();
        let response = self.client.add_tracks(session_id, tracks, offer_sdp).await?;
        info!("Added {} tracks to session {}", tracks_len, session_id);
        Ok(response)
    }

    /// Send answer SDP for renegotiation
    pub async fn send_answer_sdp(
        &self,
        session_id: &str,
        answer_sdp: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Sending answer SDP to session {}", session_id);
        
        self.client.send_answer_sdp(session_id, answer_sdp).await?;
        
        info!("Sent answer SDP to session {}", session_id);
        
        Ok(())
    }

    /// Terminate a session and cleanup
    pub async fn terminate_session(
        &self,
        session_id: &str,
        room_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Terminating session {} for room {}", session_id, room_id);
        
        // Terminate the Cloudflare session
        if let Err(e) = self.client.terminate_session(session_id).await {
            warn!("Failed to terminate Cloudflare session {}: {}", session_id, e);
        }
        
        info!("Terminated session {} for room {}", session_id, room_id);
        
        Ok(())
    }

    /// Get session information
    pub async fn get_session_info(
        &self,
        session_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Getting session info for {}", session_id);
        
        let info = self.client.get_session(session_id).await?;
        
        Ok(info)
    }

    /// Validate Cloudflare credentials
    pub async fn validate_credentials(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Validating Cloudflare credentials");
        
        let is_valid = self.client.validate_credentials().await?;
        
        if is_valid {
            info!("Cloudflare credentials are valid");
        } else {
            error!("Cloudflare credentials are invalid");
        }
        
        Ok(is_valid)
    }

    /// Generate a new room UUID
    pub fn generate_room_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Create connection info for a client
    pub fn create_connection_info(
        &self,
        room_id: &str,
        role: ClientRole,
        session_id: Option<String>,
    ) -> WebRTCConnectionInfo {
        WebRTCConnectionInfo {
            room_id: room_id.to_string(),
            role,
            app_id: self.config.cloudflare.app_id.clone(),
            session_id,
            status: ConnectionStatus::Disconnected,
            metadata: serde_json::json!({
                "created_at": Utc::now(),
            }),
        }
    }
} 