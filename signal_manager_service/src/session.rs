use crate::message::{Message, MessageType, Payload, ConnectAckPayload, ErrorPayload};
use crate::auth::AuthManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{self, Sender, Receiver};
use uuid::Uuid;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct ClientSession {
    pub client_id: String,
    pub session_id: String,
    pub connected_at: std::time::Instant,
    pub last_heartbeat: std::time::Instant,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    auth_manager: Arc<AuthManager>,
    message_sender: Sender<(String, Message)>,
}

impl SessionManager {
    pub fn new(auth_manager: Arc<AuthManager>) -> (Self, Receiver<(String, Message)>) {
        let (tx, rx) = mpsc::channel(1000);
        
        let manager = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            auth_manager,
            message_sender: tx,
        };
        
        (manager, rx)
    }

    pub async fn handle_connect(&self, client_id: String, auth_token: String) -> Result<Message, crate::Error> {
        // Authenticate the client
        if !self.auth_manager.authenticate(&client_id, &auth_token).await? {
            return Ok(Message::new(
                MessageType::Error,
                Payload::Error(ErrorPayload {
                    error_code: 1,
                    error_message: "Authentication failed".to_string(),
                })
            ));
        }

        // Create session
        let session_id = Uuid::new_v4().to_string();
        let session = ClientSession {
            client_id: client_id.clone(),
            session_id: session_id.clone(),
            connected_at: std::time::Instant::now(),
            last_heartbeat: std::time::Instant::now(),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(client_id.clone(), session);
        }

        info!("Client {} connected with session {}", client_id, session_id);

        Ok(Message::new(
            MessageType::ConnectAck,
            Payload::ConnectAck(ConnectAckPayload {
                status: "success".to_string(),
                session_id,
            })
        ))
    }

    pub async fn handle_disconnect(&self, client_id: &str) -> Result<(), crate::Error> {
        {
            let mut sessions = self.sessions.write().await;
            if sessions.remove(client_id).is_some() {
                info!("Client {} disconnected", client_id);
            }
        }
        Ok(())
    }

    pub async fn handle_heartbeat(&self, client_id: String) -> Result<Message, crate::Error> {
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&client_id) {
                session.last_heartbeat = std::time::Instant::now();
                debug!("Heartbeat from client {}", client_id);
            } else {
                return Err(crate::Error::ClientNotFound(client_id));
            }
        }

        Ok(Message::new(
            MessageType::HeartbeatAck,
            Payload::HeartbeatAck(crate::message::HeartbeatAckPayload {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        ))
    }

    pub async fn route_message(&self, from_client_id: String, message: Message) -> Result<(), crate::Error> {
        match &message.payload {
            Payload::SignalOffer(payload) | Payload::SignalAnswer(payload) | Payload::SignalIceCandidate(payload) => {
                let target_client_id = &payload.target_client_id;
                
                // Check if target client exists
                {
                    let sessions = self.sessions.read().await;
                    if !sessions.contains_key(target_client_id) {
                        return Err(crate::Error::ClientNotFound(target_client_id.clone()));
                    }
                }

                // Route the message to the target client
                if let Err(e) = self.message_sender.send((target_client_id.clone(), message.clone())).await {
                    error!("Failed to route message to {}: {}", target_client_id, e);
                    return Err(crate::Error::Connection("Failed to route message".to_string()));
                }

                debug!("Routed message from {} to {}", from_client_id, target_client_id);
            }
            _ => {
                warn!("Unexpected message type for routing: {:?}", message.message_type);
            }
        }
        Ok(())
    }

    pub async fn get_active_sessions(&self) -> Vec<ClientSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn cleanup_expired_sessions(&self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        let mut sessions = self.sessions.write().await;
        
        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| now.duration_since(session.last_heartbeat) > max_age)
            .map(|(client_id, _)| client_id.clone())
            .collect();

        for client_id in expired {
            if let Some(_) = sessions.remove(&client_id) {
                info!("Removed expired session for client {}", client_id);
            }
        }
    }

    pub async fn broadcast_message(&self, message: Message, exclude_client: Option<&str>) -> Result<(), crate::Error> {
        let sessions = self.sessions.read().await;
        let client_ids: Vec<String> = sessions
            .keys()
            .filter(|id| exclude_client.map_or(true, |exclude| *id != exclude))
            .cloned()
            .collect();

        for client_id in client_ids {
            if let Err(e) = self.message_sender.send((client_id.clone(), message.clone())).await {
                error!("Failed to broadcast message to {}: {}", client_id, e);
            }
        }

        Ok(())
    }
} 