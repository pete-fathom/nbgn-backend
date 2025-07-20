#!/bin/bash

# Create test database if it doesn't exist
createdb nbgn_backend_test 2>/dev/null || true

# Export test environment variables
export TEST_DATABASE_URL="postgres://localhost/nbgn_backend_test"
export TEST_REDIS_URL="redis://localhost:6379/1"
export RUST_LOG=debug

# Run tests
echo "Running integration tests..."
cargo test --test integration_tests -- --test-threads=1 --nocapture

echo "Tests completed!"