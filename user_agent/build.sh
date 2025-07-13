#!/bin/bash

# Build script for Tauri application
# This script compiles both the TypeScript frontend and Rust backend

set -e  # Exit on any error

echo "🚀 Starting build process for Tauri application..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the correct directory
if [ ! -f "package.json" ] || [ ! -d "src-tauri" ]; then
    print_error "This script must be run from the user_agent directory"
    exit 1
fi

print_status "Building TypeScript frontend..."

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    print_status "Installing npm dependencies..."
    npm install
fi

# Build the TypeScript frontend
print_status "Compiling TypeScript..."
npm run build

if [ $? -eq 0 ]; then
    print_success "TypeScript frontend compiled successfully"
else
    print_error "TypeScript compilation failed"
    exit 1
fi

print_status "Building Rust backend..."

# Change to src-tauri directory
cd src-tauri

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found in src-tauri directory"
    exit 1
fi

# Build the Rust backend
print_status "Compiling Rust code..."
cargo build --release

if [ $? -eq 0 ]; then
    print_success "Rust backend compiled successfully"
else
    print_error "Rust compilation failed"
    exit 1
fi

# Go back to user_agent directory
cd ..

print_status "Creating final Tauri application..."

# Build the final Tauri application
npx tauri build

if [ $? -eq 0 ]; then
    print_success "Tauri application built successfully!"
    print_status "Output files are in src-tauri/target/release/"
    
    # List the generated files
    if [ -d "src-tauri/target/release" ]; then
        echo ""
        print_status "Generated files:"
        
        # Main executable
        if [ -f "src-tauri/target/release/tauri-app" ]; then
            echo "  📦 Main executable: src-tauri/target/release/tauri-app"
        fi
        
        # Bundle files
        if [ -d "src-tauri/target/release/bundle" ]; then
            echo "  📦 Bundle packages:"
            find src-tauri/target/release/bundle -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" -o -name "*.exe" -o -name "*.msi" -o -name "*.app" | while read file; do
                echo "    - $file"
            done
        fi
    fi
else
    print_error "Tauri build failed"
    exit 1
fi

echo ""
print_success "🎉 Build process completed successfully!"
print_status "You can now run the application with: npx tauri dev" 