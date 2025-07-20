-- Users table
CREATE TABLE IF NOT EXISTS users (
    address VARCHAR(42) PRIMARY KEY,
    username VARCHAR(30) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) UNIQUE NOT NULL,
    block_number BIGINT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    user_address VARCHAR(42) NOT NULL,
    transaction_type VARCHAR(10) NOT NULL CHECK (transaction_type IN ('mint', 'redeem', 'burn')),
    eure_amount VARCHAR(78),
    nbgn_amount VARCHAR(78) NOT NULL,
    gas_used VARCHAR(78),
    gas_price VARCHAR(78),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_transactions_user_address ON transactions(user_address);
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_block_number ON transactions(block_number DESC);

-- Daily statistics table
CREATE TABLE IF NOT EXISTS daily_stats (
    date DATE PRIMARY KEY,
    total_volume VARCHAR(78),
    unique_users INTEGER,
    transaction_count INTEGER,
    average_tx_size VARCHAR(78),
    ending_supply VARCHAR(78),
    ending_reserves VARCHAR(78),
    reserve_ratio VARCHAR(78),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Block sync tracking
CREATE TABLE IF NOT EXISTS sync_status (
    id INTEGER PRIMARY KEY,
    last_indexed_block BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);