// AIDEV-NOTE: HMAC authentication for Polymarket CLOB REST API
// Generates L2 authentication headers for authenticated requests
// AIDEV-NOTE: API secret uses URL-safe base64 encoding (_- instead of +/)

use base64::{engine::general_purpose::{STANDARD as BASE64, URL_SAFE_NO_PAD}, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::auth::ApiCredentials;
use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

/// HMAC authentication helper for CLOB API requests
#[derive(Clone)]
pub struct HmacAuth {
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    address: String,
}

impl HmacAuth {
    /// Create a new HMAC auth helper from credentials
    pub fn new(credentials: &ApiCredentials) -> Self {
        Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            api_passphrase: credentials.api_passphrase.clone(),
            address: credentials.address.clone(),
        }
    }

    /// Generate authentication headers for a request
    ///
    /// Returns a tuple of headers: (api_key, signature, timestamp, passphrase)
    pub fn generate_headers(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<AuthHeaders, AppError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let signature = self.sign(&timestamp, method, path, body)?;

        tracing::debug!(
            "HMAC headers: api_key={}, timestamp={}, method={}, path={}, sig_len={}, address={}",
            &self.api_key[..8.min(self.api_key.len())],
            timestamp,
            method,
            path,
            signature.len(),
            &self.address
        );

        Ok(AuthHeaders {
            api_key: self.api_key.clone(),
            signature,
            timestamp,
            passphrase: self.api_passphrase.clone(),
            address: self.address.clone(),
        })
    }

    /// Create HMAC-SHA256 signature for the request
    fn sign(
        &self,
        timestamp: &str,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<String, AppError> {
        // Decode the base64-encoded secret - try multiple formats
        // Polymarket may use standard or URL-safe base64, with or without padding
        let secret_bytes = URL_SAFE_NO_PAD
            .decode(&self.api_secret)
            .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&self.api_secret))
            .or_else(|_| BASE64.decode(&self.api_secret))
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&self.api_secret))
            .map_err(|e| {
                tracing::error!("Failed to decode secret (len={}): {}", self.api_secret.len(), e);
                AppError::Internal(format!("Invalid API secret: {}", e))
            })?;

        tracing::debug!("Decoded secret: {} bytes", secret_bytes.len());

        // Create the message to sign: timestamp + method + path + body
        let body_str = body.unwrap_or("");
        let message = format!("{}{}{}{}", timestamp, method.to_uppercase(), path, body_str);

        tracing::debug!("HMAC message to sign: {}", message);

        // Create HMAC
        let mut mac = HmacSha256::new_from_slice(&secret_bytes)
            .map_err(|e| AppError::Internal(format!("HMAC error: {}", e)))?;

        mac.update(message.as_bytes());

        // Finalize and encode as URL-safe base64 WITH padding (matching TS/Python clients)
        // AIDEV-NOTE: Must use URL_SAFE (with padding), not URL_SAFE_NO_PAD
        let result = mac.finalize();
        let signature = base64::engine::general_purpose::URL_SAFE.encode(result.into_bytes());

        Ok(signature)
    }
}

/// Authentication headers for CLOB API requests (L2 auth)
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    /// POLY_API_KEY header
    pub api_key: String,
    /// POLY_SIGNATURE header
    pub signature: String,
    /// POLY_TIMESTAMP header
    pub timestamp: String,
    /// POLY_PASSPHRASE header
    pub passphrase: String,
    /// POLY_ADDRESS header
    pub address: String,
}

impl AuthHeaders {
    /// Apply headers to a reqwest RequestBuilder
    pub fn apply_to_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        request
            .header("POLY_ADDRESS", &self.address)
            .header("POLY_API_KEY", &self.api_key)
            .header("POLY_SIGNATURE", &self.signature)
            .header("POLY_TIMESTAMP", &self.timestamp)
            .header("POLY_PASSPHRASE", &self.passphrase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature() {
        let credentials = ApiCredentials {
            api_key: "test-key".to_string(),
            // Base64 encoded "test-secret"
            api_secret: "dGVzdC1zZWNyZXQ=".to_string(),
            api_passphrase: "test-pass".to_string(),
            address: "0x1234".to_string(),
        };

        let auth = HmacAuth::new(&credentials);
        let headers = auth.generate_headers("GET", "/orders", None);

        assert!(headers.is_ok());
        let headers = headers.unwrap();
        assert_eq!(headers.api_key, "test-key");
        assert!(!headers.signature.is_empty());
    }
}
