use crate::signalmanager::error::SignalManagerError;
use crate::signalmanager::types::*;
use crate::signalmanager::websocket::WebSocketClient;
use crate::signalmanager::config::SignalManagerConfig;
use std::time::Duration;
use log::{info, error, debug, warn};

// Simple timeout constants
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub type RoomResponse = Option<(Option<String>, Option<String>)>;

// Simple callback type for state changes
pub type StateCallback = Box<dyn Fn(ConnectionState) + Send + Sync>;

pub struct SignalManagerClient {
    config: SignalManagerConfig,
    websocket: Option<WebSocketClient>,
    state: ConnectionState,
    state_callback: Option<StateCallback>,
    last_room_response: RoomResponse,
}

impl SignalManagerClient {
    pub fn new(config: SignalManagerConfig) -> Self {
        info!("[SignalManagerClient] Creating new client with config: {:?}", config);
        Self {
            config,
            websocket: None,
            state: ConnectionState::default(),
            state_callback: None,
            last_room_response: None,
        }
    }

    // Set callback for state changes
    pub fn set_state_callback(&mut self, callback: StateCallback) {
        self.state_callback = Some(callback);
    }

    // Update state and notify callback
    fn update_state(&mut self, new_state: ConnectionState) {
        self.state = new_state.clone();
        if let Some(callback) = &self.state_callback {
            callback(new_state);
        }
    }

