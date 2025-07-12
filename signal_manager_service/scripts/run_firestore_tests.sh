#!/bin/bash

# Script to run Firestore integration tests
# This script sets up the environment and runs the Firestore integration tests

set -e

echo "Setting up Firestore integration test environment..."

# Check if GOOGLE_APPLICATION_CREDENTIALS is set
if [ -z "$GOOGLE_APPLICATION_CREDENTIALS" ]; then
    echo "Error: GOOGLE_APPLICATION_CREDENTIALS environment variable is not set"
    echo "Please set it to the path of your GCP service account key file"
    echo "Example: export GOOGLE_APPLICATION_CREDENTIALS=/path/to/your/service-account-key.json"
    exit 1
fi

# Check if the credentials file exists
if [ ! -f "$GOOGLE_APPLICATION_CREDENTIALS" ]; then
    echo "Error: Credentials file not found at $GOOGLE_APPLICATION_CREDENTIALS"
    exit 1
fi

echo "Using credentials file: $GOOGLE_APPLICATION_CREDENTIALS"

# Run the Firestore integration tests
echo "Running Firestore integration tests..."
cargo test --test firestore_integration_tests -- --ignored

echo "Firestore integration tests completed!" 