#!/bin/bash

# Stop script for NBGN Backend services

echo "🛑 Stopping NBGN Backend services..."

# Find and kill the Rust server process
if pgrep -f "target.*nbgn-backend" > /dev/null; then
    echo "Stopping NBGN backend server..."
    pkill -f "target.*nbgn-backend"
fi

# Optionally stop PostgreSQL and Redis
read -p "Stop PostgreSQL and Redis? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Stopping PostgreSQL..."
    brew services stop postgresql@15
    
    echo "Stopping Redis..."
    brew services stop redis
    
    echo "✅ All services stopped"
else
    echo "✅ Backend server stopped (PostgreSQL and Redis still running)"
fi