// AIDEV-NOTE: API credentials for Polymarket authentication

use serde::{Deserialize, Serialize};

/// API credentials returned from Polymarket auth endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCredentials {
    /// API key for authenticated requests
    pub api_key: String,
    /// API secret for HMAC signing
    pub api_secret: String,
    /// API passphrase
    pub api_passphrase: String,
    /// Wallet address that owns these credentials
    pub address: String,
}

/// Current authentication status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    /// Whether the user is authenticated
    pub is_authenticated: bool,
    /// Connected wallet address (if authenticated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

impl Default for AuthStatus {
    fn default() -> Self {
        Self {
            is_authenticated: false,
            address: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_serialization() {
        let creds = ApiCredentials {
            api_key: "test-key".to_string(),
            api_secret: "test-secret".to_string(),
            api_passphrase: "test-pass".to_string(),
            address: "0x1234".to_string(),
        };

        let json = serde_json::to_string(&creds).unwrap();
        assert!(json.contains("\"apiKey\":\"test-key\""));
    }

    #[test]
    fn test_auth_status_default() {
        let status = AuthStatus::default();
        assert!(!status.is_authenticated);
        assert!(status.address.is_none());
    }
}
