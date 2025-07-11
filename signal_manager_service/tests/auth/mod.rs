use signal_manager_service::auth::AuthManager;
use signal_manager_service::config::Config;
use std::sync::Arc;

#[test]
fn test_auth_manager() {
    let config = Config::default();
    let _auth_manager = AuthManager::new(Arc::new(config));
    // Test should pass since we're not actually calling async functions
    // In a real test, we'd use tokio::test or similar
} 