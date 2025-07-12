use crate::config::Config;
use crate::message::{Message, Payload};
use crate::session::SessionManager;
use crate::auth::AuthManager;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, Mutex};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{error, info, warn};
use native_tls::{TlsAcceptor, Identity};
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;
use std::fs::File;
use std::io::Read;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;
use crate::frame_handlers;
use crate::type_two_handlers::register::RegisterHandler;
use crate::webrtc_handlers::{WebRTCRoomCreateHandler, WebRTCRoomJoinHandler, WebRTCRoomLeaveHandler};

/// Context for message handling operations
struct MessageHandlerContext<'a> {
    session_manager: &'a Arc<SessionManager>,
    client_id: &'a Arc<Mutex<Option<String>>>,
    connections: &'a Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
    tx: &'a tokio::sync::mpsc::Sender<Message>,
    register_handler: &'a RegisterHandler,
    webrtc_room_create_handler: &'a WebRTCRoomCreateHandler,
    webrtc_room_join_handler: &'a WebRTCRoomJoinHandler,
    webrtc_room_leave_handler: &'a WebRTCRoomLeaveHandler,
}


#[derive(Clone)]
pub struct WebSocketServer {
    config: Arc<Config>,
    #[allow(dead_code)]
    auth_manager: Arc<AuthManager>,
    session_manager: Arc<SessionManager>,
    connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
    tls_acceptor: Option<TokioTlsAcceptor>,
    register_handler: RegisterHandler,
    webrtc_room_create_handler: WebRTCRoomCreateHandler,
    webrtc_room_join_handler: WebRTCRoomJoinHandler,
    webrtc_room_leave_handler: WebRTCRoomLeaveHandler,
}

impl WebSocketServer {
    pub fn new(config: Config) -> Result<Self, crate::Error> {
        let config = Arc::new(config);
        let auth_manager = Arc::new(AuthManager::new(config.clone()));
        let (session_manager, message_receiver) = SessionManager::new(auth_manager.clone());
        let session_manager = Arc::new(session_manager);

        // Initialize handlers
        let register_handler = RegisterHandler::new(config.clone());
        let webrtc_room_create_handler = WebRTCRoomCreateHandler::new(config.clone());
        let webrtc_room_join_handler = WebRTCRoomJoinHandler::new(config.clone());
        let webrtc_room_leave_handler = WebRTCRoomLeaveHandler::new(config.clone());

        // Initialize TLS if enabled
        let tls_acceptor = if config.server.tls_enabled {
            Self::init_tls_acceptor(&config)?
        } else {
            None
        };

        // Start message routing task
        let session_manager_clone = session_manager.clone();
        let connections_clone = Arc::new(RwLock::new(HashMap::new()));
        let connections_for_task = connections_clone.clone();
        
        tokio::spawn(async move {
            Self::message_routing_task(message_receiver, session_manager_clone, connections_for_task).await;
        });

        Ok(Self {
            config,
            auth_manager,
            session_manager,
            connections: connections_clone,
            tls_acceptor,
            register_handler,
            webrtc_room_create_handler,
            webrtc_room_join_handler,
            webrtc_room_leave_handler,
        })
    }

    fn init_tls_acceptor(config: &Config) -> Result<Option<TokioTlsAcceptor>, crate::Error> {
        if !config.server.tls_enabled {
            return Ok(None);
        }

        if config.server.tls_cert_path.is_empty() || config.server.tls_key_path.is_empty() {
            return Err(crate::Error::Config(config::ConfigError::NotFound(
                "TLS certificate or key path not configured".to_string()
            )));
        }

        // Load certificate and private key
        let mut cert_file = File::open(&config.server.tls_cert_path)
            .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
        let mut key_file = File::open(&config.server.tls_key_path)
            .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;

        let mut cert_data = Vec::new();
        let mut key_data = Vec::new();
        
        cert_file.read_to_end(&mut cert_data)
            .map_err(crate::Error::Io)?;
        key_file.read_to_end(&mut key_data)
            .map_err(crate::Error::Io)?;

        let identity = Identity::from_pkcs8(&cert_data, &key_data)
            .map_err(|e| crate::Error::Config(config::ConfigError::NotFound(e.to_string())))?;

        let acceptor = TlsAcceptor::builder(identity)
            .build()
            .map_err(|e| crate::Error::Config(config::ConfigError::NotFound(e.to_string())))?;

        let tokio_acceptor = TokioTlsAcceptor::from(acceptor);

        info!("TLS acceptor initialized successfully");
        Ok(Some(tokio_acceptor))
    }

