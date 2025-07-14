use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignalManagerError {
    #[error("WebSocket connection failed: {0}")]
    WebSocketConnection(String),
    
    #[error("WebSocket connect failed: {0}")]
    WebSocketConnect(String),
    
    #[error("WebSocket send failed: {0}")]
    WebSocketSend(String),
    
    #[error("WebSocket receive failed: {0}")]
    WebSocketReceive(String),
    
    #[error("Message serialization failed: {0}")]
    Serialization(String),
    
    #[error("Message deserialization failed: {0}")]
    Deserialization(String),
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error("Connection timeout: {0}")]
    Timeout(String),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Registration failed: {0}")]
    Registration(String),
    
    #[error("Room creation failed: {0}")]
    RoomCreation(String),
    
    #[error("Not connected")]
    NotConnected,
    
    #[error("Already connected")]
    AlreadyConnected,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for SignalManagerError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        SignalManagerError::WebSocketConnection(err.to_string())
    }
}

impl From<serde_json::Error> for SignalManagerError {
    fn from(err: serde_json::Error) -> Self {
        SignalManagerError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for SignalManagerError {
    fn from(err: std::io::Error) -> Self {
        SignalManagerError::WebSocketConnection(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for SignalManagerError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SignalManagerError::Serialization(err.to_string())
    }
} 