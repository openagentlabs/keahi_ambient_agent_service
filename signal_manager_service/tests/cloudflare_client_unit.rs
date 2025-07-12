use std::sync::Arc;
use mockall::predicate::*;
use mockall::*;
use async_trait::async_trait;
use signal_manager_service::cloudflare::client::{CloudflareClient, CloudflareClientTrait};
use signal_manager_service::cloudflare::models::*;
use signal_manager_service::config::Config;
use serde_json::Value;

// Mock HTTP client for testing
mock! {
    pub HttpClient {}
    
    #[async_trait]
    impl CloudflareClientTrait for HttpClient {
        async fn create_session(&self, offer_sdp: String) -> Result<CloudflareSessionResponse, Box<dyn std::error::Error + Send + Sync>>;
        async fn add_tracks(&self, session_id: &str, tracks: Vec<Track>, offer_sdp: Option<String>) -> Result<CloudflareTracksResponse, Box<dyn std::error::Error + Send + Sync>>;
        async fn send_answer_sdp(&self, session_id: &str, answer_sdp: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
        async fn terminate_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
        async fn get_session(&self, session_id: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
        async fn validate_credentials(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    }
}

#[tokio::test]
async fn test_cloudflare_client_creation() {
    // Test that we can create a CloudflareClient with default config
    let config = Arc::new(Config::default());
    let client = CloudflareClient::new(config).unwrap();
    
    // Note: We can't directly access private fields, so we test through public methods
    // This test verifies the client can be created successfully
    assert!(client.validate_credentials().await.is_ok());
}

#[tokio::test]
async fn test_cloudflare_client_create_session_success() {
    let mut mock_client = MockHttpClient::new();
    
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
    
    let result = mock_client.create_session("test_offer_sdp".to_string()).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.session_id, "test_session_id");
    assert_eq!(response.session_description.r#type, "answer");
    assert_eq!(response.session_description.sdp, "test_sdp");
}

#[tokio::test]
async fn test_cloudflare_client_create_session_error() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock failed session creation
    mock_client
        .expect_create_session()
        .returning(|_| Err("API Error".into()));
    
    let result = mock_client.create_session("test_offer_sdp".to_string()).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "API Error");
}

#[tokio::test]
async fn test_cloudflare_client_add_tracks_success() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock successful track addition
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
    
    let result = mock_client.add_tracks("test_session_id", tracks, None).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.session_description.is_some());
    assert_eq!(response.tracks.len(), 1);
    assert_eq!(response.requires_immediate_renegotiation, Some(false));
}

#[tokio::test]
async fn test_cloudflare_client_send_answer_sdp_success() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock successful answer SDP sending
    mock_client
        .expect_send_answer_sdp()
        .returning(|_, _| Ok(()));
    
    let result = mock_client.send_answer_sdp("test_session_id", "test_answer_sdp".to_string()).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cloudflare_client_terminate_session_success() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock successful session termination
    mock_client
        .expect_terminate_session()
        .returning(|_| Ok(()));
    
    let result = mock_client.terminate_session("test_session_id").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cloudflare_client_get_session_success() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock successful session retrieval
    let expected_response = serde_json::json!({
        "session_id": "test_session_id",
        "status": "active"
    });
    
    mock_client
        .expect_get_session()
        .returning(move |_| Ok(expected_response.clone()));
    
    let result = mock_client.get_session("test_session_id").await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["session_id"], "test_session_id");
    assert_eq!(response["status"], "active");
}

#[tokio::test]
async fn test_cloudflare_client_validate_credentials_success() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock successful credentials validation
    mock_client
        .expect_validate_credentials()
        .returning(|| Ok(true));
    
    let result = mock_client.validate_credentials().await;
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_cloudflare_client_validate_credentials_failure() {
    let mut mock_client = MockHttpClient::new();
    
    // Mock failed credentials validation
    mock_client
        .expect_validate_credentials()
        .returning(|| Ok(false));
    
    let result = mock_client.validate_credentials().await;
    
    assert!(result.is_ok());
    assert!(!result.unwrap());
} 