    pub async fn run(&self) -> Result<(), crate::Error> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(&addr).await?;
        
        info!("WebSocket server listening on {} (TLS: {})", addr, self.config.server.tls_enabled);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    
                    let session_manager = self.session_manager.clone();
                    let connections = self.connections.clone();
                    let tls_acceptor = self.tls_acceptor.clone();
                    
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, session_manager, connections, tls_acceptor).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        session_manager: Arc<SessionManager>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
        tls_acceptor: Option<TokioTlsAcceptor>,
    ) -> Result<(), crate::Error> {
        if let Some(acceptor) = tls_acceptor {
            self.handle_tls_connection(stream, session_manager, connections, acceptor).await
        } else {
            self.handle_plain_connection(stream, session_manager, connections).await
        }
    }

    async fn handle_tls_connection(
        &self,
        stream: TcpStream,
        session_manager: Arc<SessionManager>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
        acceptor: TokioTlsAcceptor,
    ) -> Result<(), crate::Error> {
        let tls_stream = acceptor.accept(stream).await
            .map_err(|e| crate::Error::Connection(format!("TLS handshake failed: {e}")))?;
        let ws_stream = accept_async(tls_stream).await?;
        self.handle_ws_stream(ws_stream, session_manager, connections).await
    }

    async fn handle_plain_connection(
        &self,
        stream: TcpStream,
        session_manager: Arc<SessionManager>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
    ) -> Result<(), crate::Error> {
        let ws_stream = accept_async(stream).await?;
        self.handle_ws_stream(ws_stream, session_manager, connections).await
    }

    async fn handle_ws_stream<S>(
        &self,
        ws_stream: WebSocketStream<S>,
        session_manager: Arc<SessionManager>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
    ) -> Result<(), crate::Error>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {

        let (ws_sender, mut ws_receiver) = ws_stream.split();
        let ws_sender = Arc::new(Mutex::new(ws_sender));
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);
        let client_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let session_manager_clone = session_manager.clone();
        let connections_clone = connections.clone();
        let tx_clone = tx.clone();
        let client_id_in = client_id.clone();
        let ws_sender_in = ws_sender.clone();
        let register_handler = self.register_handler.clone();
        let webrtc_room_create_handler = self.webrtc_room_create_handler.clone();
        let webrtc_room_join_handler = self.webrtc_room_join_handler.clone();
        let webrtc_room_leave_handler = self.webrtc_room_leave_handler.clone();
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(WsMessage::Binary(data)) => {
                        match Message::from_binary(&data) {
                            Ok(message) => {
                                let context = MessageHandlerContext {
                                    session_manager: &session_manager_clone,
                                    client_id: &client_id_in,
                                    connections: &connections_clone,
                                    tx: &tx_clone,
                                    register_handler: &register_handler,
                                    webrtc_room_create_handler: &webrtc_room_create_handler,
                                    webrtc_room_join_handler: &webrtc_room_join_handler,
                                    webrtc_room_leave_handler: &webrtc_room_leave_handler,
                                };
                                if let Err(e) = Self::handle_message(&message, context).await {
                                    error!("Error handling message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                let preview = data.iter().take(16).map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
                                warn!("Dropped invalid frame: {} ({} bytes, preview: [{}])", e, data.len(), preview);
                                // Continue listening for more frames
                                continue;
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!("Client disconnected");
                        break;
                    }
                    Ok(WsMessage::Ping(data)) => {
                        if let Err(e) = ws_sender_in.lock().await.send(frame_handlers::ping::handle_ping(data).await).await {
                            error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {
                        warn!("Unsupported message type");
                    }
                }
            }
        });
        let ws_sender_out = ws_sender.clone();
        let outgoing_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Ok(binary) = message.to_binary() {
                    if let Err(e) = ws_sender_out.lock().await.send(WsMessage::Binary(binary)).await {
                        error!("Failed to send message: {}", e);
                        break;
                    }
                }
            }
        });
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }
        if let Some(id) = client_id.lock().await.as_ref() {
            session_manager.handle_disconnect(id).await?;
            let mut connections = connections.write().await;
            connections.remove(id);
        }
        Ok(())
    }

    async fn handle_message(
        message: &Message,
        context: MessageHandlerContext<'_>,
    ) -> Result<(), crate::Error> {
        match &message.payload {
            Payload::Connect(payload) => {
                let response = context.session_manager.handle_connect(payload.client_id.clone(), payload.auth_token.clone()).await?;
                if let Payload::ConnectAck(ack) = &response.payload {
                    if ack.status == "success" {
                        *context.client_id.lock().await = Some(payload.client_id.clone());
                        let mut connections = context.connections.write().await;
                        connections.insert(payload.client_id.clone(), context.tx.clone());
                        info!("Client {} connected successfully", payload.client_id);
                    }
                }
                context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
            }
            Payload::Disconnect(_payload) => {
                if let Some(id) = context.client_id.lock().await.as_ref() {
                    context.session_manager.handle_disconnect(id).await?;
                    let mut connections = context.connections.write().await;
                    connections.remove(id);
                }
            }
            Payload::Heartbeat(_) => {
                if let Some(id) = context.client_id.lock().await.as_ref() {
                    let response = context.session_manager.handle_heartbeat(id.clone()).await?;
                    context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                }
            }
            Payload::Register(_) => {
                match context.register_handler.handle_register(message.clone()).await {
                    Ok(response) => {
                        context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                    Err(e) => {
                        error!("Failed to handle register message: {}", e);
                        let error_message = Message::new(
                            crate::message::MessageType::Error,
                            crate::message::Payload::Error(crate::message::ErrorPayload {
                                error_code: 1,
                                error_message: format!("Internal server error: {e}"),
                            }),
                        );
                        context.tx.send(error_message).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                }
            }
            Payload::Unregister(_) => {
                match context.register_handler.handle_unregister(message.clone()).await {
                    Ok(response) => {
                        context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                    Err(e) => {
                        error!("Failed to handle unregister message: {}", e);
                        let error_message = Message::new(
                            crate::message::MessageType::Error,
                            crate::message::Payload::Error(crate::message::ErrorPayload {
                                error_code: 1,
                                error_message: format!("Internal server error: {e}"),
                            }),
                        );
                        context.tx.send(error_message).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                }
            }
            Payload::SignalOffer(_) | Payload::SignalAnswer(_) | Payload::SignalIceCandidate(_) => {
                if let Some(id) = context.client_id.lock().await.as_ref() {
                    context.session_manager.route_message(id.clone(), message.clone()).await?;
                }
            }
            Payload::WebRTCRoomCreate(_) => {
                match context.webrtc_room_create_handler.handle_room_create(message.clone()).await {
                    Ok(response) => {
                        context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                    Err(e) => {
                        error!("Failed to handle WebRTC room create message: {}", e);
                        let error_message = Message::new(
                            crate::message::MessageType::Error,
                            crate::message::Payload::Error(crate::message::ErrorPayload {
                                error_code: 1,
                                error_message: format!("Internal server error: {e}"),
                            }),
                        );
                        context.tx.send(error_message).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                }
            }
            Payload::WebRTCRoomJoin(_) => {
                match context.webrtc_room_join_handler.handle_room_join(message.clone()).await {
                    Ok(response) => {
                        context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                    Err(e) => {
                        error!("Failed to handle WebRTC room join message: {}", e);
                        let error_message = Message::new(
                            crate::message::MessageType::Error,
                            crate::message::Payload::Error(crate::message::ErrorPayload {
                                error_code: 1,
                                error_message: format!("Internal server error: {e}"),
                            }),
                        );
                        context.tx.send(error_message).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                }
            }
            Payload::WebRTCRoomLeave(_) => {
                match context.webrtc_room_leave_handler.handle_room_leave(message.clone()).await {
                    Ok(response) => {
                        context.tx.send(response).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                    Err(e) => {
                        error!("Failed to handle WebRTC room leave message: {}", e);
                        let error_message = Message::new(
                            crate::message::MessageType::Error,
                            crate::message::Payload::Error(crate::message::ErrorPayload {
                                error_code: 1,
                                error_message: format!("Internal server error: {e}"),
                            }),
                        );
                        context.tx.send(error_message).await.map_err(|e| crate::Error::Connection(e.to_string()))?;
                    }
                }
            }
            _ => {
                warn!("Unhandled message type: {:?}", message.message_type);
            }
        }
        Ok(())
    }

    async fn message_routing_task(
        mut receiver: tokio::sync::mpsc::Receiver<(String, Message)>,
        _session_manager: Arc<SessionManager>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
    ) {
        while let Some((client_id, message)) = receiver.recv().await {
            let connections = connections.read().await;
            if let Some(tx) = connections.get(&client_id) {
                if let Err(e) = tx.send(message).await {
                    error!("Failed to send message to client {}: {}", client_id, e);
                }
            }
        }
    }
} 