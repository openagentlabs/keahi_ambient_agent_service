use signal_manager_service::message::{
    Message, MessageType, Payload, PayloadType, ConnectPayload, ConnectAckPayload,
    SignalPayload, ErrorPayload, HeartbeatPayload
};

#[test]
fn test_protocol_message_structure() {
    // Test that message structure follows the binary protocol specification
    let payload = Payload::Connect(ConnectPayload {
        client_id: "test_client".to_string(),
        auth_token: "test_token".to_string(),
    });
    
    let message = Message::new(MessageType::Connect, payload);
    let binary = message.to_binary().expect("Failed to serialize message");
    
    // Validate protocol structure:
    // [Start Byte (1 byte)] [Message Type (1 byte)] [Message UUID (16 bytes)] 
    // [Payload Type (1 byte)] [Payload Length (2 bytes)] [Payload (N bytes)]
    
    // Start byte validation
    assert_eq!(binary[0], signal_manager_service::message::START_BYTE);
    
    // Message type validation
    assert_eq!(binary[1], MessageType::Connect as u8);
    
    // UUID validation (16 bytes)
    assert_eq!(binary[2..18].len(), 16);
    
    // Payload type validation
    assert_eq!(binary[18], PayloadType::Json as u8);
    
    // Payload length validation (2 bytes, big endian)
    let payload_length = u16::from_be_bytes([binary[19], binary[20]]) as usize;
    assert_eq!(payload_length, binary[21..].len());
    
    // Total message length should be 21 + payload_length
    assert_eq!(binary.len(), 21 + payload_length);
}

