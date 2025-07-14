use std::sync::Arc;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use crate::webrtcclient::config::WebRTCConfig;
use crate::webrtcclient::error::WebRTCError;
use crate::webrtcclient::types::{SDPOffer, RoomCreationParams};
use log::{debug, error};

pub struct WebRTCClient {
    config: WebRTCConfig,
    peer_connection: Option<Arc<RTCPeerConnection>>,
}

impl WebRTCClient {
    pub fn new(config: WebRTCConfig) -> Self {
        Self { 
            config,
            peer_connection: None,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(WebRTCConfig::default())
    }

    /// Creates a WebRTC offer for room creation
    pub async fn create_offer(&mut self) -> Result<SDPOffer, WebRTCError> {
        debug!("Creating WebRTC offer");
        // Create a new MediaEngine object to configure the supported codec
        let mut m = MediaEngine::default();
        m.register_default_codecs()
            .map_err(|e| {
                error!("MediaEngine codec registration failed: {e}");
                WebRTCError::MediaEngine(e.to_string())
            })?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_setting_engine(SettingEngine::default())
            .build();

        // Create the peer connection with STUN server
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec![self.config.stun_url.clone()],
                ..Default::default()
            }],
            ..Default::default()
        };
        debug!("Creating new peer connection with STUN URL: {}", self.config.stun_url);
        let peer_connection = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| {
                    error!("PeerConnection creation failed: {e}");
                    WebRTCError::PeerConnection(e.to_string())
                })?
        );

        // Create a video track with VP8 codec
        let codec = RTCRtpCodecCapability {
            mime_type: MIME_TYPE_VP8.to_owned(),
            ..Default::default()
        };
        
        let video_track = Arc::new(TrackLocalStaticSample::new(
            codec,
            "video".to_string(),
            "video".to_string(),
        ));
        debug!("Adding video track to peer connection");
        // Add the track to the peer connection
        peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| {
                error!("AddTrack failed: {e}");
                WebRTCError::AddTrack(e.to_string())
            })?;

        // Create the offer
        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| {
                error!("Offer creation failed: {e}");
                WebRTCError::OfferCreation(e.to_string())
            })?;

        // Set the local description
        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| {
                error!("SetLocalDescription failed: {e}");
                WebRTCError::SetLocalDescription(e.to_string())
            })?;

        // Store the peer connection for later use
        self.peer_connection = Some(Arc::clone(&peer_connection));
        debug!("WebRTC offer created successfully");
        Ok(SDPOffer::new(offer.sdp, offer.sdp_type.to_string()))
    }

    /// Prepares room creation parameters with WebRTC offer
    pub async fn prepare_room_creation(
        &mut self,
        client_id: String,
        auth_token: String,
        role: String,
    ) -> Result<RoomCreationParams, WebRTCError> {
        // Generate WebRTC offer
        let offer = self.create_offer().await?;
        
        // Create room parameters
        let mut params = RoomCreationParams::new(client_id, auth_token, role);
        params = params.with_offer_sdp(offer.sdp);
        
        // Add metadata
        let metadata = serde_json::json!({
            "webrtc_offer_type": offer.type_,
            "timestamp": chrono::Utc::now().timestamp(),
            "app_id": self.config.app_id,
            "stun_url": self.config.stun_url,
        });
        params = params.with_metadata(metadata);
        
        Ok(params)
    }

    /// Closes the WebRTC connection
    pub async fn close(&mut self) -> Result<(), WebRTCError> {
        debug!("Closing WebRTC connection");
        if let Some(pc) = &self.peer_connection {
            pc.close()
                .await
                .map_err(|e| {
                    error!("CloseConnection failed: {e}");
                    WebRTCError::CloseConnection(e.to_string())
                })?;
        }
        self.peer_connection = None;
        debug!("WebRTC connection closed");
        Ok(())
    }

    /// Gets the current configuration
    pub fn get_config(&self) -> &WebRTCConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn update_config(&mut self, config: WebRTCConfig) {
        self.config = config;
    }
} 