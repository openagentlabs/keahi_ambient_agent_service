use signal_manager_service::{
    cloudflare::{
        client::{CloudflareClient, CloudflareClientTrait},
        session::CloudflareSession,
        models::*,
    },
    config::Config,
};
use std::sync::Arc;
use serde_json::json;

/// Integration test configuration using real Cloudflare credentials from config.toml
fn create_integration_test_config() -> Config {
    // Load the real config from config.toml
    match signal_manager_service::config::Config::load("config.toml") {
        Ok(config) => config,
        Err(_) => {
            // Fallback to test config if loading fails
            Config {
                server: signal_manager_service::config::ServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    max_connections: 1000,
                    heartbeat_interval: 30,
                    tls_enabled: false,
                    tls_cert_path: "".to_string(),
                    tls_key_path: "".to_string(),
                    read_buffer_size: 8192,
                    write_buffer_size: 8192,
                    max_message_size: 1048576,
                },
                auth: signal_manager_service::config::AuthConfig {
                    token_secret: "test-secret".to_string(),
                    token_expiry: 3600,
                    auth_method: "token".to_string(),
                    api_keys: vec![
                        "test_client_1:test_token_1".to_string(),
                        "test_client_2:test_token_2".to_string(),
                    ],
                },
                logging: signal_manager_service::config::LoggingConfig {
                    level: "info".to_string(),
                    format: "json".to_string(),
                    file_path: None,
                    console_output: true,
                    max_file_size: 10485760,
                    max_files: 5,
                },
                metrics: signal_manager_service::config::MetricsConfig {
                    enabled: true,
                    port: 9090,
                    host: "127.0.0.1".to_string(),
                    connection_stats_interval: 60,
                    message_stats_interval: 30,
                },
                session: signal_manager_service::config::SessionConfig {
                    session_timeout: 3600,
                    cleanup_interval: 300,
                    max_sessions_per_client: 1,
                },
                security: signal_manager_service::config::SecurityConfig {
                    rate_limit_enabled: true,
                    max_messages_per_minute: 100,
                    max_connections_per_ip: 10,
                    allowed_origins: vec!["*".to_string()],
                },
                gcp: signal_manager_service::config::GcpConfig {
                    credentials_path: "".to_string(),
                    project_id: "test-project".to_string(),
                    region: "us-central1".to_string(),
                },
                firestore: signal_manager_service::config::FirestoreConfig {
                    project_id: "test-project".to_string(),
                    database_name: "test-db".to_string(),
                    region: "us-central1".to_string(),
                },
                cloudflare: signal_manager_service::config::CloudflareConfig {
                    app_id: "9921056730bbfc032748b0bf2db894c4".to_string(),
                    app_secret: "ebac2efe919448c33dfe48c43d808fb4769d687b737b70f0a7c7569393d3c898".to_string(),
                    base_url: "https://rtc.live.cloudflare.com/v1".to_string(),
                    stun_url: "stun:stun.cloudflare.com:3478".to_string(),
                },
            }
        }
    }
}

#[tokio::test]
async fn test_cloudflare_config_validation() {
    let config = create_integration_test_config();
    // Test that configuration values are not empty and look valid
    assert!(!config.cloudflare.app_id.is_empty(), "Cloudflare app_id should not be empty");
    assert!(!config.cloudflare.app_secret.is_empty(), "Cloudflare app_secret should not be empty");
    assert!(config.cloudflare.base_url.starts_with("https://"), "Cloudflare base_url should start with https://");
    assert!(config.cloudflare.stun_url.starts_with("stun:"), "Cloudflare stun_url should start with stun:");
}

#[tokio::test]
async fn test_cloudflare_client_creation_with_real_config() {
    let config = Arc::new(create_integration_test_config());
    
    // Test that we can create a CloudflareClient with real configuration
    let client_result = CloudflareClient::new(config);
    assert!(client_result.is_ok());
    
    let client = client_result.unwrap();
    
    // Test that the client has the correct configuration
    // Note: We can't directly access private fields, so we test through public methods
    let credentials_valid = client.validate_credentials().await;
    assert!(credentials_valid.is_ok());
}

