#!/bin/bash

# Start Signal Manager Service and User Agent for testing

echo "Starting Signal Manager Service and User Agent..."

# Function to cleanup background processes
cleanup() {
    echo "Stopping services..."
    kill $SIGNAL_PID $USER_AGENT_PID 2>/dev/null
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Start Signal Manager Service
echo "Starting Signal Manager Service..."
cd /home/keith/Dev/keahi_ambient_agent_service/signal_manager_service
cargo run &
SIGNAL_PID=$!

# Wait a moment for the service to start
sleep 3

# Start User Agent
echo "Starting User Agent..."
cd /home/keith/Dev/keahi_ambient_agent_service/user_agent
npm run dev &
USER_AGENT_PID=$!

echo "Services started:"
echo "  Signal Manager Service PID: $SIGNAL_PID"
echo "  User Agent PID: $USER_AGENT_PID"
echo ""
echo "Press Ctrl+C to stop both services"

# Wait for background processes
wait 