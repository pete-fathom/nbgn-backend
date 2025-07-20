-- Drop existing tables if they exist
DROP TABLE IF EXISTS voucher_codes CASCADE;
DROP TABLE IF EXISTS claim_attempts CASCADE;

-- Create voucher_codes table
CREATE TABLE voucher_codes (
    code VARCHAR(16) PRIMARY KEY,
    voucher_id VARCHAR(66) NOT NULL, -- bytes32 hex from blockchain
    password_hash VARCHAR(255), -- optional password protection
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    creator_address VARCHAR(42),
    amount VARCHAR(78), -- wei amount from VoucherCreated event
    on_chain_created_at TIMESTAMP WITH TIME ZONE,
    claimed BOOLEAN DEFAULT FALSE,
    claimed_by VARCHAR(42),
    claimed_at TIMESTAMP WITH TIME ZONE,
    claim_tx_hash VARCHAR(66)
);

-- Create claim_attempts table for rate limiting and security
CREATE TABLE claim_attempts (
    id SERIAL PRIMARY KEY,
    voucher_code VARCHAR(16),
    ip_address VARCHAR(45),
    attempted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    success BOOLEAN,
    recipient_address VARCHAR(42)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_voucher_codes_voucher_id ON voucher_codes(voucher_id);
CREATE INDEX IF NOT EXISTS idx_voucher_codes_creator ON voucher_codes(creator_address);
CREATE INDEX IF NOT EXISTS idx_voucher_codes_claimed ON voucher_codes(claimed);
CREATE INDEX IF NOT EXISTS idx_claim_attempts_code ON claim_attempts(voucher_code);
CREATE INDEX IF NOT EXISTS idx_claim_attempts_ip ON claim_attempts(ip_address);
CREATE INDEX IF NOT EXISTS idx_claim_attempts_time ON claim_attempts(attempted_at);