# Keahi Ambient Agent Service Documentation

Welcome to the documentation for the Keahi Ambient Agent Service project. This project consists of multiple services working together to provide ambient intelligence capabilities.

## Project Overview

The Keahi Ambient Agent Service is a comprehensive system designed to provide ambient intelligence through various specialized services. The project includes:

- **Signal Manager Service**: WebSocket-based real-time communication service
- **Infrastructure**: Terraform-based infrastructure management
- **Screen Capture Agent**: Visual monitoring and analysis capabilities

## Documentation Index

### Core Services

#### [Signal Manager Service Protocol](./signal_manager_service_protocol.md)
A comprehensive guide to the WebSocket-based real-time communication service that handles signaling between agents and backend services. This documentation includes:

- **Protocol Specification**: Detailed binary protocol documentation
- **WebSocket Endpoint**: Connection details and examples
- **Authentication Flow**: Token-based authentication process
- **Message Routing**: How messages are routed between clients
- **Mermaid Diagrams**: Visual representations of system architecture
- **Code Examples**: JavaScript and Rust implementation examples
- **Configuration Guide**: TOML and environment variable configuration
- **Troubleshooting**: Common issues and solutions

**Key Features:**
- Custom binary WebSocket protocol
- Token-based authentication
- Session management
- Real-time message routing
- Heartbeat monitoring
- TLS support for secure communication
- GCP Pub/Sub integration for event publishing

### Infrastructure

#### [Infrastructure Documentation](./infrastructure/)
Terraform-based infrastructure management for the Keahi Ambient Agent Service.

- **Foundation**: Core infrastructure components
- **Firestore**: Database configuration and setup
- **Deployment**: Automated deployment processes

### Design Documents

#### [Design Documentation](./design.md)
High-level design and architecture decisions for the Keahi Ambient Agent Service.

## Quick Start

### Signal Manager Service

1. **Build the service:**
   ```bash
   cd signal_manager_service
   cargo build --release
   ```

2. **Run the service:**
   ```bash
   cargo run --release
   ```

3. **Connect a client:**
   ```javascript
   const ws = new WebSocket('ws://127.0.0.1:8080/');
   ```

### Infrastructure Setup

1. **Initialize Terraform:**
   ```bash
   cd infrastructure/foundation
   terraform init
   ```

2. **Deploy infrastructure:**
   ```bash
   terraform apply
   ```

## Development

### Prerequisites

- **Rust**: 1.88.0 or later
- **Terraform**: Latest version
- **Google Cloud Platform**: Account and project setup
- **Docker**: For containerized deployments

### Testing

```bash
# Run all tests
cargo test

# Run specific service tests
cd signal_manager_service
cargo test
```

### Building

```bash
# Build all services
cargo build --release

# Build specific service
cd signal_manager_service
cargo build --release
```

## Architecture

The Keahi Ambient Agent Service follows a microservices architecture:

```mermaid
graph TB
    Client[Client Applications] --> SignalManager[Signal Manager Service]
    SignalManager --> GCP[Google Cloud Platform]
    SignalManager --> Firestore[Firestore Database]
    
    ScreenCapture[Screen Capture Agent] --> SignalManager
    Infrastructure[Terraform Infrastructure] --> GCP
    
    subgraph "Core Services"
        SignalManager
        ScreenCapture
    end
    
    subgraph "Infrastructure"
        Infrastructure
        Firestore
    end
```

## Contributing

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Add tests for new functionality**
5. **Submit a pull request**

## Support

For questions or issues:

1. Check the relevant service documentation
2. Review the troubleshooting sections
3. Open an issue on the project repository

## License

This project is proprietary software. All rights reserved.
