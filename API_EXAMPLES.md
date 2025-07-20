# NBGN Voucher Backend API Examples

## Quick Start

1. Start the server: `cargo run`
2. View interactive docs: http://localhost:8080/docs
3. OpenAPI spec: http://localhost:8080/openapi.yaml

## Example API Calls

### 1. Create a Voucher Link

```bash
# Create a shareable link for an on-chain voucher
curl -X POST http://localhost:8080/api/vouchers/link \
  -H "Content-Type: application/json" \
  -d '{
    "voucher_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "password": "SecurePass123"
  }'

# Response:
{
  "success": true,
  "code": "ABCD1234EFGH5678",
  "link": "/claim/ABCD1234EFGH5678"
}
```

### 2. Verify a Voucher

```bash
# Check if voucher is valid (without password)
curl -X POST http://localhost:8080/api/vouchers/verify \
  -H "Content-Type: application/json" \
  -d '{
    "code": "ABCD1234EFGH5678"
  }'

# With password
curl -X POST http://localhost:8080/api/vouchers/verify \
  -H "Content-Type: application/json" \
  -d '{
    "code": "ABCD1234EFGH5678",
    "password": "SecurePass123"
  }'

# Response:
{
  "valid": true,
  "voucher_id": "0x1234...",
  "amount": "1000000000000000000",
  "creator": "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E",
  "created_at": "2024-03-20T10:30:00Z"
}
```

### 3. Claim a Voucher (Get Signature)

```bash
# Generate claim authorization
curl -X POST http://localhost:8080/api/vouchers/claim \
  -H "Content-Type: application/json" \
  -d '{
    "code": "ABCD1234EFGH5678",
    "password": "SecurePass123",
    "recipient_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E"
  }'

# Response:
{
  "voucher_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "recipient": "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E",
  "amount": "1000000000000000000",
  "deadline": 1710765432,
  "signature": "0xabcdef1234567890...",
  "contract_address": "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6"
}
```

### 4. Update Claim Status

```bash
# After frontend submits transaction
curl -X POST http://localhost:8080/api/vouchers/claim-status \
  -H "Content-Type: application/json" \
  -d '{
    "code": "ABCD1234EFGH5678",
    "tx_hash": "0xdef456...",
    "success": true
  }'

# Response:
{
  "success": true,
  "message": "Voucher claimed successfully"
}
```

### 5. List User Vouchers

```bash
# Get vouchers created by user
curl "http://localhost:8080/api/vouchers/user/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E?type=created&page=0&limit=20"

# Get vouchers received by user
curl "http://localhost:8080/api/vouchers/user/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E?type=received&page=0&limit=20"

# Response:
{
  "vouchers": [
    {
      "code": "ABCD1234EFGH5678",
      "voucher_id": "0x1234...",
      "amount": "1000000000000000000",
      "claimed": false,
      "created_at": "2024-03-20T10:30:00Z"
    }
  ],
  "page": 0,
  "limit": 20,
  "type": "created"
}
```

### 6. Get User Profile

```bash
curl http://localhost:8080/api/users/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E

# Response:
{
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD7E",
  "username": "alice.eth",
  "created_at": "2024-01-01T00:00:00Z",
  "total_minted": "10000000000000000000000",
  "total_redeemed": "5000000000000000000000",
  "total_burned": "1000000000000000000000",
  "transaction_count": 42
}
```

### 7. Health Check

```bash
curl http://localhost:8080/health

# Response:
{
  "status": "healthy",
  "service": "nbgn-voucher-backend"
}
```

## Rate Limits

Different endpoints have different rate limits:

- `/api/vouchers/verify`: 10 requests per hour (special: 5 per code+IP combo)
- `/api/vouchers/claim`: 10 requests per hour per IP
- `/api/vouchers/link`: 20 requests per minute
- `/api/users/username`: 5 requests per hour
- Default: 200 requests per minute

Rate limit headers are included in responses:
- `X-RateLimit-Limit`: Request limit
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Unix timestamp when limit resets
- `Retry-After`: Seconds until next request (on 429 responses)

## Error Responses

### 400 Bad Request
```json
{
  "error": "Invalid recipient address",
  "message": "Address must be a valid Ethereum address"
}
```

### 404 Not Found
```json
{
  "error": "Voucher not found"
}
```

### 429 Too Many Requests
```json
{
  "error": "Too Many Requests",
  "message": "Rate limit exceeded. Maximum 10 requests per 3600 seconds allowed.",
  "retry_after": 2547
}
```

## Testing with Frontend

The frontend should:

1. Use `/api/vouchers/verify` to check voucher validity
2. Call `/api/vouchers/claim` to get the signature
3. Submit the claim transaction on-chain with the signature
4. Report success/failure via `/api/vouchers/claim-status`

## WebSocket Events (Future)

Coming soon: Real-time voucher creation notifications via WebSocket.