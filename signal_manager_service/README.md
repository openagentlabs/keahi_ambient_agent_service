# Signal Manager Service

A high-performance WebSocket server for managing signaling between agents and backend services, implemented in Rust.

## Features

- **Binary WebSocket Protocol**: Custom binary message format for efficient communication
- **Authentication**: Secure client authentication with token-based validation
- **Session Management**: Robust session tracking and management
- **Message Routing**: Efficient message routing between connected clients
- **Heartbeat Support**: Keep-alive mechanism for connection health monitoring
- **High Performance**: Built with Tokio async runtime for high concurrency
- **Observability**: Comprehensive logging and metrics

## Protocol Specification

The service implements a custom binary WebSocket protocol with the following message structure:

```
[Start Byte (1 byte)] [Message Type (1 byte)] [Message UUID (16 bytes)] [Payload Type (1 byte)] [Payload Length (2 bytes)] [Payload (N bytes)]
```

### Message Types

- `CONNECT (0x01)`: Client connection request
- `CONNECT_ACK (0x02)`: Connection acknowledgment
- `DISCONNECT (0x03)`: Client disconnection notification
- `HEARTBEAT (0x04)`: Keep-alive heartbeat
- `HEARTBEAT_ACK (0x05)`: Heartbeat acknowledgment
- `SIGNAL_OFFER (0x10)`: WebRTC offer signal
- `SIGNAL_ANSWER (0x11)`: WebRTC answer signal
- `SIGNAL_ICE_CANDIDATE (0x12)`: ICE candidate signal
- `ERROR (0xFF)`: Error message

### Payload Types

- `BINARY (0x01)`: Raw binary data
- `JSON (0x02)`: JSON-encoded data
- `TEXT (0x03)`: Plain text data
- `PROTOBUF (0x04)`: Protocol Buffer encoded data
- `CBOR (0x05)`: CBOR encoded data

## Quick Start

### Prerequisites

- Rust 1.88.0 or later
- Cargo (included with Rust)

### Installation

1. Clone the repository and navigate to the service directory:
```bash
cd signal_manager_service
```

2. Build the service:
```bash
cargo build --release
```

3. Run the service:
```bash
cargo run --release
```

The service will start listening on `127.0.0.1:8080` by default.

### Configuration

The service can be configured using a TOML file. Create a `config.toml` file in the service directory:

```toml
[server]
host = "127.0.0.1"
port = 8080
max_connections = 1000
heartbeat_interval = 30

[firestore]
project_id = "your-project-id"
credentials_path = null

[auth]
token_secret = "your-secret-key-change-in-production"
token_expiry = 3600

[logging]
level = "info"
format = "json"
```

### Environment Variables

You can also configure the service using environment variables with the `SIGNAL_MANAGER_` prefix:

```bash
export SIGNAL_MANAGER_SERVER_HOST="0.0.0.0"
export SIGNAL_MANAGER_SERVER_PORT="8080"
export SIGNAL_MANAGER_AUTH_TOKEN_SECRET="your-secret"
```

## Usage

### Connecting to the Service

Clients can connect to the WebSocket server using the binary protocol. Here's an example of a connection message:

```rust
// Connect message (JSON payload)
let connect_message = Message::new(
    MessageType::Connect,
    Payload::Connect(ConnectPayload {
        client_id: "client_1".to_string(),
        auth_token: "valid_token".to_string(),
    })
);
```

### Authentication

The service supports token-based authentication. Valid tokens for testing:

- Client ID: `test_client_1`, Token: `test_token_1`
- Client ID: `test_client_2`, Token: `test_token_2`

### Message Routing

Once authenticated, clients can send signaling messages to other connected clients:

```rust
// Signal offer message
let signal_message = Message::new(
    MessageType::SignalOffer,
    Payload::SignalOffer(SignalPayload {
        target_client_id: "client_2".to_string(),
        signal_data: "base64_encoded_signal_data".to_string(),
    })
);
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture

The service is built with a modular architecture:

- **`main.rs`**: Application entry point and CLI handling
- **`lib.rs`**: Module definitions and exports
- **`config.rs`**: Configuration management
- **`error.rs`**: Custom error types
- **`message.rs`**: Binary protocol implementation
- **`server.rs`**: WebSocket server implementation
- **`session.rs`**: Session management
- **`auth.rs`**: Authentication handling

## Performance

The service is designed for high performance:

- **Async I/O**: Built on Tokio runtime for efficient async operations
- **Concurrent Connections**: Supports thousands of concurrent WebSocket connections
- **Memory Efficient**: Uses Arc and RwLock for shared state management
- **Binary Protocol**: Efficient binary message format for minimal overhead

## Monitoring

The service provides comprehensive logging using the `tracing` crate:

- Connection events (connect/disconnect)
- Message routing
- Authentication attempts
- Error conditions
- Performance metrics

Log levels can be configured via the `logging.level` setting.

## Security

- **Authentication**: All connections require valid authentication tokens
- **Session Validation**: Sessions are validated on each message
- **Input Validation**: All incoming messages are validated and sanitized
- **Error Handling**: Secure error responses that don't leak sensitive information

## Deployment

### Docker

Create a `Dockerfile`:

```dockerfile
FROM rust:1.88 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/signal-manager-service /usr/local/bin/
COPY --from=builder /app/config.toml /etc/signal-manager/
EXPOSE 8080
CMD ["signal-manager-service", "--config", "/etc/signal-manager/config.toml"]
```

### Systemd Service

Create `/etc/systemd/system/signal-manager.service`:

```ini
[Unit]
Description=Signal Manager Service
After=network.target

[Service]
Type=simple
User=signal-manager
ExecStart=/usr/local/bin/signal-manager-service
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 