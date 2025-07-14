use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct WebRTCRoomCreatePayload {
    version: String,
    client_id: String,
    auth_token: String,
    role: String,
    offer_sdp: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let ws_url = "ws://localhost:8080/ws";
    info!("Testing WebRTC room creation...");
    info!("Connecting to: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .context("Failed to connect to WebSocket server")?;
    
    let (mut write, mut read) = ws_stream.split();
    
    // First register the client
    let client_id = format!("test_client_{}", Uuid::new_v4());
    let auth_token = "test_token";
    
    let register_payload = serde_json::json!({
        "version": "1.0",
        "client_id": client_id,
        "auth_token": auth_token,
        "capabilities": ["websocket"],
        "metadata": {"platform": "test", "version": "1.0"}
    });

    let register_message = serde_json::json!({
        "type": 2,
        "payload": {
            "type": "REGISTER",
            "data": register_payload
        }
    });
    
    info!("Sending REGISTER message...");
    write.send(Message::Text(serde_json::to_string(&register_message)?)).await?;
    
    // Wait for register response
    let register_timeout = Duration::from_secs(10);
    match timeout(register_timeout, read.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            info!("Received REGISTER response: {}", response);
        }
        _ => {
            error!("Failed to register client");
            return Ok(());
        }
    }
    
    // Now test WebRTC room creation
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    
    let webrtc_payload = WebRTCRoomCreatePayload {
        version: "1.0".to_string(),
        client_id: client_id.clone(),
        auth_token: auth_token.to_string(),
        role: "sender".to_string(),
        offer_sdp: Some(offer_sdp.to_string()),
        metadata: Some(serde_json::json!({"test": true})),
    };

    let webrtc_message = serde_json::json!({
        "type": 0x30, // WebRTCRoomCreate
        "payload": webrtc_payload
    });
    
    info!("Sending WebRTC room create message...");
    write.send(Message::Text(serde_json::to_string(&webrtc_message)?)).await?;
    
    // Wait for WebRTC response
    let webrtc_timeout = Duration::from_secs(15);
    match timeout(webrtc_timeout, read.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            info!("Received WebRTC response: {}", response);
        }
        Ok(Some(Ok(msg))) => {
            info!("Received message: {:?}", msg);
        }
        Ok(Some(Err(e))) => {
            error!("WebSocket error: {}", e);
        }
        Ok(None) => {
            error!("WebSocket connection closed");
        }
        Err(_) => {
            error!("Timeout waiting for WebRTC response");
        }
    }
    
    info!("Test completed!");
    Ok(())
} 