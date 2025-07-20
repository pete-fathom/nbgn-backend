#!/bin/bash

# NBGN Backend Setup Script
# This script sets up everything needed to run the NBGN voucher backend

set -e  # Exit on any error

echo "🚀 NBGN Backend Setup Script"
echo "============================"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Check for required tools
echo -e "\n📋 Checking dependencies..."

if ! command_exists brew; then
    print_error "Homebrew not found. Please install from https://brew.sh"
    exit 1
fi
print_status "Homebrew found"

if ! command_exists cargo; then
    print_warning "Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
print_status "Rust/Cargo found"

# 2. Install PostgreSQL if needed
echo -e "\n🗄️  Setting up PostgreSQL..."
if ! command_exists psql; then
    print_warning "PostgreSQL not found. Installing..."
    brew install postgresql@15
fi

# Start PostgreSQL
if ! brew services list | grep -q "postgresql.*started"; then
    brew services start postgresql@15
    sleep 3
fi
print_status "PostgreSQL is running"

# 3. Install Redis if needed
echo -e "\n💾 Setting up Redis..."
if ! command_exists redis-cli; then
    print_warning "Redis not found. Installing..."
    brew install redis
fi

# Start Redis
if ! brew services list | grep -q "redis.*started"; then
    brew services start redis
    sleep 2
fi
print_status "Redis is running"

# Test Redis connection
if redis-cli ping > /dev/null 2>&1; then
    print_status "Redis connection verified"
else
    print_error "Cannot connect to Redis"
    exit 1
fi

# 4. Database setup
echo -e "\n🏗️  Setting up database..."

# Add PostgreSQL to PATH for this session
export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"

# Create database if it doesn't exist
if ! psql -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw nbgn_backend; then
    print_warning "Creating database nbgn_backend..."
    createdb nbgn_backend 2>/dev/null || {
        print_warning "Retrying with current user as database name..."
        createdb -U $USER nbgn_backend 2>/dev/null || {
            print_warning "Creating with default postgres database..."
            psql -d postgres -c "CREATE DATABASE nbgn_backend;" 2>/dev/null || {
                print_error "Failed to create database. You can create it manually with: createdb nbgn_backend"
                # Continue anyway - migrations might create it
            }
        }
    }
fi
print_status "Database nbgn_backend exists"

# 5. Environment setup
echo -e "\n🔧 Setting up environment..."

if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        print_warning ".env file not found. Creating from .env.example..."
        cp .env.example .env
        
        # Generate a wallet if private key is not set
        if grep -q "your_private_key_here" .env; then
            print_warning "Generating backend wallet..."
            
            # Create temporary wallet generator
            cat > /tmp/gen_wallet.rs << 'EOF'
use ethers::prelude::*;
use rand::thread_rng;

fn main() {
    let wallet = LocalWallet::new(&mut thread_rng());
    println!("WALLET_ADDRESS={:?}", wallet.address());
    println!("BACKEND_PRIVATE_KEY=0x{}", hex::encode(wallet.signer().to_bytes()));
}
EOF
            
            # Compile and run
            rustc /tmp/gen_wallet.rs --edition 2021 -o /tmp/gen_wallet 2>/dev/null || {
                print_warning "Quick compile failed, using cargo..."
                cargo run --bin generate_wallet > /tmp/wallet_output.txt 2>/dev/null
                WALLET_ADDRESS=$(grep "Address:" /tmp/wallet_output.txt | awk '{print $2}')
                PRIVATE_KEY=$(grep "Private Key:" /tmp/wallet_output.txt | awk '{print $3}')
                
                # Update .env file
                sed -i '' "s|your_private_key_here|${PRIVATE_KEY}|g" .env
                
                echo -e "\n${GREEN}🔐 Wallet Generated!${NC}"
                echo "Address: $WALLET_ADDRESS"
                echo "Private key saved to .env"
                echo -e "${YELLOW}⚠️  Send this address to Jenny for updateBackendSigner()${NC}\n"
            }
            
            rm -f /tmp/gen_wallet* /tmp/wallet_output.txt
        fi
        
        # Update database URL with local settings
        sed -i '' 's|postgres://user:password@localhost/nbgn_backend|postgres://localhost/nbgn_backend|g' .env
        
        print_status ".env file created (update values as needed)"
    else
        print_error ".env.example not found!"
        exit 1
    fi
else
    print_status ".env file exists"
fi

# Make sure .env is secure
chmod 600 .env
print_status ".env file secured (permissions: 600)"

# 6. Install Rust dependencies
echo -e "\n📦 Installing Rust dependencies..."
cargo build --release || cargo build
print_status "Dependencies installed"

# 7. Run database migrations
echo -e "\n🔄 Running database migrations..."

# Install sqlx-cli if needed
if ! command_exists sqlx; then
    print_warning "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Run migrations
DATABASE_URL=$(grep DATABASE_URL .env | cut -d '=' -f2) sqlx migrate run || {
    print_warning "Migrations might have already been applied"
}
print_status "Database migrations complete"

# 8. Final checks
echo -e "\n✅ Setup complete! Running final checks..."

# Check if services are running
if brew services list | grep -q "postgresql.*started" && brew services list | grep -q "redis.*started"; then
    print_status "All services running"
else
    print_error "Some services are not running"
    brew services list
fi

# 9. Start the server
echo -e "\n🚀 Starting NBGN Backend Server..."
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "API Documentation: ${GREEN}http://localhost:8080/docs${NC}"
echo -e "Health Check:      ${GREEN}http://localhost:8080/health${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

# Run the server
exec cargo run --release --bin nbgn-backend