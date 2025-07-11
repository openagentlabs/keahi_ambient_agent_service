use crate::events::{EventClient, EventConfig, EventMessage, GcpPubSubClient};
use tracing::{info, error};

/// Example demonstrating how to use the events module with GCP Pub/Sub
pub async fn example_usage() -> crate::Result<()> {
    // Create configuration for the event client from app config
    let app_config = crate::config::get_config();
    let config = EventConfig::from_app_config(&app_config.gcp);

    // Create a new GCP Pub/Sub client
    let client = GcpPubSubClient::new(config).await?;
    
    info!("Created GCP Pub/Sub client for topic: {}", client.topic_name());
    info!("Project ID: {}", client.project_id());
    info!("Connected: {}", client.is_connected());

    // Example: Send a message
    let message_data = serde_json::to_vec(&serde_json::json!({
        "event_type": "client_connected",
        "client_id": "test_client_123",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "ip_address": "192.168.1.100",
            "user_agent": "Mozilla/5.0..."
        }
    }))?;

    let message = EventMessage::new(message_data)
        .with_attribute("event_type".to_string(), "client_connected".to_string())
        .with_attribute("client_id".to_string(), "test_client_123".to_string())
        .with_ordering_key("test_client_123".to_string());

    match client.send_message(message).await {
        Ok(message_id) => {
            info!("Successfully sent message with ID: {}", message_id);
        }
        Err(e) => {
            error!("Failed to send message: {}", e);
            return Err(e);
        }
    }

    // Example: Send multiple messages
    let messages = vec![
        EventMessage::new(serde_json::to_vec(&serde_json::json!({
            "event_type": "message_received",
            "client_id": "test_client_123",
            "message": "Hello, world!"
        }))?),
        EventMessage::new(serde_json::to_vec(&serde_json::json!({
            "event_type": "client_disconnected",
            "client_id": "test_client_123",
            "reason": "timeout"
        }))?),
    ];

    match client.send_messages(messages).await {
        Ok(message_ids) => {
            info!("Successfully sent {} messages: {:?}", message_ids.len(), message_ids);
        }
        Err(e) => {
            error!("Failed to send messages: {}", e);
            return Err(e);
        }
    }

    // Example: Subscribe to receive messages
    match client.subscribe().await {
        Ok(mut receiver) => {
            info!("Successfully subscribed to topic");
            
            // Receive a single message
            match receiver.receive_message().await {
                Ok(Some(message)) => {
                    info!("Received message with ID: {:?}", message.message_id);
                    if let Ok(message_str) = message.data_as_string() {
                        info!("Message data: {}", message_str);
                    }
                    
                    // Acknowledge the message
                    if let Some(message_id) = &message.message_id {
                        if let Err(e) = receiver.acknowledge(message_id).await {
                            error!("Failed to acknowledge message: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    info!("No messages available");
                }
                Err(e) => {
                    error!("Failed to receive message: {}", e);
                }
            }

            // Receive multiple messages
            match receiver.receive_messages(10).await {
                Ok(messages) => {
                    info!("Received {} messages", messages.len());
                    for message in messages {
                        if let Ok(message_str) = message.data_as_string() {
                            info!("Message: {}", message_str);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive messages: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to subscribe: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Example of how to integrate events with the WebSocket server
pub async fn integrate_with_websocket_server() -> crate::Result<()> {
    // This would be called from the WebSocket server when events need to be published
    // For example, when a client connects, disconnects, or sends a message
    
    let app_config = crate::config::get_config();
    let config = EventConfig::from_app_config(&app_config.gcp);

    let client = GcpPubSubClient::new(config).await?;
    
    // Example: Publish client connection event
    let connection_event = EventMessage::new(serde_json::to_vec(&serde_json::json!({
        "event_type": "client_connected",
        "client_id": "websocket_client_456",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "server_info": {
            "host": "127.0.0.1",
            "port": 8080
        }
    }))?);

    if let Err(e) = client.send_message(connection_event).await {
        error!("Failed to publish connection event: {}", e);
    }

    Ok(())
} 