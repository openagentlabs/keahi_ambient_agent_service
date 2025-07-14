use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebRTCError {
    #[error("WebRTC initialization failed: {0}")]
    Initialization(String),
    
    #[error("Failed to create offer: {0}")]
    OfferCreation(String),
    
    #[error("Failed to set local description: {0}")]
    SetLocalDescription(String),
    
    #[error("Failed to add track: {0}")]
    AddTrack(String),
    
    #[error("Failed to close connection: {0}")]
    CloseConnection(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Media engine error: {0}")]
    MediaEngine(String),
    
    #[error("Peer connection error: {0}")]
    PeerConnection(String),
}

impl From<anyhow::Error> for WebRTCError {
    fn from(err: anyhow::Error) -> Self {
        WebRTCError::Initialization(err.to_string())
    }
}

impl From<webrtc::Error> for WebRTCError {
    fn from(err: webrtc::Error) -> Self {
        WebRTCError::PeerConnection(err.to_string())
    }
} 