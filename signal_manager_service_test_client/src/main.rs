use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct RegisterPayload {
    client_id: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterResponse {
    status: u16,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonResponse {
    status: u16,
    message: String,
}

async fn test_ping(ws_url: &str) -> Result<()> {
    info!("Testing Ping functionality...");
    
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .context("Failed to connect to WebSocket server")?;
    
    let (mut write, mut read) = ws_stream.split();
    
    // Send ping message
    let ping_message = "PING";
    info!("Sending ping: {}", ping_message);
    
    write.send(Message::Text(ping_message.to_string()))
        .await
        .context("Failed to send ping message")?;
    
    // Wait for pong response with timeout
    let pong_timeout = Duration::from_secs(5);
    match timeout(pong_timeout, read.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            info!("Received pong response: {}", response);
            if response == "PONG" {
                info!("✅ Ping test passed!");
                Ok(())
            } else {
                anyhow::bail!("Unexpected pong response: {}", response);
            }
        }
        Ok(Some(Ok(msg))) => {
            anyhow::bail!("Unexpected message type: {:?}", msg);
        }
        Ok(Some(Err(e))) => {
            anyhow::bail!("WebSocket error: {}", e);
        }
        Ok(None) => {
            anyhow::bail!("WebSocket connection closed unexpectedly");
        }
        Err(_) => {
            anyhow::bail!("Timeout waiting for pong response");
        }
    }
}

async fn test_register_json(ws_url: &str) -> Result<()> {
    info!("Testing JSON REGISTER functionality...");
    
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .context("Failed to connect to WebSocket server")?;
    
    let (mut write, mut read) = ws_stream.split();
    
    // Create REGISTER payload
    let frame_id = Uuid::new_v4().to_string();
    let payload = RegisterPayload {
        client_id: format!("test_client_{}", Uuid::new_v4()),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    let json_message = serde_json::json!({
        "frame_id": frame_id,
        "type": 2,
        "payload": {
            "type": "REGISTER",
            "data": payload
        }
    });
    
    let message_text = serde_json::to_string(&json_message)
        .context("Failed to serialize JSON message")?;
    
    info!("Sending JSON REGISTER message: {}", message_text);
    
    write.send(Message::Text(message_text))
        .await
        .context("Failed to send JSON message")?;
    
    // Wait for JSON response with timeout
    let json_timeout = Duration::from_secs(10);
    match timeout(json_timeout, read.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            info!("Received JSON response: {}", response);
            
            // Parse the response
            let response_json: serde_json::Value = serde_json::from_str(&response)
                .context("Failed to parse JSON response")?;
            
            // Verify response structure
            if let Some(status) = response_json.get("status").and_then(|s| s.as_u64()) {
                if status == 200 {
                    info!("✅ JSON REGISTER test passed! Status: {}", status);
                    Ok(())
                } else {
                    anyhow::bail!("JSON REGISTER failed with status: {}", status);
                }
            } else {
                anyhow::bail!("Invalid JSON response format: {}", response);
            }
        }
        Ok(Some(Ok(msg))) => {
            anyhow::bail!("Unexpected message type: {:?}", msg);
        }
        Ok(Some(Ok(Message::Close(_)))) => {
            anyhow::bail!("WebSocket connection closed by server");
        }
        Ok(Some(Err(e))) => {
            anyhow::bail!("WebSocket error: {}", e);
        }
        Ok(None) => {
            anyhow::bail!("WebSocket connection closed unexpectedly");
        }
        Err(_) => {
            anyhow::bail!("Timeout waiting for JSON response");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let ws_url = std::env::var("WS_URL").unwrap_or_else(|_| {
        "ws://localhost:8080/ws".to_string()
    });
    
    info!("Starting WebSocket test client");
    info!("Connecting to: {}", ws_url);
    
    // Test Ping functionality
    match test_ping(&ws_url).await {
        Ok(_) => info!("Ping test completed successfully"),
        Err(e) => {
            error!("Ping test failed: {}", e);
            return Err(e);
        }
    }
    
    // Test JSON REGISTER functionality
    match test_register_json(&ws_url).await {
        Ok(_) => info!("JSON REGISTER test completed successfully"),
        Err(e) => {
            error!("JSON REGISTER test failed: {}", e);
            return Err(e);
        }
    }
    
    info!("All tests completed successfully!");
    Ok(())
} 