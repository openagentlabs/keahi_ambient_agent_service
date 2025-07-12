// Import all test modules
mod message;
mod config;
mod auth;
mod protocol;
mod server;
mod database;
mod cloudflare_session_unit;

// The modules are automatically discovered by Rust's test runner
// No need to re-export them explicitly 