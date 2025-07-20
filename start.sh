#!/bin/bash

# Quick start script for NBGN Backend
# Use this when everything is already set up

echo "🚀 Starting NBGN Backend..."

# Check if services are running
if ! pgrep -x "postgres" > /dev/null; then
    echo "⚠️  PostgreSQL is not running. Starting..."
    brew services start postgresql@15
    sleep 2
fi

if ! pgrep -x "redis-server" > /dev/null; then
    echo "⚠️  Redis is not running. Starting..."
    brew services start redis
    sleep 2
fi

# Check .env exists
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found!"
    echo "Run ./setup.sh first to set up the environment"
    exit 1
fi

echo "✅ All services running"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📚 API Docs:   http://localhost:8080/docs"
echo "🏥 Health:     http://localhost:8080/health"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Start with cargo run (development) or cargo run --release (production)
if [ "$1" = "--release" ]; then
    echo "Running in release mode..."
    exec cargo run --release --bin nbgn-backend
else
    echo "Running in development mode..."
    exec cargo run --bin nbgn-backend
fi