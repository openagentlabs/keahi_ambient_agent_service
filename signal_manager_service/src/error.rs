use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Message parsing error: {0}")]
    MessageParse(String),



    #[error("Session error: {0}")]
    Session(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Slice conversion error: {0}")]
    Slice(#[from] std::array::TryFromSliceError),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("Invalid payload type: {0}")]
    InvalidPayloadType(u8),

    #[error("Payload length mismatch: expected {expected}, got {actual}")]
    PayloadLengthMismatch { expected: usize, actual: usize },

    #[error("Client not found: {0}")]
    ClientNotFound(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Publish error: {0}")]
    PublishError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),

    #[error("Acknowledge error: {0}")]
    AcknowledgeError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),
} 