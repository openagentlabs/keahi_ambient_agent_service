use crate::webrtcclient::{WebRTCClient, SDPOffer, RoomCreationParams};
use crate::signalmanager::{SignalManagerClient, SignalManagerConfig, ConnectionState, WebRTCRoomCreatePayload};
use crate::WebRTCRoomCreatePayloadWrapper;
use log::{info, error, debug};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tauri::Emitter;
use std::sync::Mutex as StdMutex;

// Global signal manager client - now re-initializable and async safe
pub static SIGNAL_MANAGER: once_cell::sync::Lazy<TokioMutex<Option<Arc<TokioMutex<SignalManagerClient>>>>> = once_cell::sync::Lazy::new(|| TokioMutex::new(None));

// Tauri commands for WebRTC
#[tauri::command]
pub async fn generate_webrtc_offer() -> Result<SDPOffer, String> {
    info!("[generate_webrtc_offer] Generating WebRTC offer");
    let mut client = WebRTCClient::with_default_config();
    client.create_offer().await.map_err(|e| {
        error!("[generate_webrtc_offer] Failed to generate offer: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn prepare_room_creation(
    client_id: String,
    auth_token: String,
    role: String,
) -> Result<RoomCreationParams, String> {
    info!("[prepare_room_creation] Preparing room creation for client_id: {}", client_id);
    let mut client = WebRTCClient::with_default_config();
    client.prepare_room_creation(client_id, auth_token, role)
        .await
        .map_err(|e| {
            error!("[prepare_room_creation] Failed to prepare room creation: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn cleanup_webrtc_connection() -> Result<(), String> {
    info!("[cleanup_webrtc_connection] Cleaning up WebRTC connection");
    let mut client = WebRTCClient::with_default_config();
    client.reset_with_config(false).await.map_err(|e| {
        error!("[cleanup_webrtc_connection] Failed to cleanup WebRTC connection: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn create_room_with_webrtc(
    client_id: String,
    auth_token: String,
    role: String,
) -> Result<WebRTCRoomCreatePayloadWrapper, String> {
    info!("[create_room_with_webrtc] Creating room with WebRTC for client_id: {}", client_id);
    let mut client = WebRTCClient::with_default_config();
    
    let room_params = client.prepare_room_creation(client_id.clone(), auth_token.clone(), role.clone())
        .await
        .map_err(|e| {
            error!("[create_room_with_webrtc] Failed to prepare room creation: {}", e);
            e.to_string()
        })?;
    
    Ok(WebRTCRoomCreatePayloadWrapper {
        version: "1.0.0".to_string(),
        client_id,
        auth_token,
        role,
        offer_sdp: room_params.offer_sdp,
        metadata: room_params.metadata,
    })
}

// Tauri commands for Signal Manager
#[tauri::command]
pub async fn init_signal_manager(
    url: String,
    port: u16,
    client_id: String,
    auth_token: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    info!("[init_signal_manager] Initializing signal manager: url={}, port={}, client_id={}", url, port, client_id);
    let config = SignalManagerConfig::new(url, port, client_id, auth_token);
    let mut client = SignalManagerClient::new(config);
    // Set up state callback to emit events
    let app_handle_clone = app_handle.clone();
    client.set_state_callback(Box::new(move |state| {
        info!("SignalManager state changed: {:?}", state);
        let _ = app_handle_clone.emit("signal-manager:state-changed", state.clone());
        match state.state_type {
            crate::signalmanager::types::ConnectionStateType::Connected => {
                let _ = app_handle_clone.emit("signal-manager:connected", ());
            }
            crate::signalmanager::types::ConnectionStateType::TryingToConnect => {
                let _ = app_handle_clone.emit("signal-manager:connecting", ());
            }
            crate::signalmanager::types::ConnectionStateType::WasConnectedTryingToReconnect => {
                let _ = app_handle_clone.emit("signal-manager:reconnecting", state.reconnect_attempts);
            }
            crate::signalmanager::types::ConnectionStateType::DisconnectedNotToConnect => {
                let _ = app_handle_clone.emit("signal-manager:disconnected", ());
            }
            crate::signalmanager::types::ConnectionStateType::DisconnectingDisconnectRequested => {
                let _ = app_handle_clone.emit("signal-manager:disconnecting", ());
            }
        }
    }));
    let arc_client = Arc::new(TokioMutex::new(client));
    let mut global = SIGNAL_MANAGER.lock().await;
    if global.is_some() {
        return Err("Signal manager already initialized".to_string());
    }
    *global = Some(arc_client);
    app_handle.emit("signal-manager:initialized", ()).map_err(|e| {
        error!("[init_signal_manager] Failed to emit initialized event: {}", e);
        e.to_string()
    })?;
    info!("[init_signal_manager] Signal manager initialized successfully");
    Ok(())
}

#[tauri::command]
pub async fn connect_signal_manager() -> Result<(), String> {
    info!("[connect_signal_manager] Connecting to signal manager");
    let global = SIGNAL_MANAGER.lock().await;
    let client = match &*global {
        Some(c) => c.clone(),
        None => return Err("Signal manager not initialized".to_string()),
    };
    drop(global);
    let mut client = client.lock().await;
    match client.connect().await {
        Ok(()) => {
            info!("[connect_signal_manager] Successfully connected to signal manager");
            Ok(())
        }
        Err(e) => {
            error!("[connect_signal_manager] Failed to connect to signal manager: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn disconnect_signal_manager() -> Result<(), String> {
    info!("[disconnect_signal_manager] Disconnecting from signal manager");
    let client = {
        let mut global = SIGNAL_MANAGER.lock().await;
        if let Some(client) = &*global {
            let client = client.clone();
            drop(global);
            let mut client = client.lock().await;
            match client.disconnect().await {
                Ok(()) => {
                    info!("[disconnect_signal_manager] Successfully disconnected from signal manager");
                    if let Err(e) = client.reset().await {
                        error!("[disconnect_signal_manager] Failed to reset client state: {}", e);
                        return Err(format!("Failed to reset client state: {}", e));
                    }
                    // Drop the client
                    let mut global = SIGNAL_MANAGER.lock().await;
                    *global = None;
                    Ok(())
                }
                Err(e) => {
                    error!("[disconnect_signal_manager] Failed to disconnect from signal manager: {}", e);
                    Err(e.to_string())
                }
            }
        } else {
            Err("Signal manager not initialized".to_string())
        }
    };
    client
}

#[tauri::command]
pub async fn reset_signal_manager() -> Result<(), String> {
    info!("[reset_signal_manager] Resetting signal manager state");
    let client = {
        let mut global = SIGNAL_MANAGER.lock().await;
        if let Some(client) = &*global {
            let client = client.clone();
            drop(global);
            let mut client = client.lock().await;
            match client.reset().await {
                Ok(()) => {
                    info!("[reset_signal_manager] Signal manager state reset successfully");
                    let mut global = SIGNAL_MANAGER.lock().await;
                    *global = None;
                    Ok(())
                }
                Err(e) => {
                    error!("[reset_signal_manager] Failed to reset signal manager state: {}", e);
                    Err(format!("Failed to reset signal manager state: {}", e))
                }
            }
        } else {
            info!("[reset_signal_manager] Signal manager not initialized, nothing to reset");
            Ok(())
        }
    };
    client
}

#[tauri::command]
pub async fn get_signal_manager_state() -> Result<ConnectionState, String> {
    let global = SIGNAL_MANAGER.lock().await;
    let client = match &*global {
        Some(c) => c.clone(),
        None => return Err("Signal manager not initialized".to_string()),
    };
    drop(global);
    let client = client.lock().await;
    Ok(client.get_state())
}

#[tauri::command]
pub async fn send_room_create(
    version: String,
    client_id: String,
    auth_token: String,
    role: String,
    offer_sdp: Option<String>,
    metadata: Option<serde_json::Value>,
) -> Result<(Option<String>, Option<String>), String> {
    info!("[send_room_create] Sending room create request for client_id: {}", client_id);
    let global = SIGNAL_MANAGER.lock().await;
    let client = match &*global {
        Some(c) => c.clone(),
        None => return Err("Signal manager not initialized".to_string()),
    };
    drop(global);
    let mut client = client.lock().await;
    let payload = WebRTCRoomCreatePayload {
        version,
        client_id,
        auth_token,
        role,
        offer_sdp,
        metadata,
    };
    match client.send_room_create(payload).await {
        Ok(result) => {
            info!("[send_room_create] Room created successfully: {:?}", result);
            Ok(result)
        }
        Err(e) => {
            error!("[send_room_create] Failed to create room: {}", e);
            Err(e.to_string())
        }
    }
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
} 