// AIDEV-NOTE: EIP-712 order signing for Polymarket CTF Exchange
// This uses a DIFFERENT domain than ClobAuth (which is for API key derivation)
// Domain: name="Polymarket CTF Exchange", version="1", chainId=137, verifyingContract=0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E

use alloy_primitives::{keccak256, Address, U256};
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use std::str::FromStr;

use crate::api::order::{SignedOrder, UnsignedOrder};
use crate::error::ApiError;

// CTF Exchange domain constants (different from ClobAuth!)
const CTF_EXCHANGE_NAME: &str = "Polymarket CTF Exchange";
const CTF_EXCHANGE_VERSION: &str = "1";
const CTF_CHAIN_ID: u64 = 137; // Polygon mainnet
const CTF_VERIFYING_CONTRACT: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";

// Order type hash - all 12 fields in order
// AIDEV-NOTE: Field order MUST match the contract exactly
const ORDER_TYPE_STRING: &str = "Order(uint256 salt,address maker,address signer,address taker,uint256 tokenId,uint256 makerAmount,uint256 takerAmount,uint256 expiration,uint256 nonce,uint256 feeRateBps,uint8 side,uint8 signatureType)";

/// Order signer for CTF Exchange orders
pub struct OrderSigner {
    signer: PrivateKeySigner,
    address: Address,
}

impl OrderSigner {
    /// Create a new order signer from a private key hex string
    pub fn from_private_key(private_key: &str) -> Result<Self, ApiError> {
        let key_str = private_key.strip_prefix("0x").unwrap_or(private_key);

        let signer = PrivateKeySigner::from_str(key_str)
            .map_err(|e| ApiError::Signing(format!("Invalid private key: {}", e)))?;

        let address = signer.address();

        Ok(Self { signer, address })
    }

    /// Get the wallet address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the address as a checksummed string
    pub fn address_string(&self) -> String {
        self.address.to_checksum(None)
    }

    /// Sign an order using EIP-712 for CTF Exchange
    pub async fn sign_order(&self, order: &UnsignedOrder) -> Result<SignedOrder, ApiError> {
        let domain_separator = self.build_domain_separator()?;
        let struct_hash = self.build_order_struct_hash(order)?;

        // EIP-712: \x19\x01 + domainSeparator + structHash
        let mut message = Vec::with_capacity(66);
        message.extend_from_slice(&[0x19, 0x01]);
        message.extend_from_slice(&domain_separator);
        message.extend_from_slice(&struct_hash);

        let digest = keccak256(&message);

        tracing::debug!("Order EIP-712 digest: 0x{}", hex::encode(&digest));

        let signature = self.signer
            .sign_hash(&digest.into())
            .await
            .map_err(|e| ApiError::Signing(format!("Failed to sign order: {}", e)))?;

        // Convert recovery id from 0/1 to 27/28 (Polymarket expects v as 27/28)
        let mut sig_bytes = signature.as_bytes().to_vec();
        if sig_bytes[64] < 27 {
            sig_bytes[64] += 27;
        }

        let sig_hex = format!("0x{}", hex::encode(&sig_bytes));
        tracing::debug!("Order signature: {}", sig_hex);

        Ok(SignedOrder {
            order: order.clone(),
            signature: sig_hex,
        })
    }

    /// Build EIP-712 domain separator for CTF Exchange
    /// AIDEV-NOTE: This includes verifyingContract, unlike ClobAuth domain
    fn build_domain_separator(&self) -> Result<[u8; 32], ApiError> {
        // Domain type includes verifyingContract
        let domain_type_hash = keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        );

        let name_hash = keccak256(CTF_EXCHANGE_NAME);
        let version_hash = keccak256(CTF_EXCHANGE_VERSION);

        // Parse the verifying contract address
        let contract_addr = Address::from_str(CTF_VERIFYING_CONTRACT)
            .map_err(|e| ApiError::Signing(format!("Invalid contract address: {}", e)))?;

        // Encode: typeHash + nameHash + versionHash + chainId + verifyingContract
        let mut encoded = Vec::with_capacity(160);
        encoded.extend_from_slice(domain_type_hash.as_slice());
        encoded.extend_from_slice(name_hash.as_slice());
        encoded.extend_from_slice(version_hash.as_slice());
        encoded.extend_from_slice(&U256::from(CTF_CHAIN_ID).to_be_bytes::<32>());
        // Address is left-padded with zeros to 32 bytes
        encoded.extend_from_slice(&[0u8; 12]);
        encoded.extend_from_slice(contract_addr.as_slice());

