use signal_manager_service::config::Config;

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