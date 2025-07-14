use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage, MaybeTlsStream};
use futures_util::{SinkExt, StreamExt};
use crate::signalmanager::error::SignalManagerError;
use crate::signalmanager::types::Message;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

// Type aliases to simplify complex types
type WebSocketSink = futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, WsMessage>;
type WebSocketStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WebSocketRead = futures_util::stream::SplitStream<WebSocketStream>;

pub struct WebSocketClient {
    url: String,
    sender: Option<mpsc::UnboundedSender<Message>>,
    receiver: Option<mpsc::UnboundedReceiver<Message>>,
    connection_handle: Option<tokio::task::JoinHandle<()>>,
}

impl WebSocketClient {
    pub fn new(url: String) -> Self {
        info!("[WebSocketClient] Creating new WebSocket client for URL: {}", url);
        Self {
            url,
            sender: None,
            receiver: None,
            connection_handle: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), SignalManagerError> {
        info!("[WebSocketClient] Connecting to WebSocket at {}", self.url);
        
        // Establish WebSocket connection
        let (ws_stream, _) = connect_async(&self.url).await
            .map_err(|e| SignalManagerError::WebSocketConnect(format!("Failed to connect: {}", e)))?;
        
        let (write, mut read) = ws_stream.split();
        
        // Create channels for message passing
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let (response_tx, response_rx) = mpsc::unbounded_channel::<Message>();
        
        // Store channels
        self.sender = Some(tx);
        self.receiver = Some(response_rx);

        // Share the write handle
        let write = Arc::new(Mutex::new(write));

        // Create a oneshot channel to signal when connection is ready
        let (ready_tx, ready_rx) = oneshot::channel::<Result<(), SignalManagerError>>();

        // Spawn the main connection task
        let write_clone = write.clone();
        let connection_task = tokio::spawn(async move {
            // Signal that connection is ready as soon as the task starts
            let _ = ready_tx.send(Ok(()));
            let result = Self::run_websocket_loop(write_clone, write, read, rx, response_tx).await;
            if let Err(e) = result {
                error!("[WebSocketClient] WebSocket loop error after connection: {}", e);
            }
        });

        self.connection_handle = Some(connection_task);

        // Wait for connection to be ready
        match ready_rx.await {
            Ok(Ok(())) => {
                info!("[WebSocketClient] WebSocket connection established successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("[WebSocketClient] WebSocket connection failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("[WebSocketClient] Connection task panicked");
                Err(SignalManagerError::WebSocketConnect("Connection task failed".to_string()))
            }
        }
    }

    async fn run_websocket_loop(
        write_clone: Arc<Mutex<WebSocketSink>>,
        write: Arc<Mutex<WebSocketSink>>,
        mut read: WebSocketRead,
        mut rx: mpsc::UnboundedReceiver<Message>,
        response_tx: mpsc::UnboundedSender<Message>,
    ) -> Result<(), SignalManagerError> {
        // Spawn sender task
        let sender_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message.to_binary() {
                    Ok(binary) => {
                        debug!("[WebSocketClient] Sending binary frame: {} bytes", binary.len());
                        let mut write_guard = write_clone.lock().await;
                        if let Err(e) = write_guard.send(WsMessage::Binary(binary)).await {
                            error!("[WebSocketClient] WebSocket send error: {e}");
                            return Err(SignalManagerError::WebSocketSend(e.to_string()));
                        }
                    }
                    Err(e) => {
                        error!("[WebSocketClient] Binary serialization error: {e}");
                        return Err(SignalManagerError::Serialization(e.to_string()));
                    }
                }
            }
            Ok::<(), SignalManagerError>(())
        });

        // Spawn receiver task
        let receiver_task = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(WsMessage::Binary(data)) => {
                        debug!("[WebSocketClient] Received binary frame: {} bytes", data.len());
                        match Message::from_binary(&data) {
                            Ok(message) => {
                                debug!("[WebSocketClient] Successfully deserialized message: {:?}", message.message_type);
                                if response_tx.send(message).is_err() {
                                    error!("[WebSocketClient] Failed to send message to receiver");
                                    return Err(SignalManagerError::WebSocketReceive("Failed to send message to receiver".to_string()));
                                }
                            }
                            Err(e) => {
                                error!("[WebSocketClient] Binary deserialization error: {e}");
                                return Err(SignalManagerError::Deserialization(e.to_string()));
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!("[WebSocketClient] WebSocket connection closed by server");
                        break;
                    }
                    Ok(WsMessage::Ping(data)) => {
                        debug!("[WebSocketClient] Received ping, sending pong");
                        let mut write_guard = write.lock().await;
                        if let Err(e) = write_guard.send(WsMessage::Pong(data)).await {
                            error!("[WebSocketClient] Failed to send pong: {}", e);
                            return Err(SignalManagerError::WebSocketSend(e.to_string()));
                        }
                    }
                    Ok(WsMessage::Pong(_)) => {
                        debug!("[WebSocketClient] Received pong");
                    }
                    Ok(WsMessage::Text(text)) => {
                        warn!("[WebSocketClient] Received text message (not expected): {}", text);
                    }
                    Err(e) => {
                        error!("[WebSocketClient] WebSocket receive error: {e}");
                        return Err(SignalManagerError::WebSocketReceive(e.to_string()));
                    }
                    _ => {
                        debug!("[WebSocketClient] Ignoring unsupported message type");
                    }
                }
            }
            Ok::<(), SignalManagerError>(())
        });

        // Wait for either task to complete
        tokio::select! {
            result = sender_task => {
                if let Err(e) = result {
                    error!("[WebSocketClient] Sender task failed: {e:?}");
                    return Err(SignalManagerError::WebSocketSend("Sender task failed".to_string()));
                }
            }
            result = receiver_task => {
                if let Err(e) = result {
                    error!("[WebSocketClient] Receiver task failed: {e:?}");
                    return Err(SignalManagerError::WebSocketReceive("Receiver task failed".to_string()));
                }
            }
        }

        info!("[WebSocketClient] WebSocket loop completed");
        Ok(())
    }

    pub fn send(&self, message: Message) -> Result<(), SignalManagerError> {
        debug!("[WebSocketClient] Queueing message for send: {:?}", message.message_type);
        if let Some(sender) = &self.sender {
            sender.send(message)
                .map_err(|_| SignalManagerError::WebSocketSend("Failed to send message".to_string()))?;
            Ok(())
        } else {
            error!("[WebSocketClient] Tried to send message but not connected");
            Err(SignalManagerError::NotConnected)
        }
    }

    pub async fn receive(&mut self) -> Result<Option<Message>, SignalManagerError> {
        debug!("[WebSocketClient] Waiting to receive message from WebSocket");
        if let Some(receiver) = &mut self.receiver {
            let msg = receiver.recv().await;
            debug!("[WebSocketClient] Received message from WebSocket: {:?}", msg.as_ref().map(|m| m.message_type));
            Ok(msg)
        } else {
            error!("[WebSocketClient] Tried to receive message but not connected");
            Err(SignalManagerError::NotConnected)
        }
    }

    pub fn is_connected(&self) -> bool {
        self.sender.is_some()
    }

    pub async fn close(&mut self) {
        info!("[WebSocketClient] Closing WebSocket connection");
        
        // Drop sender to signal shutdown
        self.sender = None;
        
        // Wait for connection task to complete
        if let Some(handle) = self.connection_handle.take() {
            let _ = handle.await;
        }
        
        info!("[WebSocketClient] WebSocket connection closed");
    }
} 