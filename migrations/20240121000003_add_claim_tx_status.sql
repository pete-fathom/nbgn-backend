-- Add columns to track gasless claim transaction status
ALTER TABLE voucher_codes
ADD COLUMN IF NOT EXISTS claim_tx_status VARCHAR(20),
ADD COLUMN IF NOT EXISTS claim_tx_submitted_at TIMESTAMP WITH TIME ZONE;

-- Create index for monitoring pending transactions
CREATE INDEX IF NOT EXISTS idx_voucher_codes_claim_tx_status 
ON voucher_codes(claim_tx_status) 
WHERE claim_tx_status = 'pending';