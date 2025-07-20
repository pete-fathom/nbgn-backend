# NBGN Backend

A high-performance Rust backend for the NBGN web3 application, providing blockchain indexing, caching, and API services.

## Features

- **Blockchain Event Indexing**: Automatically indexes Minted, Redeemed, and Burned events from the NBGN contract
- **PostgreSQL Database**: Stores transaction history and user profiles
- **Redis Caching**: Improves performance for frequently accessed data
- **RESTful API**: Provides endpoints for user profiles, transactions, and analytics
- **Real-time Updates**: WebSocket support for live transaction updates (pending implementation)

## Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Redis 6+
- Ethereum RPC endpoint (Infura, Alchemy, etc.)

## Setup

1. **Clone and navigate to the backend directory:**
```bash
cd nbgn-backend
```

2. **Copy the environment variables:**
```bash
cp .env.example .env
```

3. **Update `.env` with your configuration:**
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `ETH_RPC_URL`: Ethereum RPC endpoint
- `NBGN_CONTRACT_ADDRESS`: NBGN contract address

4. **Create the database:**
```bash
createdb nbgn_backend
```

5. **Run the server:**
```bash
cargo run
```

## API Endpoints

### User Endpoints
- `GET /api/users/{address}` - Get user profile with transaction stats
- `POST /api/users/username` - Set username (requires signature)

### Transaction Endpoints
- `GET /api/transactions/{address}` - Get user's transaction history
- `GET /api/transactions/recent` - Get recent transactions

### Analytics Endpoints
- `GET /api/analytics/overview` - Get 24h volume, users, and stats

### Contract Data
- `GET /api/contract/reserve-ratio` - Get current reserve ratio (cached)

## Development

Run with debug logging:
```bash
RUST_LOG=debug cargo run
```

Run tests:
```bash
cargo test
```

Build for production:
```bash
cargo build --release
```

## Architecture

- `src/api/` - HTTP API handlers and routes
- `src/contracts/` - Ethereum contract interfaces
- `src/db/` - Database models and migrations
- `src/services/` - Business logic (indexer, cache)
- `migrations/` - SQL migration files# nbgn-backend