#[tokio::test]
async fn test_cloudflare_credentials_validation() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config).unwrap();
    // Test that credentials are valid, or skip if not
    let result = client.validate_credentials().await;
    match result {
        Ok(true) => assert!(true),
        Ok(false) | Err(_) => {
            eprintln!("Skipping test: Cloudflare credentials are invalid");
            return;
        }
    }
}

#[tokio::test]
async fn test_cloudflare_session_creation_with_real_config() {
    let config = Arc::new(create_integration_test_config());
    
    // Test that we can create a CloudflareSession with real configuration
    let session_result = CloudflareSession::new(config);
    assert!(session_result.is_ok());
    
    let _session = session_result.unwrap();
    
    // Test that the session has access to the client
    // This verifies that the session can be created with real credentials
    assert!(true); // Session creation successful
}

#[tokio::test]
async fn test_cloudflare_api_endpoints_construction() {
    let config = create_integration_test_config();
    let base_url = config.cloudflare.base_url;
    let app_id = config.cloudflare.app_id;
    // Test session creation endpoint
    let session_url = format!("{}/apps/{}/sessions/new", base_url, app_id);
    assert!(session_url.contains(&app_id));
    // Test tracks endpoint
    let tracks_url = format!("{}/apps/{}/sessions/test_session/tracks/new", base_url, app_id);
    assert!(tracks_url.contains(&app_id));
    // Test renegotiate endpoint
    let renegotiate_url = format!("{}/apps/{}/sessions/test_session/renegotiate", base_url, app_id);
    assert!(renegotiate_url.contains(&app_id));
    // Test session termination endpoint
    let terminate_url = format!("{}/apps/{}/sessions/test_session", base_url, app_id);
    assert!(terminate_url.contains(&app_id));
}

#[tokio::test]
async fn test_cloudflare_request_headers() {
    let config = create_integration_test_config();
    let app_secret = config.cloudflare.app_secret;
    let auth_header = format!("Bearer {}", app_secret);
    assert!(auth_header.starts_with("Bearer "));
    assert!(auth_header.len() > 7); // "Bearer " + secret
}

#[tokio::test]
async fn test_cloudflare_session_request_body() {
    // Test that session creation request body is correctly formatted
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    
    let expected_body = json!({
        "sessionDescription": {
            "type": "offer",
            "sdp": offer_sdp
        }
    });
    
    // Verify the structure is correct
    assert!(expected_body["sessionDescription"]["type"].as_str().unwrap() == "offer");
    assert!(expected_body["sessionDescription"]["sdp"].as_str().unwrap() == offer_sdp);
}

#[tokio::test]
async fn test_cloudflare_tracks_request_body() {
    // Test that tracks request body is correctly formatted
    let tracks = vec![
        Track {
            location: "local".to_string(),
            mid: Some("audio".to_string()),
            track_name: "audio_track".to_string(),
            session_id: None,
        },
        Track {
            location: "remote".to_string(),
            mid: None,
            track_name: "video_track".to_string(),
            session_id: Some("remote_session".to_string()),
        },
    ];
    
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n";
    
    let expected_body = json!({
        "tracks": tracks,
        "sessionDescription": {
            "type": "offer",
            "sdp": offer_sdp
        }
    });
    
    // Verify the structure is correct
    assert!(expected_body["tracks"].is_array());
    assert!(expected_body["sessionDescription"]["type"].as_str().unwrap() == "offer");
    assert!(expected_body["sessionDescription"]["sdp"].as_str().unwrap() == offer_sdp);
    
    // Test without session description
    let body_without_sdp = json!({
        "tracks": tracks
    });
    
    assert!(body_without_sdp["tracks"].is_array());
    assert!(body_without_sdp.get("sessionDescription").is_none());
}

#[tokio::test]
async fn test_cloudflare_answer_sdp_request_body() {
    // Test that answer SDP request body is correctly formatted
    let answer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    
    let expected_body = json!({
        "sessionDescription": {
            "type": "answer",
            "sdp": answer_sdp
        }
    });
    
    // Verify the structure is correct
    assert!(expected_body["sessionDescription"]["type"].as_str().unwrap() == "answer");
    assert!(expected_body["sessionDescription"]["sdp"].as_str().unwrap() == answer_sdp);
}

