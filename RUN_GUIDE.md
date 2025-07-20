# Running the NBGN Voucher Backend

## Prerequisites

1. **PostgreSQL** - Database server
2. **Redis** - Caching and rate limiting
3. **Rust** - Latest stable version

## Step 1: Install Dependencies

### macOS
```bash
# Install PostgreSQL
brew install postgresql@15
brew services start postgresql@15

# Install Redis
brew install redis
brew services start redis

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Ubuntu/Debian
```bash
# Install PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib

# Install Redis
sudo apt install redis-server

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Step 2: Set Up Database

```bash
# Create database and user
sudo -u postgres psql

# In PostgreSQL console:
CREATE DATABASE nbgn_backend;
CREATE USER nbgn_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE nbgn_backend TO nbgn_user;
\q

# Test connection
psql -U nbgn_user -d nbgn_backend -h localhost
```

## Step 3: Configure Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit .env file
nano .env  # or use your favorite editor
```

Update the `.env` file with your values:
```env
# Database
DATABASE_URL=postgres://nbgn_user:your_secure_password@localhost/nbgn_backend

# Redis
REDIS_URL=redis://localhost:6379

# Arbitrum RPC
ETHEREUM_RPC_URL=https://arb1.arbitrum.io/rpc

# Contract addresses - Arbitrum One
ETHEREUM_NBGN_CONTRACT_ADDRESS=0x47F9CF7043C8A059f82a988C0B9fF73F0c3e6067
ETHEREUM_VOUCHER_CONTRACT_ADDRESS=0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6

# Backend Wallet Configuration
# IMPORTANT: Generate this with the wallet script!
BACKEND_PRIVATE_KEY=0x... # Run wallet generator first

# Server config
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Logging
RUST_LOG=info,nbgn_backend=debug

# Indexer config
INDEXER_START_BLOCK=0
INDEXER_POLL_INTERVAL_SECS=30
```

## Step 4: Generate Backend Wallet

```bash
# Create a simple Rust script to generate wallet
cat > generate_wallet.rs << 'EOF'
use ethers::prelude::*;

fn main() {
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    println!("Address: {:?}", wallet.address());
    println!("Private Key: 0x{}", hex::encode(wallet.signer().to_bytes()));
}
EOF

# Run it
cargo run --bin generate_wallet

# Copy the private key to your .env file
# Send the address to Jenny for updateBackendSigner()
```

## Step 5: Run Database Migrations

```bash
# Install sqlx-cli if needed
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Or let the app run them automatically on startup
```

## Step 6: Build and Run

### Development Mode
```bash
# Run in development with auto-reload
cargo run

# Or with specific log level
RUST_LOG=debug cargo run
```

### Production Mode
```bash
# Build optimized binary
cargo build --release

# Run the production binary
./target/release/nbgn-backend
```

## Step 7: Verify It's Working

1. **Check API Documentation**
   ```bash
   # Open in browser
   open http://localhost:8080/docs  # macOS
   xdg-open http://localhost:8080/docs  # Linux
   ```

2. **Health Check**
   ```bash
   curl http://localhost:8080/health
   # Should return: {"status":"healthy","service":"nbgn-voucher-backend"}
   ```

3. **Test Voucher Link Creation**
   ```bash
   curl -X POST http://localhost:8080/api/vouchers/link \
     -H "Content-Type: application/json" \
     -d '{"voucher_id":"0x0000000000000000000000000000000000000000000000000000000000000001"}'
   ```

## Common Issues

### PostgreSQL Connection Error
```
Error: Failed to create database pool
```
**Solution**: Check DATABASE_URL in .env and ensure PostgreSQL is running:
```bash
# Check PostgreSQL status
brew services list | grep postgresql  # macOS
sudo systemctl status postgresql  # Linux
```

### Redis Connection Error
```
Error: Failed to initialize Redis connection
```
**Solution**: Ensure Redis is running:
```bash
# Check Redis
redis-cli ping  # Should return PONG
```

### Missing Environment Variables
```
Error: Failed to load configuration
```
**Solution**: Ensure all required variables are in .env file

### Migration Errors
```
Error: Failed to run database migrations
```
**Solution**: Check database permissions and run migrations manually:
```bash
sqlx migrate run
```

## Running with Docker (Alternative)

Create a `docker-compose.yml`:
```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: nbgn_backend
      POSTGRES_USER: nbgn_user
      POSTGRES_PASSWORD: secure_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  app:
    build: .
    ports:
      - "8080:8080"
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgres://nbgn_user:secure_password@postgres/nbgn_backend
      REDIS_URL: redis://redis:6379
    env_file:
      - .env

volumes:
  postgres_data:
```

Then run:
```bash
docker-compose up
```

## Next Steps

1. Send your backend wallet address to Jenny
2. Wait for Jenny to call `updateBackendSigner(your_address)`
3. Test signature generation with a real voucher
4. Monitor logs for indexed voucher events
5. Share API docs with Harrison for frontend integration

## Monitoring

Watch the logs for:
- `Starting NBGN backend server` - Server started
- `Created voucher code XXX for voucher_id` - Voucher indexed
- `Generated claim authorization` - Signature created
- Rate limit warnings
- Database connection status