use tokio_tungstenite::tungstenite::Message as WsMessage;

pub async fn handle_ping(data: Vec<u8>) -> WsMessage {
    WsMessage::Pong(data)
} 