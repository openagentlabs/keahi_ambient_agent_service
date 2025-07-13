#!/bin/bash

# Signal Manager Service Build Script
# This script builds the signal manager service with proper error handling

set -e  # Exit on any error

echo "Building Signal Manager Service..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the signal_manager_service directory."
    exit 1
fi

# Clean previous build artifacts
echo "Cleaning previous build artifacts..."
cargo clean

# Build the project
echo "Compiling signal manager service..."
cargo build

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📦 Binary location: target/debug/signal-manager-service"
    echo ""
    echo "To run the service:"
    echo "  cargo run"
    echo ""
    echo "To run with specific config:"
    echo "  cargo run -- --config /path/to/config.toml"
else
    echo "❌ Build failed!"
    exit 1
fi 