        Ok(*keccak256(&encoded))
    }

    /// Build EIP-712 struct hash for Order
    fn build_order_struct_hash(&self, order: &UnsignedOrder) -> Result<[u8; 32], ApiError> {
        let type_hash = keccak256(ORDER_TYPE_STRING);

        // Parse all fields
        let salt = parse_u256(&order.salt)?;
        let maker = parse_address(&order.maker)?;
        let signer = parse_address(&order.signer)?;
        let taker = parse_address(&order.taker)?;
        let token_id = parse_u256(&order.token_id)?;
        let maker_amount = parse_u256(&order.maker_amount)?;
        let taker_amount = parse_u256(&order.taker_amount)?;
        let expiration = parse_u256(&order.expiration)?;
        let nonce = parse_u256(&order.nonce)?;
        let fee_rate_bps = parse_u256(&order.fee_rate_bps)?;
        let side = U256::from(order.side.as_u8());
        let sig_type = U256::from(order.signature_type.as_u8());

        // Encode: typeHash + all 12 fields as 32 bytes each
        // Total: 13 * 32 = 416 bytes
        let mut encoded = Vec::with_capacity(416);
        encoded.extend_from_slice(type_hash.as_slice());

        // uint256 salt
        encoded.extend_from_slice(&salt.to_be_bytes::<32>());

        // address maker (left-padded to 32 bytes)
        encoded.extend_from_slice(&[0u8; 12]);
        encoded.extend_from_slice(maker.as_slice());

        // address signer
        encoded.extend_from_slice(&[0u8; 12]);
        encoded.extend_from_slice(signer.as_slice());

        // address taker
        encoded.extend_from_slice(&[0u8; 12]);
        encoded.extend_from_slice(taker.as_slice());

        // uint256 tokenId
        encoded.extend_from_slice(&token_id.to_be_bytes::<32>());

        // uint256 makerAmount
        encoded.extend_from_slice(&maker_amount.to_be_bytes::<32>());

        // uint256 takerAmount
        encoded.extend_from_slice(&taker_amount.to_be_bytes::<32>());

        // uint256 expiration
        encoded.extend_from_slice(&expiration.to_be_bytes::<32>());

        // uint256 nonce
        encoded.extend_from_slice(&nonce.to_be_bytes::<32>());

        // uint256 feeRateBps
        encoded.extend_from_slice(&fee_rate_bps.to_be_bytes::<32>());

        // uint8 side (stored as uint256)
        encoded.extend_from_slice(&side.to_be_bytes::<32>());

        // uint8 signatureType (stored as uint256)
        encoded.extend_from_slice(&sig_type.to_be_bytes::<32>());

        Ok(*keccak256(&encoded))
    }
}

/// Parse a string to U256, supporting both decimal and hex formats
fn parse_u256(s: &str) -> Result<U256, ApiError> {
    let s = s.trim();

    if s.starts_with("0x") || s.starts_with("0X") {
        // Hex format
        U256::from_str_radix(&s[2..], 16)
            .map_err(|e| ApiError::Signing(format!("Invalid hex U256 '{}': {}", s, e)))
    } else {
        // Decimal format
        U256::from_str_radix(s, 10)
            .map_err(|e| ApiError::Signing(format!("Invalid decimal U256 '{}': {}", s, e)))
    }
}

/// Parse an address string to Address
fn parse_address(s: &str) -> Result<Address, ApiError> {
    Address::from_str(s)
        .map_err(|e| ApiError::Signing(format!("Invalid address '{}': {}", s, e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::order::{OrderSide, SignatureType};

    #[tokio::test]
    async fn test_order_signing() {
        // Test with a known test private key (DO NOT use in production)
        let test_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let signer = OrderSigner::from_private_key(test_key).unwrap();

        let order = UnsignedOrder {
            salt: "12345".to_string(),
            maker: signer.address_string(),
            signer: signer.address_string(),
            taker: "0x0000000000000000000000000000000000000000".to_string(),
            token_id: "1234567890".to_string(),
            maker_amount: "1000000".to_string(), // 1 USDC
            taker_amount: "1000000".to_string(), // 1 share
            expiration: "1735689600".to_string(), // Some future timestamp
            nonce: "1".to_string(),
            fee_rate_bps: "0".to_string(),
            side: OrderSide::Buy,
            signature_type: SignatureType::Eoa,
        };

        let signed = signer.sign_order(&order).await;
        assert!(signed.is_ok());

        let signed_order = signed.unwrap();
        assert!(signed_order.signature.starts_with("0x"));
        assert_eq!(signed_order.signature.len(), 132); // 0x + 65 bytes = 0x + 130 hex chars
    }

    #[test]
    fn test_parse_u256() {
        assert!(parse_u256("12345").is_ok());
        assert!(parse_u256("0x1234").is_ok());
        assert_eq!(parse_u256("0x10").unwrap(), U256::from(16));
        assert_eq!(parse_u256("16").unwrap(), U256::from(16));
    }

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x0000000000000000000000000000000000000000");
        assert!(addr.is_ok());

        let invalid = parse_address("invalid");
        assert!(invalid.is_err());
    }
}