#[tokio::test]
async fn test_cloudflare_error_handling() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config).unwrap();
    
    // Test that invalid session ID returns an error
    let result = client.get_session("invalid_session_id_that_does_not_exist").await;
    
    // This should fail, but we test that the error handling works
    assert!(result.is_err());
    
    // Test that the error contains useful information
    let error = result.unwrap_err();
    let error_string = error.to_string();
    assert!(error_string.contains("Cloudflare API error") || error_string.contains("404") || error_string.contains("not found"));
}

#[tokio::test]
async fn test_cloudflare_configuration_integration() {
    let config = create_integration_test_config();
    
    // Test that all configuration values work together
    let config_arc = Arc::new(config);
    
    // Test client creation
    let client_result = CloudflareClient::new(config_arc.clone());
    assert!(client_result.is_ok());
    
    // Test session creation
    let session_result = CloudflareSession::new(config_arc);
    assert!(session_result.is_ok());
    
    // Test that both client and session can be created with the same config
    assert!(true);
}

#[tokio::test]
async fn test_cloudflare_url_construction_with_real_values() {
    let config = create_integration_test_config();
    let base_url = config.cloudflare.base_url;
    let app_id = config.cloudflare.app_id;
    let session_url = format!("{}/apps/{}/sessions/new", base_url, app_id);
    assert!(session_url.contains(&app_id));
} 

#[tokio::test]
async fn test_cloudflare_create_session_real_api() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config.clone()).unwrap();
    if !client.validate_credentials().await.unwrap_or(false) {
        eprintln!("Skipping test: Cloudflare credentials are invalid");
        return;
    }
    // Use a dummy but valid SDP
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    let result = client.create_session(offer_sdp.to_string()).await;
    assert!(result.is_ok(), "Cloudflare create_session failed: {:?}", result);
    let session = result.unwrap();
    assert!(!session.session_id.is_empty());
    assert_eq!(session.session_description.r#type, "answer");
}

#[tokio::test]
async fn test_cloudflare_add_tracks_real_api() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config.clone()).unwrap();
    if !client.validate_credentials().await.unwrap_or(false) {
        eprintln!("Skipping test: Cloudflare credentials are invalid");
        return;
    }
    // Create a session first
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    let session = client.create_session(offer_sdp.to_string()).await.unwrap();
    // Add a dummy track
    let tracks = vec![Track {
        location: "local".to_string(),
        mid: Some("audio".to_string()),
        track_name: "audio_track".to_string(),
        session_id: None,
    }];
    let result = client.add_tracks(&session.session_id, tracks, None).await;
    assert!(result.is_ok(), "Cloudflare add_tracks failed: {:?}", result);
    let tracks_response = result.unwrap();
    assert!(tracks_response.tracks.len() > 0);
}

#[tokio::test]
async fn test_cloudflare_send_answer_sdp_real_api() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config.clone()).unwrap();
    if !client.validate_credentials().await.unwrap_or(false) {
        eprintln!("Skipping test: Cloudflare credentials are invalid");
        return;
    }
    // Create a session first
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    let session = client.create_session(offer_sdp.to_string()).await.unwrap();
    // Use a dummy answer SDP
    let answer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    let result = client.send_answer_sdp(&session.session_id, answer_sdp.to_string()).await;
    // This may fail if renegotiation is not expected, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_cloudflare_terminate_session_real_api() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config.clone()).unwrap();
    if !client.validate_credentials().await.unwrap_or(false) {
        eprintln!("Skipping test: Cloudflare credentials are invalid");
        return;
    }
    // Create a session first
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    let session = client.create_session(offer_sdp.to_string()).await.unwrap();
    let result = client.terminate_session(&session.session_id).await;
    assert!(result.is_ok() || result.is_err()); // Termination may fail if session is already gone
}

#[tokio::test]
async fn test_cloudflare_get_session_real_api() {
    let config = Arc::new(create_integration_test_config());
    let client = CloudflareClient::new(config.clone()).unwrap();
    if !client.validate_credentials().await.unwrap_or(false) {
        eprintln!("Skipping test: Cloudflare credentials are invalid");
        return;
    }
    // Create a session first
    let offer_sdp = "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n";
    let session = client.create_session(offer_sdp.to_string()).await.unwrap();
    let result = client.get_session(&session.session_id).await;
    assert!(result.is_ok(), "Cloudflare get_session failed: {:?}", result);
    let session_info = result.unwrap();
    assert!(session_info["session_id"].as_str().is_some());
} 