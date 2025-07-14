// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod webrtcclient;
pub mod signalmanager;
pub mod commands;

use serde::{Deserialize, Serialize};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs;
use std::fs::File;
use log::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRTCRoomCreatePayloadWrapper {
    pub version: String,
    pub client_id: String,
    pub auth_token: String,
    pub role: String,
    pub offer_sdp: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure logs directory exists
    let _ = fs::create_dir_all("../logs");
    let log_file = File::create("../logs/tauri-app.log").unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::generate_webrtc_offer,
            commands::prepare_room_creation,
            commands::cleanup_webrtc_connection,
            commands::create_room_with_webrtc,
            commands::init_signal_manager,
            commands::connect_signal_manager,
            commands::disconnect_signal_manager,
            commands::get_signal_manager_state,
            commands::send_room_create
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
