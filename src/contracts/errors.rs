use ethers::prelude::*;
use serde::{Deserialize, Serialize};

// Custom errors from the NBGNVoucherClaim contract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoucherError {
    VoucherDoesNotExist,
    VoucherAlreadyClaimed,
    InvalidBackendSignature,
    SignatureExpired,
    InvalidAmount,
    TransferFailed,
}

impl VoucherError {
    pub fn from_revert_data(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        
        // Custom error selectors (first 4 bytes of keccak256(error_signature))
        match &data[0..4] {
            [0x3f, 0x68, 0x66, 0x85] => Some(VoucherError::VoucherDoesNotExist),
            [0xe1, 0x38, 0x29, 0xd1] => Some(VoucherError::VoucherAlreadyClaimed),
            [0x64, 0x86, 0x9d, 0xad] => Some(VoucherError::InvalidBackendSignature),
            [0x0b, 0xf3, 0x18, 0x87] => Some(VoucherError::SignatureExpired),
            [0x2c, 0x5a, 0x3a, 0xf5] => Some(VoucherError::InvalidAmount),
            [0x90, 0xb8, 0xec, 0x18] => Some(VoucherError::TransferFailed),
            _ => None,
        }
    }
    
    pub fn to_user_message(&self) -> &'static str {
        match self {
            VoucherError::VoucherDoesNotExist => "This voucher does not exist on-chain",
            VoucherError::VoucherAlreadyClaimed => "This voucher has already been claimed",
            VoucherError::InvalidBackendSignature => "Invalid signature - please try again",
            VoucherError::SignatureExpired => "The claim authorization has expired - please request a new one",
            VoucherError::InvalidAmount => "Invalid voucher amount",
            VoucherError::TransferFailed => "Token transfer failed - please check your wallet",
        }
    }
}