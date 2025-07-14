use tauri_app_lib::webrtcclient::{
    WebRTCClient, WebRTCConfig, WebRTCError, SDPOffer, RoomCreationParams,
};

#[tokio::test]
async fn test_webrtc_client_creation() {
    let config = WebRTCConfig::new(
        "stun:stun.cloudflare.com:3478".to_string(),
        "test_app_id".to_string(),
        "test_app_secret".to_string(),
    );
    
    let client = WebRTCClient::new(config);
    assert_eq!(client.get_config().app_id, "test_app_id");
    assert_eq!(client.get_config().app_secret, "test_app_secret");
}

#[tokio::test]
async fn test_webrtc_client_default_config() {
    let client = WebRTCClient::with_default_config();
    let config = client.get_config();
    
    assert_eq!(config.stun_url, "stun:stun.cloudflare.com:3478");
    assert_eq!(config.app_id, "bffd14dc10f70248bbcf42d3c5ef4307");
    assert_eq!(config.app_secret, "98468ea69f92fc7cb75c436bbfb4155296f4a29a0dc0b642247a124dc328420a");
}

#[tokio::test]
async fn test_config_update() {
    let mut client = WebRTCClient::with_default_config();
    let new_config = WebRTCConfig::new(
        "stun:stun.example.com:3478".to_string(),
        "new_app_id".to_string(),
        "new_app_secret".to_string(),
    );
    
    client.update_config(new_config);
    assert_eq!(client.get_config().stun_url, "stun:stun.example.com:3478");
    assert_eq!(client.get_config().app_id, "new_app_id");
}

#[tokio::test]
async fn test_sdp_offer_creation() {
    let mut client = WebRTCClient::with_default_config();
    
    let result = client.create_offer().await;
    assert!(result.is_ok());
    
    let offer = result.unwrap();
    assert_eq!(offer.type_, "offer");
    assert!(!offer.sdp.is_empty());
    assert!(offer.sdp.contains("v=0"));
    assert!(offer.sdp.contains("m=video"));
}

#[tokio::test]
async fn test_room_creation_params() {
    let mut client = WebRTCClient::with_default_config();
    
    let result = client.prepare_room_creation(
        "test_client_1".to_string(),
        "test_token_1".to_string(),
        "sender".to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    let params = result.unwrap();
    assert_eq!(params.client_id, "test_client_1");
    assert_eq!(params.auth_token, "test_token_1");
    assert_eq!(params.role, "sender");
    assert!(params.offer_sdp.is_some());
    assert!(params.metadata.is_some());
    
    // Check metadata
    let metadata = params.metadata.unwrap();
    assert!(metadata.get("webrtc_offer_type").is_some());
    assert!(metadata.get("timestamp").is_some());
    assert!(metadata.get("app_id").is_some());
    assert!(metadata.get("stun_url").is_some());
}

#[tokio::test]
async fn test_room_creation_params_with_metadata() {
    let mut client = WebRTCClient::with_default_config();
    
    let params = RoomCreationParams::new(
        "test_client_2".to_string(),
        "test_token_2".to_string(),
        "receiver".to_string(),
    );
    
    assert_eq!(params.client_id, "test_client_2");
    assert_eq!(params.auth_token, "test_token_2");
    assert_eq!(params.role, "receiver");
    assert!(params.offer_sdp.is_none());
    assert!(params.metadata.is_none());
}

#[tokio::test]
async fn test_room_creation_params_builder() {
    let params = RoomCreationParams::new(
        "test_client_3".to_string(),
        "test_token_3".to_string(),
        "sender".to_string(),
    )
    .with_offer_sdp("test_sdp_offer".to_string())
    .with_metadata(serde_json::json!({
        "custom_field": "custom_value"
    }));
    
    assert_eq!(params.offer_sdp, Some("test_sdp_offer".to_string()));
    assert!(params.metadata.is_some());
    
    let metadata = params.metadata.unwrap();
    assert_eq!(metadata.get("custom_field").unwrap(), "custom_value");
}

#[tokio::test]
async fn test_webrtc_client_close() {
    let mut client = WebRTCClient::with_default_config();
    
    // Create an offer first to establish a connection
    let _offer = client.create_offer().await.unwrap();
    
    // Close the connection
    let result = client.close().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sdp_offer_serialization() {
    let offer = SDPOffer::new(
        "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n".to_string(),
        "offer".to_string(),
    );
    
    let serialized = serde_json::to_string(&offer).unwrap();
    let deserialized: SDPOffer = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(offer.sdp, deserialized.sdp);
    assert_eq!(offer.type_, deserialized.type_);
}

#[tokio::test]
async fn test_room_creation_params_serialization() {
    let params = RoomCreationParams::new(
        "test_client_4".to_string(),
        "test_token_4".to_string(),
        "sender".to_string(),
    )
    .with_offer_sdp("test_sdp".to_string())
    .with_metadata(serde_json::json!({
        "test_field": "test_value"
    }));
    
    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: RoomCreationParams = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(params.client_id, deserialized.client_id);
    assert_eq!(params.auth_token, deserialized.auth_token);
    assert_eq!(params.role, deserialized.role);
    assert_eq!(params.offer_sdp, deserialized.offer_sdp);
    assert_eq!(params.metadata, deserialized.metadata);
}

#[tokio::test]
async fn test_webrtc_config_serialization() {
    let config = WebRTCConfig::new(
        "stun:stun.test.com:3478".to_string(),
        "test_app".to_string(),
        "test_secret".to_string(),
    );
    
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: WebRTCConfig = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(config.stun_url, deserialized.stun_url);
    assert_eq!(config.app_id, deserialized.app_id);
    assert_eq!(config.app_secret, deserialized.app_secret);
}

#[tokio::test]
async fn test_webrtc_config_default() {
    let config = WebRTCConfig::default();
    
    assert_eq!(config.stun_url, "stun:stun.cloudflare.com:3478");
    assert_eq!(config.app_id, "bffd14dc10f70248bbcf42d3c5ef4307");
    assert_eq!(config.app_secret, "98468ea69f92fc7cb75c436bbfb4155296f4a29a0dc0b642247a124dc328420a");
} 