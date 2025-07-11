use crate::events::{EventClient, EventConfig, EventMessage, GcpPubSubClient};
use tracing::{info, error};

/// Test function to demonstrate the events module
pub async fn test_events_module() -> crate::Result<()> {
    info!("Starting events module test...");

    // Create configuration for the event client from app config
    let app_config = crate::config::get_config();
    let config = EventConfig::from_app_config(&app_config.gcp);

    // Create a new GCP Pub/Sub client
    let client = match GcpPubSubClient::new(config).await {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Could not connect to GCP Pub/Sub: {}. Skipping integration test.", e);
            return Ok(()); // Skip test if cannot connect
        }
    };
    
    info!("Created GCP Pub/Sub client for topic: {}", client.topic_name());
    info!("Project ID: {}", client.project_id());
    info!("Connected: {}", client.is_connected());

    // Test sending a single message
    let message_data = serde_json::to_vec(&serde_json::json!({
        "event_type": "test_event",
        "client_id": "test_client_123",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "test_field": "test_value",
            "number": 42
        }
    }))?;

    let message = EventMessage::new(message_data)
        .with_attribute("event_type".to_string(), "test_event".to_string())
        .with_attribute("client_id".to_string(), "test_client_123".to_string())
        .with_ordering_key("test_client_123".to_string());

    let message_id = client.send_message(message).await?;
    info!("✅ Successfully sent message with ID: {}", message_id);

    // Test subscribing to receive messages
    let mut receiver = client.subscribe().await?;
    info!("✅ Successfully subscribed to topic");

    // Try to receive the message (may need to poll a few times due to Pub/Sub propagation)
    let mut received = None;
    for _ in 0..5 {
        match receiver.receive_message().await? {
            Some(msg) => {
                info!("📨 Received message with ID: {:?}", msg.message_id);
                if let Ok(message_str) = msg.data_as_string() {
                    info!("📄 Message data: {}", message_str);
                }
                // Acknowledge the message
                if let Some(msg_id) = &msg.message_id {
                    receiver.acknowledge(msg_id).await?;
                    info!("✅ Successfully acknowledged message");
                }
                received = Some(msg);
                break;
            }
            None => {
                info!("No message yet, retrying...");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
    assert!(received.is_some(), "Did not receive the published message from Pub/Sub");
    info!("✅ Events module integration test completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_events_module_integration() {
        // Initialize logging for tests
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .try_init();

        // Run the test
        let result = test_events_module().await;
        assert!(result.is_ok(), "Events module test failed: {:?}", result.err());
    }
} 