# 🔐 Security Guidelines for NBGN Backend

## Critical: Private Key Management

### ⚠️ NEVER DO THIS:
- ❌ Commit private keys to git
- ❌ Log private keys
- ❌ Share private keys via Slack/email/Discord
- ❌ Use the same key for dev/staging/prod
- ❌ Store keys in plain text files

### ✅ ALWAYS DO THIS:
- ✅ Use environment variables
- ✅ Add .env to .gitignore
- ✅ Use secret management services in production
- ✅ Rotate keys quarterly
- ✅ Monitor wallet activity

## Local Development Setup

1. **Create .env file** (already in .gitignore):
```bash
cp .env.example .env
```

2. **Add your private key**:
```env
BACKEND_PRIVATE_KEY=0x... # Your generated private key
```

3. **Secure the file**:
```bash
chmod 600 .env  # Only you can read/write
```

## Production Security

### 1. AWS Secrets Manager (Recommended)
```rust
// src/config.rs
use aws_sdk_secretsmanager;

async fn get_private_key() -> String {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_secretsmanager::Client::new(&config);
    
    let secret = client
        .get_secret_value()
        .secret_id("nbgn/backend/private_key")
        .send()
        .await
        .expect("Failed to retrieve secret");
    
    secret.secret_string().unwrap().to_string()
}
```

### 2. HashiCorp Vault
```bash
vault kv put secret/nbgn/backend private_key=0x...
```

### 3. Kubernetes Secrets
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: nbgn-backend-secrets
type: Opaque
data:
  private_key: <base64-encoded-key>
```

## Wallet Security Checklist

- [ ] Private key stored securely
- [ ] .env added to .gitignore
- [ ] Minimal funds in wallet (gas only)
- [ ] Monitoring alerts configured
- [ ] Key rotation schedule set
- [ ] Access logs enabled
- [ ] Multi-sig for high-value operations

## Monitoring & Alerts

### 1. Balance Monitoring
```rust
// Add to your monitoring service
async fn check_wallet_balance() {
    let balance = provider.get_balance(wallet_address).await?;
    if balance < MIN_BALANCE {
        alert("Low wallet balance!");
    }
}
```

### 2. Transaction Monitoring
- Set up Arbitrum block explorer alerts
- Use services like Tenderly or OpenZeppelin Defender
- Log all signature generations

## Emergency Procedures

### If Private Key is Compromised:
1. **Immediately** contact Jenny to update backend signer
2. Generate new wallet
3. Update all environments
4. Review logs for unauthorized usage
5. File security incident report

### Key Rotation Process:
1. Generate new wallet quarterly
2. Update Jenny with new address
3. Wait for contract update confirmation
4. Update all environments
5. Monitor old wallet for 30 days

## Additional Security Measures

### Rate Limiting (Already Implemented)
- `/api/vouchers/verify`: 5 attempts per code+IP/hour
- `/api/vouchers/claim`: 10 attempts per IP/hour

### Input Validation
- All addresses validated
- Voucher codes sanitized
- SQL injection protected via parameterized queries

### CORS Configuration
- Restrict to known frontend domains in production
- Use environment-specific CORS settings

## Security Contacts

- **Backend Security**: Molly (you)
- **Contract Security**: Jenny
- **Frontend Security**: Harrison
- **Emergency**: Create incident in #security channel

Remember: The backend wallet only signs messages, never holds user funds!