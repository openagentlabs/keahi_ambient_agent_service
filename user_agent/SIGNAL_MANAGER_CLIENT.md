# Signal Manager Client Implementation

## Overview

This implementation adds a WebSocket client to the user agent that connects to the signal manager service. The client implements the binary protocol used by the signal manager service and provides visual feedback for connection status.

## Features

### Connection Management
- **Connect/Disconnect**: Manual connection control with visual feedback
- **Auto-reconnect**: Automatic reconnection with exponential backoff
- **Heartbeat**: Sends heartbeat every 5 seconds and validates responses
- **Timeout handling**: Disconnects and reconnects if no heartbeat response received

### Visual Feedback
- **Connection Status**: Real-time status indicator (Connected, Connecting, Reconnecting, Error)
- **Last Heartbeat**: Shows time since last successful heartbeat
- **Error Display**: Shows connection errors and protocol errors
- **Message Log**: Displays recent messages received from the server

### Configuration
- **TOML Config**: Configuration file (`app-config.toml`) with all connection parameters
- **Environment Variables**: Support for environment-based configuration
- **Default Values**: Sensible defaults for development

## Protocol Implementation

### Message Format
The client implements the binary message format used by the signal manager service:

```
[Start Byte (0xAA)] [Message Type (1 byte)] [UUID (16 bytes)] [Payload Type (1 byte)] [Payload Length (2 bytes)] [Payload (variable)]
```

### Message Types
- `0x20` - Register: Client registration with capabilities and metadata
- `0x21` - RegisterAck: Server acknowledgment of registration
- `0x04` - Heartbeat: Periodic ping with timestamp
- `0x05` - HeartbeatAck: Server acknowledgment of heartbeat
- `0xFF` - Error: Error messages from server

### Registration Process
1. Establish WebSocket connection
2. Send Register message with client ID, auth token, and capabilities
3. Wait for RegisterAck with status 200
4. Start heartbeat timer on successful registration

### Heartbeat Process
1. Send Heartbeat message every 5 seconds
2. Wait for HeartbeatAck response
3. Update last heartbeat timestamp
4. Disconnect and reconnect if no response received within timeout

## Files Structure

```
user_agent/
├── src/
│   ├── lib/
│   │   ├── signal-manager-client.ts    # Core WebSocket client
│   │   └── config.ts                   # Configuration loader
│   ├── hooks/
│   │   └── useSignalManager.ts         # React hook for client management
│   ├── components/
│   │   └── ConnectionStatus.tsx        # Visual status component
│   └── App.tsx                         # Main app with connection UI
├── app-config.toml                     # Configuration file
└── test-connection.html                # Standalone test page
```

## Configuration

The `app-config.toml` file contains all connection parameters:

```toml
[signal_manager]
url = "127.0.0.1"
port = 8080
client_id = "user_agent_client"
auth_token = "test_token_1"
version = "1.0.0"
heartbeat_interval = 5
timeout = 10
reconnect_attempts = 5
reconnect_delay = 1
```

## Usage

### Development
1. Start the signal manager service:
   ```bash
   cd signal_manager_service
   cargo run
   ```

2. Start the user agent:
   ```bash
   cd user_agent
   npm run dev
   ```

3. Open the application and click "Connect" to establish connection

### Testing
Use the standalone test page (`test-connection.html`) to test the WebSocket connection without the React app:

```bash
cd user_agent
python3 -m http.server 3000
# Open http://localhost:3000/test-connection.html
```

## Error Handling

The client handles various error scenarios:

- **Connection failures**: Automatic retry with exponential backoff
- **Protocol errors**: Display error messages and attempt reconnection
- **Heartbeat timeouts**: Disconnect and reconnect automatically
- **WebSocket errors**: Graceful error display and recovery

## Security

- **Authentication**: Uses auth tokens for client identification
- **TLS Support**: Ready for TLS/SSL connections (not implemented in test setup)
- **Input validation**: Validates all incoming messages and payloads

## Future Enhancements

- **TLS Support**: Add support for secure WebSocket connections
- **Message Encryption**: Implement message encryption for sensitive data
- **Advanced Reconnection**: Implement more sophisticated reconnection strategies
- **Metrics**: Add connection metrics and performance monitoring
- **Multiple Connections**: Support for multiple simultaneous connections 