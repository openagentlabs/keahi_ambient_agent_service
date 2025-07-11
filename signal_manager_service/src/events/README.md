# Events Module

The events module provides a clean interface for connecting to message queues and event systems. It's designed to be extensible, allowing different implementations for various messaging platforms.

## Architecture

The module consists of three main components:

1. **Interface** (`interface.rs`) - Defines the contract that all event clients must implement
2. **GCP Pub/Sub Implementation** (`events_client_gcp.rs`) - Google Cloud Pub/Sub implementation
3. **Example Usage** (`example.rs`) - Demonstrates how to use the module

## Core Concepts

### EventClient Trait

The `EventClient` trait defines the interface that any messaging system implementation must follow:

```rust
#[async_trait]
pub trait EventClient: Send + Sync + Debug {
    fn new(config: EventConfig) -> EventResult<Self> where Self: Sized;
    async fn send_message(&self, message: EventMessage) -> EventResult<String>;
    async fn send_messages(&self, messages: Vec<EventMessage>) -> EventResult<Vec<String>>;
    async fn subscribe(&self) -> EventResult<Box<dyn EventReceiver>>;
    fn is_connected(&self) -> bool;
    fn topic_name(&self) -> &str;
    fn project_id(&self) -> &str;
}
```

### EventReceiver Trait

The `EventReceiver` trait defines how to receive and acknowledge messages:

```rust
#[async_trait]
pub trait EventReceiver: Send + Sync {
    async fn receive_message(&mut self) -> EventResult<Option<EventMessage>>;
    async fn receive_messages(&mut self, limit: usize) -> EventResult<Vec<EventMessage>>;
    async fn acknowledge(&mut self, message_id: &str) -> EventResult<()>;
    async fn acknowledge_messages(&mut self, message_ids: &[String]) -> EventResult<()>;
    async fn nack(&mut self, message_id: &str) -> EventResult<()>;
    async fn nack_messages(&mut self, message_ids: &[String]) -> EventResult<()>;
}
```

### EventMessage

Represents a message that can be sent or received:

```rust
pub struct EventMessage {
    pub data: Vec<u8>,
    pub attributes: HashMap<String, String>,
    pub message_id: Option<String>,
    pub ordering_key: Option<String>,
}
```

### EventConfig

Configuration for connecting to a topic:

```rust
pub struct EventConfig {
    pub topic_name: String,
    pub project_id: String,
    pub options: HashMap<String, String>,
}
```

## Usage

### Basic Usage with GCP Pub/Sub

```rust
use signal_manager_service::events::{EventClient, EventConfig, EventMessage, GcpPubSubClient};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let mut options = HashMap::new();
    options.insert("region".to_string(), "europe-west2".to_string());
    
    let config = EventConfig {
        topic_name: "my-topic".to_string(),
        project_id: "my-project-id".to_string(),
        options,
    };

    // Create client
    let client = GcpPubSubClient::new(config)?;
    
    // Send a message
    let message_data = serde_json::to_vec(&serde_json::json!({
        "event_type": "user_login",
        "user_id": "123",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))?;

    let message = EventMessage::new(message_data)
        .with_attribute("event_type".to_string(), "user_login".to_string())
        .with_attribute("user_id".to_string(), "123".to_string());

    let message_id = client.send_message(message).await?;
    println!("Sent message with ID: {}", message_id);

    // Subscribe to receive messages
    let mut receiver = client.subscribe().await?;
    
    // Receive messages
    while let Ok(Some(message)) = receiver.receive_message().await {
        if let Ok(message_str) = message.data_as_string() {
            println!("Received: {}", message_str);
        }
        
        // Acknowledge the message
        if let Some(message_id) = &message.message_id {
            receiver.acknowledge(message_id).await?;
        }
    }

    Ok(())
}
```

### Integration with WebSocket Server

The events module can be integrated with the WebSocket server to publish events when clients connect, disconnect, or send messages:

```rust
use signal_manager_service::events::{EventClient, EventConfig, EventMessage, GcpPubSubClient};

pub struct WebSocketServerWithEvents {
    websocket_server: WebSocketServer,
    events_client: GcpPubSubClient,
}

impl WebSocketServerWithEvents {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let websocket_server = WebSocketServer::new(config.clone())?;
        
        let events_config = EventConfig {
            topic_name: "websocket-events".to_string(),
            project_id: "my-project-id".to_string(),
            options: HashMap::new(),
        };
        
        let events_client = GcpPubSubClient::new(events_config)?;
        
        Ok(Self {
            websocket_server,
            events_client,
        })
    }
    
    pub async fn publish_client_connected(&self, client_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let event_data = serde_json::to_vec(&serde_json::json!({
            "event_type": "client_connected",
            "client_id": client_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?;
        
        let message = EventMessage::new(event_data)
            .with_attribute("event_type".to_string(), "client_connected".to_string())
            .with_attribute("client_id".to_string(), client_id.to_string());
            
        self.events_client.send_message(message).await?;
        Ok(())
    }
}
```

## Adding New Implementations

To add support for a new messaging system (e.g., Apache Kafka, RabbitMQ, AWS SQS), implement the `EventClient` and `EventReceiver` traits:

```rust
use crate::events::interface::{EventClient, EventConfig, EventMessage, EventReceiver, EventResult};
use async_trait::async_trait;

pub struct KafkaClient {
    // Kafka-specific fields
}

#[async_trait]
impl EventClient for KafkaClient {
    fn new(config: EventConfig) -> EventResult<Self> {
        // Initialize Kafka client
    }
    
    async fn send_message(&self, message: EventMessage) -> EventResult<String> {
        // Send message to Kafka
    }
    
    // Implement other required methods...
}

pub struct KafkaReceiver {
    // Kafka-specific fields
}

#[async_trait]
impl EventReceiver for KafkaReceiver {
    async fn receive_message(&mut self) -> EventResult<Option<EventMessage>> {
        // Receive message from Kafka
    }
    
    // Implement other required methods...
}
```

## Error Handling

The module uses the application's error system. Common error types include:

- `Connection` - Failed to connect to the messaging system
- `PublishError` - Failed to publish a message
- `ReceiveError` - Failed to receive a message
- `AcknowledgeError` - Failed to acknowledge a message
- `RuntimeError` - Runtime-related errors

## Configuration

The `EventConfig` struct allows for flexible configuration:

- `topic_name` - The topic/queue name to connect to
- `project_id` - The project ID (used by GCP Pub/Sub)
- `options` - Additional configuration options as key-value pairs

## Thread Safety

All implementations are designed to be thread-safe and can be shared across multiple threads. The `Send + Sync` bounds ensure this.

## Async Support

The module is built around async/await, making it suitable for high-performance applications that need to handle many concurrent connections. 