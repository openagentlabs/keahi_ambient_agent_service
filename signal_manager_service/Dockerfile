FROM rust:1.88 as builder

WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy main.rs and copy the actual source code
RUN rm src/main.rs
COPY src ./src

# Build the actual application
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false signal-manager

# Create necessary directories
RUN mkdir -p /etc/signal-manager /var/log/signal-manager

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/signal-manager-service /usr/local/bin/

# Copy configuration files
COPY config.toml /etc/signal-manager/

# Set ownership
RUN chown -R signal-manager:signal-manager /etc/signal-manager /var/log/signal-manager

# Switch to non-root user
USER signal-manager

# Expose the WebSocket port
EXPOSE 8080

# Set the working directory
WORKDIR /etc/signal-manager

# Run the service
CMD ["signal-manager-service", "--config", "config.toml"] 