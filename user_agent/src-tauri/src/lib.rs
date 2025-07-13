// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRTCConfig {
    pub stun_url: String,
    pub app_id: String,
    pub app_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SDPOffer {
    pub sdp: String,
    pub type_: String,
}

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

    pub async fn create_offer(&mut self) -> Result<SDPOffer> {
        // Create a new MediaEngine object to configure the supported codec
        let mut m = MediaEngine::default();
        m.register_default_codecs()?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_setting_engine(SettingEngine::default())
            .build();

        // Create the peer connection
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec![self.config.stun_url.clone()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // Create a video track
        let video_track = Arc::new(TrackLocalStaticSample::new(
            webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSampleOptions {
                track_id: "video".to_string(),
                stream_id: "video".to_string(),
                ..Default::default()
            },
        ));

        // Add the track to the peer connection
        peer_connection.add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>).await?;

        // Create the offer
        let offer = peer_connection.create_offer(None).await?;

        // Set the local description
        peer_connection.set_local_description(offer.clone()).await?;

        // Store the peer connection for later use
        self.peer_connection = Some(Arc::clone(&peer_connection));

        Ok(SDPOffer {
            sdp: offer.sdp,
            type_: offer.sdp_type.to_string(),
        })
    }

    pub async fn close(&mut self) -> Result<()> {
        if let Some(pc) = &self.peer_connection {
            pc.close().await?;
        }
        self.peer_connection = None;
        Ok(())
    }
}

// Tauri commands
#[tauri::command]
pub async fn generate_webrtc_offer() -> Result<SDPOffer, String> {
    let config = WebRTCConfig {
        stun_url: "stun:stun.cloudflare.com:3478".to_string(),
        app_id: "bffd14dc10f70248bbcf42d3c5ef4307".to_string(),
        app_secret: "98468ea69f92fc7cb75c436bbfb4155296f4a29a0dc0b642247a124dc328420a".to_string(),
    };

    let mut client = WebRTCClient::new(config);
    client.create_offer().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_webrtc_connection() -> Result<(), String> {
    let config = WebRTCConfig {
        stun_url: "stun:stun.cloudflare.com:3478".to_string(),
        app_id: "bffd14dc10f70248bbcf42d3c5ef4307".to_string(),
        app_secret: "98468ea69f92fc7cb75c436bbfb4155296f4a29a0dc0b642247a124dc328420a".to_string(),
    };

    let mut client = WebRTCClient::new(config);
    client.close().await.map_err(|e| e.to_string())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_webrtc_offer,
            cleanup_webrtc_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
