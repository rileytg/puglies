// AIDEV-NOTE: Tests for auth module - HMAC signatures, EIP-712 signing, credentials

#[cfg(test)]
mod tests {
    use crate::auth::{ApiCredentials, AuthStatus, HmacAuth, OrderSigner, PolymarketSigner};

    // ==================== Credentials Tests ====================

    #[test]
    fn test_api_credentials_creation() {
        let creds = ApiCredentials {
            api_key: "test_key".to_string(),
            api_secret: "dGVzdF9zZWNyZXQ=".to_string(), // base64 for "test_secret"
            api_passphrase: "test_pass".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        assert_eq!(creds.api_key, "test_key");
        assert_eq!(creds.address.len(), 42);
        assert!(creds.address.starts_with("0x"));
    }

    #[test]
    fn test_api_credentials_serialization() {
        let creds = ApiCredentials {
            api_key: "key".to_string(),
            api_secret: "secret".to_string(),
            api_passphrase: "pass".to_string(),
            address: "0xaddr".to_string(),
        };

        let json = serde_json::to_string(&creds).unwrap();
        assert!(json.contains("\"apiKey\":\"key\""));
        assert!(json.contains("\"apiSecret\":\"secret\""));
    }

    #[test]
    fn test_api_credentials_deserialization() {
        let json = r#"{
            "apiKey": "my_key",
            "apiSecret": "my_secret",
            "apiPassphrase": "my_pass",
            "address": "0x123"
        }"#;

        let creds: ApiCredentials = serde_json::from_str(json).unwrap();
        assert_eq!(creds.api_key, "my_key");
        assert_eq!(creds.api_secret, "my_secret");
    }

    #[test]
    fn test_auth_status_authenticated() {
        let status = AuthStatus {
            is_authenticated: true,
            address: Some("0x123".to_string()),
        };

        assert!(status.is_authenticated);
        assert_eq!(status.address.as_deref(), Some("0x123"));
    }

    #[test]
    fn test_auth_status_unauthenticated() {
        let status = AuthStatus {
            is_authenticated: false,
            address: None,
        };

        assert!(!status.is_authenticated);
        assert!(status.address.is_none());
    }

    // ==================== HMAC Auth Tests ====================

    #[test]
    fn test_hmac_auth_creation() {
        use base64::Engine;

        // Create valid credentials with base64-encoded secret
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode("test_secret");
        let creds = ApiCredentials {
            api_key: "api_key".to_string(),
            api_secret: secret_b64,
            api_passphrase: "passphrase".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        let auth = HmacAuth::new(&creds);
        assert!(std::mem::size_of_val(&auth) > 0);
    }

    #[test]
    fn test_hmac_generate_headers() {
        use base64::Engine;

        // Create credentials with known secret
        let secret = "my_test_secret_key";
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(secret);

        let creds = ApiCredentials {
            api_key: "test_key".to_string(),
            api_secret: secret_b64,
            api_passphrase: "test_pass".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        let auth = HmacAuth::new(&creds);

        // Generate headers for known inputs
        let headers = auth.generate_headers("GET", "/balance", None).unwrap();

        // Verify structure
        assert!(!headers.signature.is_empty());
        assert_eq!(headers.api_key, "test_key");
        assert_eq!(headers.passphrase, "test_pass");
        assert!(!headers.timestamp.is_empty());
    }

    // ==================== EIP-712 Signer Tests ====================

    // Known test private key (Anvil's first account - DO NOT USE IN PRODUCTION)
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const TEST_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    #[test]
    fn test_polymarket_signer_from_private_key() {
        let signer = PolymarketSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let address = signer.address_string();

        // Should derive correct address from private key
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
        // The actual address for this key
        assert_eq!(address.to_lowercase(), TEST_ADDRESS.to_lowercase());
    }

    #[test]
    fn test_polymarket_signer_without_0x_prefix() {
        let key_without_prefix = TEST_PRIVATE_KEY.strip_prefix("0x").unwrap();
        let signer = PolymarketSigner::from_private_key(key_without_prefix).unwrap();

        // Should work with or without 0x prefix
        assert_eq!(
            signer.address_string().to_lowercase(),
            TEST_ADDRESS.to_lowercase()
        );
    }

    #[test]
    fn test_polymarket_signer_invalid_key() {
        let result = PolymarketSigner::from_private_key("not_a_valid_key");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_polymarket_signer_l1_headers() {
        let signer = PolymarketSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let headers = signer.create_l1_headers(0).await.unwrap();

        // Verify L1 headers structure
        assert!(!headers.timestamp.is_empty());
        assert!(!headers.signature.is_empty());
        assert!(!headers.address.is_empty());
    }

    // ==================== Order Signer Tests ====================

    #[test]
    fn test_order_signer_creation() {
        let signer = OrderSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let address = signer.address_string();

        assert_eq!(address.to_lowercase(), TEST_ADDRESS.to_lowercase());
    }

    #[tokio::test]
    async fn test_order_signer_signs_order() {
        use crate::api::order::{OrderSide, SignatureType, UnsignedOrder};

        let signer = OrderSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();

        let unsigned_order = UnsignedOrder {
            salt: "12345".to_string(),
            maker: TEST_ADDRESS.to_string(),
            signer: TEST_ADDRESS.to_string(),
            taker: "0x0000000000000000000000000000000000000000".to_string(),
            token_id: "71321045679252212594626385532706912750332728571942532289631379312455583992563".to_string(),
            maker_amount: "1000000".to_string(),
            taker_amount: "650000".to_string(),
            expiration: "1735689600".to_string(),
            nonce: "0".to_string(),
            fee_rate_bps: "0".to_string(),
            side: OrderSide::Buy,
            signature_type: SignatureType::Eoa,
        };

        let signed = signer.sign_order(&unsigned_order).await.unwrap();

        // Signed order should have a signature
        assert!(!signed.signature.is_empty());
        assert!(signed.signature.starts_with("0x"));

        // Original fields preserved through nested order
        assert_eq!(signed.order.salt, unsigned_order.salt);
        assert_eq!(signed.order.maker_amount, unsigned_order.maker_amount);
    }
}
