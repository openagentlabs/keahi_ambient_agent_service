use signal_manager_service::message::{Message, MessageType, Payload, ConnectPayload};

#[test]
fn test_message_creation() {
    let payload = Payload::Connect(ConnectPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
    });
    let message = Message::new(MessageType::Connect, payload);
    assert_eq!(message.message_type, MessageType::Connect);
}

#[test]
fn test_message_binary_serialization() {
    let payload = Payload::Connect(ConnectPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
    });
    let message = Message::new(MessageType::Connect, payload);
    let binary = message.to_binary().expect("Failed to serialize message");
    // Verify the binary format
    assert_eq!(binary[0], signal_manager_service::message::START_BYTE);
    assert_eq!(binary[1], MessageType::Connect as u8);
    assert_eq!(binary[18], signal_manager_service::message::PayloadType::Json as u8);
}

#[test]
fn test_message_types() {
    assert_eq!(MessageType::Connect as u8, 0x01);
    assert_eq!(MessageType::ConnectAck as u8, 0x02);
    assert_eq!(MessageType::Disconnect as u8, 0x03);
    assert_eq!(MessageType::Heartbeat as u8, 0x04);
    assert_eq!(MessageType::HeartbeatAck as u8, 0x05);
    assert_eq!(MessageType::SignalOffer as u8, 0x10);
    assert_eq!(MessageType::SignalAnswer as u8, 0x11);
    assert_eq!(MessageType::SignalIceCandidate as u8, 0x12);
    assert_eq!(MessageType::Error as u8, 0xFF);
}

#[test]
fn test_payload_types() {
    use signal_manager_service::message::PayloadType;
    assert_eq!(PayloadType::Binary as u8, 0x01);
    assert_eq!(PayloadType::Json as u8, 0x02);
    assert_eq!(PayloadType::Text as u8, 0x03);
    assert_eq!(PayloadType::Protobuf as u8, 0x04);
    assert_eq!(PayloadType::Cbor as u8, 0x05);
} 