    pub async fn connect(&mut self) -> Result<(), SignalManagerError> {
        info!("[connect] Connecting client_id={}", self.config.client_id);
        
        // Check if already connected
        if self.state.is_connected {
            info!("[connect] Already connected");
            return Ok(());
        }

        // Update state to connecting
        self.update_state(ConnectionState {
            state_type: ConnectionStateType::TryingToConnect,
            is_connected: false,
            is_connecting: true,
            is_reconnecting: false,
            reconnect_attempts: 0,
            current_retry_interval: 0,
            next_retry_time: None,
            last_heartbeat: 0,
        });

        // Create WebSocket connection
        let url = self.config.websocket_url();
        info!("[connect] Connecting to WebSocket at {}", url);
        
        let mut websocket = WebSocketClient::new(url);
        match websocket.connect().await {
            Ok(()) => {
                info!("[connect] WebSocket connection established");
                self.websocket = Some(websocket);
                
                // Update state to connected
                self.update_state(ConnectionState {
                    state_type: ConnectionStateType::Connected,
                    is_connected: true,
                    is_connecting: false,
                    is_reconnecting: false,
                    reconnect_attempts: 0,
                    current_retry_interval: 0,
                    next_retry_time: None,
                    last_heartbeat: 0,
                });

                // Send initial messages
                self.send_connect().await?;
                self.send_register().await?;
                
                info!("[connect] Successfully connected and registered");
                Ok(())
            }
            Err(e) => {
                error!("[connect] WebSocket connection failed: {}", e);
                self.update_state(ConnectionState {
                    state_type: ConnectionStateType::DisconnectedNotToConnect,
                    is_connected: false,
                    is_connecting: false,
                    is_reconnecting: false,
                    reconnect_attempts: 0,
                    current_retry_interval: 0,
                    next_retry_time: None,
                    last_heartbeat: 0,
                });
                Err(e)
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<(), SignalManagerError> {
        info!("[disconnect] Disconnecting from SignalManager");
        
        // Update state to disconnecting
        self.update_state(ConnectionState {
            state_type: ConnectionStateType::DisconnectingDisconnectRequested,
            is_connected: false,
            is_connecting: false,
            is_reconnecting: false,
            reconnect_attempts: 0,
            current_retry_interval: 0,
            next_retry_time: None,
            last_heartbeat: 0,
        });

        // Send unregister if connected
        if let Some(_) = &self.websocket {
            self.send_unregister().await?;
        }

        // Close WebSocket
        if let Some(mut websocket) = self.websocket.take() {
            websocket.close().await;
        }

        // Update state to disconnected
        self.update_state(ConnectionState {
            state_type: ConnectionStateType::DisconnectedNotToConnect,
            is_connected: false,
            is_connecting: false,
            is_reconnecting: false,
            reconnect_attempts: 0,
            current_retry_interval: 0,
            next_retry_time: None,
            last_heartbeat: 0,
        });

        info!("[disconnect] Successfully disconnected");
        Ok(())
    }

    pub async fn send_room_create(&mut self, payload: WebRTCRoomCreatePayload) -> Result<(Option<String>, Option<String>), SignalManagerError> {
        let client_id = payload.client_id.clone();
        info!("[send_room_create] Sending room create request for client_id: {}", client_id);
        
        if let Some(websocket) = &self.websocket {
            // Clear previous response
            self.last_room_response = None;
            
            // Send message
            let message = Message::room_create(payload);
            websocket.send(message)?;
            
            // Wait for response with timeout
            let start_time = std::time::Instant::now();
            while start_time.elapsed() < COMMAND_TIMEOUT {
                // Process incoming messages
                if let Some(websocket) = &mut self.websocket {
                    if let Ok(Some(msg)) = websocket.receive().await {
                        self.handle_message(msg).await?;
                        
                        // Check if we got our response
                        if let Some(response) = &self.last_room_response {
                            info!("[send_room_create] Received room creation response: {:?}", response);
                            return Ok(response.clone());
                        }
                    }
                }
                
                // Small delay before next check
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            
            warn!("[send_room_create] Timeout waiting for room creation response");
            Err(SignalManagerError::Timeout("Room creation timeout".to_string()))
        } else {
            error!("[send_room_create] Not connected to WebSocket");
            Err(SignalManagerError::NotConnected)
        }
    }

    pub fn get_state(&self) -> ConnectionState {
        self.state.clone()
    }

    // Private methods
    async fn send_connect(&self) -> Result<(), SignalManagerError> {
        if let Some(websocket) = &self.websocket {
            let message = Message::connect(
                self.config.client_id.clone(),
                self.config.auth_token.clone(),
            );
            websocket.send(message)?;
            info!("[send_connect] Connect message sent");
        }
        Ok(())
    }

    async fn send_register(&self) -> Result<(), SignalManagerError> {
        if let Some(websocket) = &self.websocket {
            let message = Message::register(
                self.config.client_id.clone(),
                self.config.auth_token.clone(),
            );
            websocket.send(message)?;
            info!("[send_register] Register message sent");
        }
        Ok(())
    }

    async fn send_unregister(&self) -> Result<(), SignalManagerError> {
        if let Some(websocket) = &self.websocket {
            let message = Message::unregister(self.config.client_id.clone());
            websocket.send(message)?;
            info!("[send_unregister] Unregister message sent");
        }
        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> Result<(), SignalManagerError> {
        debug!("[handle_message] Processing message: {:?}", message.message_type);
        
        match &message.payload {
            Payload::ConnectAck(ack) => {
                info!("[handle_message] Received ConnectAck: {}", ack.status);
            }
            Payload::RegisterAck(ack) => {
                info!("[handle_message] Received RegisterAck: {}", ack.status);
            }
            Payload::UnregisterAck(ack) => {
                info!("[handle_message] Received UnregisterAck: {}", ack.status);
            }
            Payload::WebRTCRoomCreateAck(ack) => {
                info!("[handle_message] Received RoomCreateAck: room_id={:?}, session_id={:?}", 
                    ack.room_id, ack.session_id);
                self.last_room_response = Some((ack.room_id.clone(), ack.session_id.clone()));
            }
            Payload::Error(error_payload) => {
                error!("[handle_message] Received Error: {} - {}", 
                    error_payload.error_code, error_payload.error_message);
            }
            _ => {
                debug!("[handle_message] Unhandled message type: {:?}", message.message_type);
            }
        }
        
        Ok(())
    }
} 