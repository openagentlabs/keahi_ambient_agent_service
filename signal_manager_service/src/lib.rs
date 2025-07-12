pub mod config;
pub mod error;
pub mod message;
pub mod server;
pub mod session;
pub mod auth;
pub mod database;
pub mod frame_handlers;
pub mod type_two_handlers;
pub mod cloudflare;
pub mod webrtc_handlers;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>; 