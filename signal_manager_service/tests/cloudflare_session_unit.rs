use signal_manager_service::config::Config;
use signal_manager_service::cloudflare::{CloudflareSession, CloudflareClientTrait, models::*};
use std::sync::Arc;
use mockall::predicate::*;
use mockall::*;
use async_trait::async_trait;
use serde_json::Value;

// Mock CloudflareClientTrait for testing
mock! {
    pub MockCloudflareClient {}
    
    #[async_trait]
    impl CloudflareClientTrait for MockCloudflareClient {
        async fn create_session(&self, offer_sdp: String) -> Result<CloudflareSessionResponse, Box<dyn std::error::Error + Send + Sync>>;
        async fn add_tracks(&self, session_id: &str, tracks: Vec<Track>, offer_sdp: Option<String>) -> Result<CloudflareTracksResponse, Box<dyn std::error::Error + Send + Sync>>;
        async fn send_answer_sdp(&self, session_id: &str, answer_sdp: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
        async fn terminate_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
        async fn get_session(&self, session_id: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
        async fn validate_credentials(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    }
}

#[tokio::test]
async fn test_cloudflare_config_default() {
    // Test that we can create a default config
    let config = Config::default();
    assert_eq!(config.cloudflare.app_id, "your-cloudflare-app-id");
    assert_eq!(config.cloudflare.base_url, "https://rtc.live.cloudflare.com/v1");
    assert_eq!(config.cloudflare.stun_url, "stun:stun.cloudflare.com:3478");
}

#[tokio::test]
async fn test_cloudflare_config_custom() {
    // Test that we can create a custom config
    let mut config = Config::default();
    config.cloudflare.app_id = "test_app_id".to_string();
    config.cloudflare.app_secret = "test_app_secret".to_string();
    config.cloudflare.base_url = "https://rtc.live.cloudflare.com/v1".to_string();
    
    assert_eq!(config.cloudflare.app_id, "test_app_id");
    assert_eq!(config.cloudflare.app_secret, "test_app_secret");
    assert_eq!(config.cloudflare.base_url, "https://rtc.live.cloudflare.com/v1");
}

#[tokio::test]
async fn test_cloudflare_session_creation() {
    let config = Arc::new(Config::default());
    let session = CloudflareSession::new(config);
    assert!(session.is_ok());
}

#[tokio::test]
async fn test_cloudflare_session_with_mock_client() {
    let config = Arc::new(Config::default());
    let mock_client = MockMockCloudflareClient::new();
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client));
    assert!(session.is_ok());
}

#[tokio::test]
async fn test_create_room_with_sender_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    // Mock successful session creation
    let expected_response = CloudflareSessionResponse {
        session_id: "test_session_id".to_string(),
        session_description: SessionDescription {
            r#type: "answer".to_string(),
            sdp: "test_sdp".to_string(),
        },
    };
    
    mock_client
        .expect_create_session()
        .returning(move |_| Ok(expected_response.clone()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.create_room_with_sender("test_room", "test_client", "test_offer_sdp".to_string()).await;
    
    assert!(result.is_ok());
    let connection_info = result.unwrap();
    assert_eq!(connection_info.room_id, "test_room");
    assert_eq!(connection_info.role, ClientRole::Sender);
    assert_eq!(connection_info.session_id, Some("test_session_id".to_string()));
    assert_eq!(connection_info.status, ConnectionStatus::Connecting);
}

#[tokio::test]
async fn test_create_room_with_sender_error() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    // Mock failed session creation
    mock_client
        .expect_create_session()
        .returning(|_| Err("API Error".into()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.create_room_with_sender("test_room", "test_client", "test_offer_sdp".to_string()).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_join_room_as_receiver_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    // Mock successful track addition
    let tracks = vec![
        Track {
            location: "remote".to_string(),
            mid: None,
            track_name: "video".to_string(),
            session_id: Some("sender_session_id".to_string()),
        },
        Track {
            location: "remote".to_string(),
            mid: None,
            track_name: "audio".to_string(),
            session_id: Some("sender_session_id".to_string()),
        },
    ];
    
    let expected_response = CloudflareTracksResponse {
        session_description: Some(SessionDescription {
            r#type: "answer".to_string(),
            sdp: "test_sdp".to_string(),
        }),
        tracks: tracks.clone(),
        requires_immediate_renegotiation: Some(false),
    };
    
    mock_client
        .expect_add_tracks()
        .returning(move |_, _, _| Ok(expected_response.clone()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.join_room_as_receiver("test_room", "test_client", "sender_session_id").await;
    
    assert!(result.is_ok());
    let connection_info = result.unwrap();
    assert_eq!(connection_info.room_id, "test_room");
    assert_eq!(connection_info.role, ClientRole::Receiver);
    assert_eq!(connection_info.session_id, Some("sender_session_id".to_string()));
    assert_eq!(connection_info.status, ConnectionStatus::Connecting);
}

#[tokio::test]
async fn test_join_room_as_receiver_error() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    // Mock failed track addition
    mock_client
        .expect_add_tracks()
        .returning(|_, _, _| Err("API Error".into()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.join_room_as_receiver("test_room", "test_client", "sender_session_id").await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_tracks_to_session_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    let tracks = vec![
        Track {
            location: "local".to_string(),
            mid: Some("audio".to_string()),
            track_name: "audio".to_string(),
            session_id: None,
        }
    ];
    
    let expected_response = CloudflareTracksResponse {
        session_description: Some(SessionDescription {
            r#type: "answer".to_string(),
            sdp: "test_sdp".to_string(),
        }),
        tracks: tracks.clone(),
        requires_immediate_renegotiation: Some(false),
    };
    
    mock_client
        .expect_add_tracks()
        .returning(move |_, _, _| Ok(expected_response.clone()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.add_tracks_to_session("test_session_id", tracks, None).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.session_description.is_some());
    assert_eq!(response.tracks.len(), 1);
    assert_eq!(response.requires_immediate_renegotiation, Some(false));
}

#[tokio::test]
async fn test_send_answer_sdp_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_send_answer_sdp()
        .returning(|_, _| Ok(()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.send_answer_sdp("test_session_id", "test_answer_sdp".to_string()).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_answer_sdp_error() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_send_answer_sdp()
        .returning(|_, _| Err("API Error".into()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.send_answer_sdp("test_session_id", "test_answer_sdp".to_string()).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_terminate_session_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_terminate_session()
        .returning(|_| Ok(()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.terminate_session("test_session_id", "test_room").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_terminate_session_error() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_terminate_session()
        .returning(|_| Err("API Error".into()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    // Terminate session should not fail even if Cloudflare API fails
    let result = session.terminate_session("test_session_id", "test_room").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_session_info_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    let expected_response = serde_json::json!({
        "session_id": "test_session_id",
        "status": "active"
    });
    
    mock_client
        .expect_get_session()
        .returning(move |_| Ok(expected_response.clone()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.get_session_info("test_session_id").await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["session_id"], "test_session_id");
    assert_eq!(response["status"], "active");
}

#[tokio::test]
async fn test_get_session_info_error() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_get_session()
        .returning(|_| Err("API Error".into()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.get_session_info("test_session_id").await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_credentials_success() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_validate_credentials()
        .returning(|| Ok(true));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.validate_credentials().await;
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_validate_credentials_failure() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_validate_credentials()
        .returning(|| Ok(false));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.validate_credentials().await;
    
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_validate_credentials_error() {
    let config = Arc::new(Config::default());
    let mut mock_client = MockMockCloudflareClient::new();
    
    mock_client
        .expect_validate_credentials()
        .returning(|| Err("API Error".into()));
    
    let session = CloudflareSession::new_with_client(config, Box::new(mock_client)).unwrap();
    
    let result = session.validate_credentials().await;
    
    assert!(result.is_err());
}

#[test]
fn test_generate_room_id() {
    let room_id1 = CloudflareSession::generate_room_id();
    let room_id2 = CloudflareSession::generate_room_id();
    
    // Room IDs should be unique UUIDs
    assert_ne!(room_id1, room_id2);
    assert_eq!(room_id1.len(), 36); // UUID length
    assert_eq!(room_id2.len(), 36);
}

#[test]
fn test_create_connection_info() {
    let config = Arc::new(Config::default());
    let session = CloudflareSession::new(config).unwrap();
    
    let connection_info = session.create_connection_info("test_room", ClientRole::Sender, Some("test_session".to_string()));
    
    assert_eq!(connection_info.room_id, "test_room");
    assert_eq!(connection_info.role, ClientRole::Sender);
    assert_eq!(connection_info.session_id, Some("test_session".to_string()));
    assert_eq!(connection_info.status, ConnectionStatus::Disconnected);
    assert_eq!(connection_info.app_id, "your-cloudflare-app-id");
} 