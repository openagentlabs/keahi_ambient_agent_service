use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::Result;

/// Configuration for event client connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// The topic name to connect to
    pub topic_name: String,
    /// The project ID (for GCP Pub/Sub)
    pub project_id: String,
    /// Additional configuration options
    pub options: std::collections::HashMap<String, String>,
}

impl EventConfig {
    /// Create an EventConfig from the application's GCP configuration
    pub fn from_app_config(gcp_config: &crate::config::GcpConfig) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("region".to_string(), gcp_config.region.clone());
        
        Self {
            topic_name: gcp_config.pubsub_topic.clone(),
            project_id: gcp_config.project_id.clone(),
            options,
        }
    }
}

/// Represents a message that can be sent or received
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    /// The message payload
    pub data: Vec<u8>,
    /// Optional message attributes
    pub attributes: std::collections::HashMap<String, String>,
    /// Optional message ID
    pub message_id: Option<String>,
    /// Optional ordering key
    pub ordering_key: Option<String>,
}

/// Result type for event operations
pub type EventResult<T> = Result<T>;

/// Trait that defines the interface for event clients
/// Each implementation should handle one topic only
#[async_trait]
pub trait EventClient: Send + Sync + Debug {
    /// Create a new event client instance for a specific topic
    async fn new(config: EventConfig) -> EventResult<Self>
    where
        Self: Sized;

    /// Send a message to the topic
    async fn send_message(&self, message: EventMessage) -> EventResult<String>;

    /// Send multiple messages to the topic
    async fn send_messages(&self, messages: Vec<EventMessage>) -> EventResult<Vec<String>>;

    /// Subscribe to receive messages from the topic
    /// Returns a receiver that can be used to receive messages
    async fn subscribe(&self) -> EventResult<Box<dyn EventReceiver>>;

    /// Check if the client is connected
    fn is_connected(&self) -> bool;

    /// Get the topic name this client is connected to
    fn topic_name(&self) -> &str;

    /// Get the project ID this client is connected to
    fn project_id(&self) -> &str;
}

/// Receiver for receiving messages from a topic
#[async_trait]
pub trait EventReceiver: Send + Sync {
    /// Receive a single message
    async fn receive_message(&mut self) -> EventResult<Option<EventMessage>>;

    /// Receive multiple messages (up to the specified limit)
    async fn receive_messages(&mut self, limit: usize) -> EventResult<Vec<EventMessage>>;

    /// Acknowledge a message (mark as processed)
    async fn acknowledge(&mut self, message_id: &str) -> EventResult<()>;

    /// Acknowledge multiple messages
    async fn acknowledge_messages(&mut self, message_ids: &[String]) -> EventResult<()>;

    /// Nack a message (mark as failed to process)
    async fn nack(&mut self, message_id: &str) -> EventResult<()>;

    /// Nack multiple messages
    async fn nack_messages(&mut self, message_ids: &[String]) -> EventResult<()>;
}

impl EventMessage {
    /// Create a new event message with the given data
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            attributes: std::collections::HashMap::new(),
            message_id: None,
            ordering_key: None,
        }
    }

    /// Create a new event message with data and attributes
    pub fn with_attributes(
        data: Vec<u8>,
        attributes: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            data,
            attributes,
            message_id: None,
            ordering_key: None,
        }
    }

    /// Add an attribute to the message
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Set the ordering key for the message
    pub fn with_ordering_key(mut self, ordering_key: String) -> Self {
        self.ordering_key = Some(ordering_key);
        self
    }

    /// Get the message data as a string
    pub fn data_as_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone()).map_err(|e| {
            crate::Error::MessageParse(format!("Failed to convert data to string: {}", e))
        })
    }

    /// Get the message data as JSON
    pub fn data_as_json<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_slice(&self.data).map_err(|e| {
            crate::Error::Serialization(e)
        })
    }
} 