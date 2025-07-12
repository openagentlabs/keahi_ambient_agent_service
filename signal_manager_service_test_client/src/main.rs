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
    version: String,
    client_id: String,
    auth_token: String,
    capabilities: Option<Vec<String>>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnregisterPayload {
    version: String,
    client_id: String,
    auth_token: String,
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

async fn test_register(ws_url: &str, client_id: &str, auth_token: &str) -> Result<()> {
    info!("Testing REGISTER functionality...");
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .context("Failed to connect to WebSocket server")?;
    let (mut write, mut read) = ws_stream.split();

    let payload = RegisterPayload {
        version: "1.0".to_string(),
        client_id: client_id.to_string(),
        auth_token: auth_token.to_string(),
        capabilities: Some(vec!["websocket".to_string()]),
        metadata: Some(serde_json::json!({"platform": "test", "version": "1.0"})),
    };

    let json_message = serde_json::json!({
        "type": 2,
        "payload": {
            "type": "REGISTER",
            "data": payload
        }
    });
    let message_text = serde_json::to_string(&json_message)?;
    info!("Sending REGISTER message: {}", message_text);
    write.send(Message::Text(message_text)).await?;

    let json_timeout = Duration::from_secs(10);
    match timeout(json_timeout, read.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            info!("Received REGISTER response: {}", response);
            let response_json: serde_json::Value = serde_json::from_str(&response)?;
            if let Some(status) = response_json.get("status").and_then(|s| s.as_u64()) {
                if status == 200 {
                    info!("✅ REGISTER test passed! Status: {}", status);
                    Ok(())
                } else {
                    anyhow::bail!("REGISTER failed with status: {}", status);
                }
            } else {
                anyhow::bail!("Invalid REGISTER response format: {}", response);
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
            anyhow::bail!("Timeout waiting for REGISTER response");
        }
    }
}

async fn test_unregister(ws_url: &str, client_id: &str, auth_token: &str) -> Result<()> {
    info!("Testing UNREGISTER functionality...");
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .context("Failed to connect to WebSocket server")?;
    let (mut write, mut read) = ws_stream.split();

    let payload = UnregisterPayload {
        version: "1.0".to_string(),
        client_id: client_id.to_string(),
        auth_token: auth_token.to_string(),
    };

    let json_message = serde_json::json!({
        "type": 2,
        "payload": {
            "type": "UNREGISTER",
            "data": payload
        }
    });
    let message_text = serde_json::to_string(&json_message)?;
    info!("Sending UNREGISTER message: {}", message_text);
    write.send(Message::Text(message_text)).await?;

    let json_timeout = Duration::from_secs(10);
    match timeout(json_timeout, read.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            info!("Received UNREGISTER response: {}", response);
            let response_json: serde_json::Value = serde_json::from_str(&response)?;
            if let Some(status) = response_json.get("status").and_then(|s| s.as_u64()) {
                if status == 200 {
                    info!("✅ UNREGISTER test passed! Status: {}", status);
                    Ok(())
                } else {
                    anyhow::bail!("UNREGISTER failed with status: {}", status);
                }
            } else {
                anyhow::bail!("Invalid UNREGISTER response format: {}", response);
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
            anyhow::bail!("Timeout waiting for UNREGISTER response");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let ws_url = std::env::var("WS_URL").unwrap_or_else(|_| {
        "ws://localhost:8080/ws".to_string()
    });
    info!("Starting WebSocket test client");
    info!("Connecting to: {}", ws_url);

    // Test Ping functionality
    test_ping(&ws_url).await?;

    // Use a fixed client_id and auth_token for register/unregister
    let client_id = format!("test_client_{}", Uuid::new_v4());
    let auth_token = "test_token";

    // Test REGISTER functionality
    test_register(&ws_url, &client_id, auth_token).await?;

    // Test UNREGISTER functionality
    test_unregister(&ws_url, &client_id, auth_token).await?;

    info!("All tests completed successfully!");
    Ok(())
} 