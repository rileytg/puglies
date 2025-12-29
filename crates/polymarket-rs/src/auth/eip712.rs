// AIDEV-NOTE: EIP-712 typed data signing for Polymarket authentication
// Used to derive API keys and sign orders
// See: https://docs.polymarket.com/developers/CLOB/authentication

use alloy_primitives::{keccak256, Address, U256};
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use std::str::FromStr;

use crate::error::ApiError;

// Polymarket uses a specific EIP-712 domain
const POLYMARKET_DOMAIN_NAME: &str = "ClobAuthDomain";
const POLYMARKET_DOMAIN_VERSION: &str = "1";
const POLYMARKET_CHAIN_ID: u64 = 137; // Polygon mainnet
const AUTH_MESSAGE: &str = "This message attests that I control the given wallet";

/// Polymarket signer for authentication and order signing
pub struct PolymarketSigner {
    signer: PrivateKeySigner,
    address: Address,
}

impl PolymarketSigner {
    /// Create a new signer from a private key hex string
    pub fn from_private_key(private_key: &str) -> Result<Self, ApiError> {
        // Remove 0x prefix if present
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
        // Use Display trait which provides checksummed format
        self.address.to_checksum(None)
    }

    /// Build EIP-712 struct hash for ClobAuth
    /// Type: ClobAuth(address address,string timestamp,uint256 nonce,string message)
    fn build_struct_hash(&self, timestamp: &str, nonce: u64) -> [u8; 32] {
        // Type hash for ClobAuth - note timestamp is STRING not uint256
        let type_hash = keccak256(
            "ClobAuth(address address,string timestamp,uint256 nonce,string message)"
        );

        // Hash the string fields
        let timestamp_hash = keccak256(timestamp);
        let message_hash = keccak256(AUTH_MESSAGE);

        // Encode the struct: typeHash + address + timestampHash + nonce + messageHash
        let mut encoded = Vec::with_capacity(160);
        encoded.extend_from_slice(type_hash.as_slice());
        // Address is padded to 32 bytes (left-padded with zeros)
        encoded.extend_from_slice(&[0u8; 12]);
        encoded.extend_from_slice(self.address.as_slice());
        encoded.extend_from_slice(timestamp_hash.as_slice());
        encoded.extend_from_slice(&U256::from(nonce).to_be_bytes::<32>());
        encoded.extend_from_slice(message_hash.as_slice());

        *keccak256(&encoded)
    }

    /// Build EIP-712 domain separator
    fn build_domain_separator(&self) -> [u8; 32] {
        let domain_type_hash = keccak256(
            "EIP712Domain(string name,string version,uint256 chainId)"
        );

        let name_hash = keccak256(POLYMARKET_DOMAIN_NAME);
        let version_hash = keccak256(POLYMARKET_DOMAIN_VERSION);

        let mut encoded = Vec::with_capacity(128);
        encoded.extend_from_slice(domain_type_hash.as_slice());
        encoded.extend_from_slice(name_hash.as_slice());
        encoded.extend_from_slice(version_hash.as_slice());
        encoded.extend_from_slice(&U256::from(POLYMARKET_CHAIN_ID).to_be_bytes::<32>());

        *keccak256(&encoded)
    }

    /// Sign authentication message for API key derivation
    /// Returns L1 headers needed for the API request
    pub async fn create_l1_headers(&self, nonce: u64) -> Result<L1Headers, ApiError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let timestamp_str = timestamp.to_string();

        // Build EIP-712 hash
        let domain_separator = self.build_domain_separator();
        let struct_hash = self.build_struct_hash(&timestamp_str, nonce);

        // Final message: \x19\x01 + domainSeparator + structHash
        let mut message = Vec::with_capacity(66);
        message.extend_from_slice(&[0x19, 0x01]);
        message.extend_from_slice(&domain_separator);
        message.extend_from_slice(&struct_hash);

        let digest = keccak256(&message);

        tracing::debug!("EIP-712 digest: 0x{}", hex::encode(digest));

        // Sign the hash
        let signature = self.signer
            .sign_hash(&digest)
            .await
            .map_err(|e| ApiError::Signing(format!("Failed to sign: {}", e)))?;

        // Get signature components - alloy uses recovery id 0/1, but Polymarket expects 27/28
        let mut sig_bytes = signature.as_bytes().to_vec();
        // The last byte is the recovery id - convert from 0/1 to 27/28 if needed
        if sig_bytes[64] < 27 {
            sig_bytes[64] += 27;
        }

        let sig_hex = format!("0x{}", hex::encode(&sig_bytes));
        tracing::debug!("Signature: {}", sig_hex);

        Ok(L1Headers {
            address: self.address_string(),
            timestamp: timestamp_str,
            nonce,
            signature: sig_hex,
        })
    }
}

/// L1 authentication headers for Polymarket API
#[derive(Debug, Clone)]
pub struct L1Headers {
    pub address: String,
    pub timestamp: String,
    pub nonce: u64,
    pub signature: String,
}

impl L1Headers {
    /// Apply L1 headers to a request
    pub fn apply_to_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request
            .header("POLY_ADDRESS", &self.address)
            .header("POLY_SIGNATURE", &self.signature)
            .header("POLY_TIMESTAMP", &self.timestamp)
            .header("POLY_NONCE", self.nonce.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signer_creation() {
        // Test with a known test private key (DO NOT use in production)
        let test_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let signer = PolymarketSigner::from_private_key(test_key);
        assert!(signer.is_ok());
    }

    #[tokio::test]
    async fn test_l1_headers_generation() {
        let test_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let signer = PolymarketSigner::from_private_key(test_key).unwrap();

        let headers = signer.create_l1_headers(0).await;
        assert!(headers.is_ok());

        let headers = headers.unwrap();
        assert!(headers.signature.starts_with("0x"));
        assert_eq!(headers.signature.len(), 132); // 0x + 65 bytes hex
    }
}
