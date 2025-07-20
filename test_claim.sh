#!/bin/bash

# First, let's get a voucher code from the database
VOUCHER_CODE=$(psql -d nbgn_backend -t -c "SELECT code FROM voucher_codes WHERE NOT claimed AND NOT cancelled ORDER BY created_at DESC LIMIT 1;" | xargs)

if [ -z "$VOUCHER_CODE" ]; then
    echo "No unclaimed voucher found in database"
    exit 1
fi

echo "Testing with voucher code: $VOUCHER_CODE"

# Test verify endpoint first
echo -e "\n1. Testing verify endpoint..."
curl -X POST http://localhost:8080/api/vouchers/verify \
  -H "Content-Type: application/json" \
  -d "{\"code\": \"$VOUCHER_CODE\"}" | jq

# Test claim endpoint  
echo -e "\n2. Testing claim endpoint..."
RESPONSE=$(curl -s -X POST http://localhost:8080/api/vouchers/claim \
  -H "Content-Type: application/json" \
  -d "{
    \"code\": \"$VOUCHER_CODE\",
    \"recipient_address\": \"0x742d35Cc6634C0532925a3b844Bc9e7595f8fA8e\"
  }")

echo "$RESPONSE" | jq

# Extract signature from response
SIGNATURE=$(echo "$RESPONSE" | jq -r '.signature // empty')

if [ -n "$SIGNATURE" ]; then
    echo -e "\n3. Got signature: $SIGNATURE"
    
    # Get voucher details from DB
    psql -d nbgn_backend -c "SELECT voucher_id, amount, creator_address FROM voucher_codes WHERE code = '$VOUCHER_CODE';"
fi