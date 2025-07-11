# Signal Manager Service Test Client

A Rust WebSocket test client for testing the Signal Manager Service.

## Features

- **Ping Test**: Sends a "PING" message and expects "PONG" response with 5-second timeout
- **JSON REGISTER Test**: Sends a JSON REGISTER message and expects a JSON response with status 200 and 10-second timeout
- **Async/await**: Fully asynchronous implementation using tokio
- **Error Handling**: Comprehensive error handling with detailed error messages
- **Logging**: Structured logging with tracing

## Building

```bash
cargo build --release
```

## Running

### Default (localhost:8080)
```bash
cargo run
```

### Custom WebSocket URL
```bash
WS_URL=ws://your-server:8080/ws cargo run
```

## Test Details

### Ping Test
- Sends: `"PING"`
- Expects: `"PONG"`
- Timeout: 5 seconds
- Validates exact string match

### JSON REGISTER Test
- Sends: JSON message with frame_id, type 2, and REGISTER payload
- Expects: JSON response with status 200
- Timeout: 10 seconds
- Validates JSON structure and status code

## Example Output

```
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Starting WebSocket test client
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Connecting to: ws://localhost:8080/ws
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Testing Ping functionality...
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Sending ping: PING
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Received pong response: PONG
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client ✅ Ping test passed!
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Ping test completed successfully
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Testing JSON REGISTER functionality...
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Sending JSON REGISTER message: {"frame_id":"...","type":2,"payload":{"type":"REGISTER","data":{"client_id":"...","timestamp":...}}}
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client Received JSON response: {"status":200,"message":"Registered successfully"}
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client ✅ JSON REGISTER test passed! Status: 200
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client JSON REGISTER test completed successfully
2024-01-15T10:30:00.000Z INFO  signal_manager_service_test_client All tests completed successfully!
```

## Error Handling

The client handles various error scenarios:
- Connection failures
- Timeout errors
- Unexpected message types
- JSON parsing errors
- WebSocket protocol errors

All errors include detailed context for debugging. 