#[test]
fn test_protocol_message_types() {
    // Validate all message types have correct byte values
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
fn test_protocol_payload_types() {
    // Validate all payload types have correct byte values
    assert_eq!(PayloadType::Binary as u8, 0x01);
    assert_eq!(PayloadType::Json as u8, 0x02);
    assert_eq!(PayloadType::Text as u8, 0x03);
    assert_eq!(PayloadType::Protobuf as u8, 0x04);
    assert_eq!(PayloadType::Cbor as u8, 0x05);
}

#[test]
fn test_protocol_serialization_deserialization() {
    // Test round-trip serialization and deserialization
    let original_message = Message::new(
        MessageType::Connect,
        Payload::Connect(ConnectPayload {
            client_id: "test_client_123".to_string(),
            auth_token: "test_token_456".to_string(),
        })
    );
    
    let binary = original_message.to_binary().expect("Failed to serialize");
    let deserialized_message = Message::from_binary(&binary).expect("Failed to deserialize");
    
    // Validate the deserialized message matches the original
    assert_eq!(original_message.message_type, deserialized_message.message_type);
    assert_eq!(original_message.uuid, deserialized_message.uuid);
    assert_eq!(original_message.payload_type, deserialized_message.payload_type);
    
    // Validate payload content
    match (&original_message.payload, &deserialized_message.payload) {
        (Payload::Connect(orig), Payload::Connect(deser)) => {
            assert_eq!(orig.client_id, deser.client_id);
            assert_eq!(orig.auth_token, deser.auth_token);
        }
        _ => panic!("Payload types don't match"),
    }
}

#[test]
fn test_protocol_connect_ack_message() {
    let payload = Payload::ConnectAck(ConnectAckPayload {
        status: "success".to_string(),
        session_id: "session_123".to_string(),
    });
    
    let message = Message::new(MessageType::ConnectAck, payload);
    let binary = message.to_binary().expect("Failed to serialize");
    
    // Validate protocol structure
    assert_eq!(binary[0], signal_manager_service::message::START_BYTE);
    assert_eq!(binary[1], MessageType::ConnectAck as u8);
    assert_eq!(binary[18], PayloadType::Json as u8);
    
    // Test deserialization
    let deserialized = Message::from_binary(&binary).expect("Failed to deserialize");
    assert_eq!(deserialized.message_type, MessageType::ConnectAck);
}

#[test]
fn test_protocol_signal_message() {
    let payload = Payload::SignalOffer(SignalPayload {
        target_client_id: "target_client".to_string(),
        signal_data: "base64_encoded_signal_data".to_string(),
    });
    
    let message = Message::new(MessageType::SignalOffer, payload);
    let binary = message.to_binary().expect("Failed to serialize");
    
    // Validate protocol structure
    assert_eq!(binary[0], signal_manager_service::message::START_BYTE);
    assert_eq!(binary[1], MessageType::SignalOffer as u8);
    assert_eq!(binary[18], PayloadType::Json as u8);
    
    // Test deserialization
    let deserialized = Message::from_binary(&binary).expect("Failed to deserialize");
    assert_eq!(deserialized.message_type, MessageType::SignalOffer);
}

#[test]
fn test_protocol_error_message() {
    let payload = Payload::Error(ErrorPayload {
        error_code: 1,
        error_message: "Authentication failed".to_string(),
    });
    
    let message = Message::new(MessageType::Error, payload);
    let binary = message.to_binary().expect("Failed to serialize");
    
    // Validate protocol structure
    assert_eq!(binary[0], signal_manager_service::message::START_BYTE);
    assert_eq!(binary[1], MessageType::Error as u8);
    assert_eq!(binary[18], PayloadType::Json as u8);
    
    // Test deserialization
    let deserialized = Message::from_binary(&binary).expect("Failed to deserialize");
    assert_eq!(deserialized.message_type, MessageType::Error);
}

#[test]
fn test_protocol_heartbeat_message() {
    let payload = Payload::Heartbeat(HeartbeatPayload {
        timestamp: 1234567890,
    });
    
    let message = Message::new(MessageType::Heartbeat, payload);
    let binary = message.to_binary().expect("Failed to serialize");
    
    // Validate protocol structure
    assert_eq!(binary[0], signal_manager_service::message::START_BYTE);
    assert_eq!(binary[1], MessageType::Heartbeat as u8);
    assert_eq!(binary[18], PayloadType::Json as u8);
    
    // Test deserialization
    let deserialized = Message::from_binary(&binary).expect("Failed to deserialize");
    assert_eq!(deserialized.message_type, MessageType::Heartbeat);
}

#[test]
fn test_protocol_invalid_message_handling() {
    // Test handling of invalid start byte
    let invalid_data = vec![0x00; 22]; // Wrong start byte
    assert!(Message::from_binary(&invalid_data).is_err());
    
    // Test handling of message too short
    let short_data = vec![0xAA, 0x01]; // Only start byte and message type
    assert!(Message::from_binary(&short_data).is_err());
    
    // Test handling of invalid message type
    let mut invalid_type_data = vec![0xAA, 0x99]; // Invalid message type
    invalid_type_data.extend_from_slice(&vec![0x00; 20]); // Rest of required bytes
    assert!(Message::from_binary(&invalid_type_data).is_err());
    
    // Test handling of invalid payload type
    let mut invalid_payload_data = vec![0xAA, 0x01]; // Valid start and message type
    invalid_payload_data.extend_from_slice(&vec![0x00; 16]); // UUID
    invalid_payload_data.push(0x99); // Invalid payload type
    invalid_payload_data.extend_from_slice(&[0x00, 0x00]); // Payload length
    assert!(Message::from_binary(&invalid_payload_data).is_err());
}

#[test]
fn test_protocol_payload_length_validation() {
    let payload = Payload::Connect(ConnectPayload {
        client_id: "test".to_string(),
        auth_token: "token".to_string(),
    });
    
    let message = Message::new(MessageType::Connect, payload);
    let mut binary = message.to_binary().expect("Failed to serialize");
    
    // Test with mismatched payload length
    binary[19] = 0xFF;
    binary[20] = 0xFF; // Set payload length to maximum
    
    assert!(Message::from_binary(&binary).is_err());
    
    // Test with zero payload length
    let mut zero_length_binary = message.to_binary().expect("Failed to serialize");
    zero_length_binary[19] = 0x00;
    zero_length_binary[20] = 0x00;
    
    // This should still work for empty payloads
    let _result = Message::from_binary(&zero_length_binary);
    // The result depends on whether empty payloads are valid in your protocol
}

#[test]
fn test_protocol_uuid_handling() {
    // Test that UUIDs are properly handled in the protocol
    let payload = Payload::Connect(ConnectPayload {
        client_id: "test".to_string(),
        auth_token: "token".to_string(),
    });
    
    let message = Message::new(MessageType::Connect, payload);
    let binary = message.to_binary().expect("Failed to serialize");
    
    // Extract UUID from binary (bytes 2-17)
    let uuid_bytes = &binary[2..18];
    let uuid = uuid::Uuid::from_slice(uuid_bytes).expect("Failed to parse UUID");
    
    // Verify UUID is valid
    assert_eq!(uuid.get_version_num(), 4); // Should be version 4 UUID
    assert_eq!(uuid, message.uuid);
}

#[test]
fn test_protocol_message_size_limits() {
    // Test with very large payload to ensure protocol handles size correctly
    let large_payload = Payload::Connect(ConnectPayload {
        client_id: "a".repeat(1000),
        auth_token: "b".repeat(1000),
    });
    
    let message = Message::new(MessageType::Connect, large_payload);
    let binary = message.to_binary().expect("Failed to serialize large message");
    
    // Verify the message can be deserialized
    let deserialized = Message::from_binary(&binary).expect("Failed to deserialize large message");
    assert_eq!(deserialized.message_type, MessageType::Connect);
    
    // Verify payload length is correctly encoded
    let payload_length = u16::from_be_bytes([binary[19], binary[20]]) as usize;
    assert_eq!(payload_length, binary[21..].len());
} 