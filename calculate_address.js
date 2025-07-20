require('dotenv').config();
const crypto = require('crypto');

// Simple address calculation from private key (without ethers dependency)
function privateKeyToAddress(privateKey) {
    // Remove 0x prefix
    const key = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;
    
    // This is a simplified version - normally you'd use elliptic curve crypto
    // For now, just show the private key for verification
    console.log('Private key:', '0x' + key);
    console.log('Note: Run this with ethers to get the actual address');
}

const privateKey = process.env.BACKEND_PRIVATE_KEY || '0x0000000000000000000000000000000000000000000000000000000000000001';
privateKeyToAddress(privateKey);