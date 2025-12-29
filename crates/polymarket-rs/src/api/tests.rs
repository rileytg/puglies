// AIDEV-NOTE: Tests for API module - deserialization and client creation

#[cfg(test)]
mod tests {
    use crate::api::order::{OrderParams, OrderSide, OrderType, SignatureType, UnsignedOrder};
    use crate::api::{ClobClient, GammaClient};
    use crate::types::{Balance, Market, Order, Position, RawMarket};

    // ==================== Type Deserialization Tests ====================

    #[test]
    fn test_raw_market_deserialization() {
        // Test that raw markets can deserialize from API response
        let json = r#"{
            "id": "0x123",
            "question": "Will it rain tomorrow?",
            "conditionId": "0xabc",
            "slug": "rain-tomorrow",
            "active": true,
            "closed": false,
            "outcomes": "[\"Yes\",\"No\"]",
            "outcomePrices": "[\"0.65\",\"0.35\"]",
            "clobTokenIds": "[\"token1\",\"token2\"]"
        }"#;

        let raw: RawMarket = serde_json::from_str(json).unwrap();
        assert_eq!(raw.id, "0x123");
        assert_eq!(raw.question, "Will it rain tomorrow?");
        assert!(raw.active);
        assert!(!raw.closed);
    }

    #[test]
    fn test_market_from_raw() {
        let json = r#"{
            "id": "0x456",
            "question": "Test market?",
            "conditionId": "0xdef",
            "slug": "test-market",
            "active": true,
            "closed": false,
            "outcomes": "[\"Yes\",\"No\"]",
            "outcomePrices": "[\"0.7\",\"0.3\"]",
            "clobTokenIds": "[\"t1\",\"t2\"]"
        }"#;

        let raw: RawMarket = serde_json::from_str(json).unwrap();
        let market: Market = raw.into();

        assert_eq!(market.id, "0x456");
        assert_eq!(market.tokens.len(), 2);
        assert_eq!(market.tokens[0].outcome, "Yes");
        assert!((market.tokens[0].price - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_balance_deserialization() {
        let json = r#"{"balance": "1234.56"}"#;
        let balance: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.balance, "1234.56");
    }

    #[test]
    fn test_balance_with_allowances() {
        let json = r#"{
            "balance": "1000.00",
            "allowances": {"0xexchange": "1000000"}
        }"#;

        let balance: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.balance, "1000.00");
        assert!(balance.allowances.contains_key("0xexchange"));
    }

    #[test]
    fn test_position_deserialization() {
        let json = r#"{
            "asset": "0x123",
            "conditionId": "0xabc",
            "size": 100.5,
            "avgPrice": 0.65,
            "initialValue": 65.325,
            "currentValue": 70.35,
            "cashPnl": 5.025,
            "percentPnl": 7.69,
            "curPrice": 0.70
        }"#;

        let position: Position = serde_json::from_str(json).unwrap();
        assert_eq!(position.asset, "0x123");
        assert!((position.size - 100.5).abs() < 0.001);
        assert!((position.avg_price - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_order_deserialization() {
        let json = r#"{
            "id": "order-123",
            "market": "0xmarket",
            "asset_id": "token-123",
            "side": "BUY",
            "originalSize": "100",
            "sizeMatched": "50",
            "price": "0.65",
            "status": "LIVE",
            "createdAt": "2024-01-01T00:00:00Z"
        }"#;

        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.id, "order-123");
        assert_eq!(order.side, "BUY");
        assert_eq!(order.status, "LIVE");
        assert_eq!(order.price, "0.65");
    }

    // ==================== Order Types Tests ====================

    #[test]
    fn test_order_params_creation() {
        let params = OrderParams {
            token_id: "123456".to_string(),
            price: 0.65,
            size: 100.0,
            side: OrderSide::Buy,
            order_type: OrderType::Gtc,
            expiration_secs: Some(86400),
        };

        assert_eq!(params.token_id, "123456");
        assert!((params.price - 0.65).abs() < 0.001);
        assert_eq!(params.side, OrderSide::Buy);
    }

    #[test]
    fn test_order_side_serialization() {
        let buy = OrderSide::Buy;
        let sell = OrderSide::Sell;

        assert_eq!(serde_json::to_string(&buy).unwrap(), "\"BUY\"");
        assert_eq!(serde_json::to_string(&sell).unwrap(), "\"SELL\"");
    }

    #[test]
    fn test_order_type_serialization() {
        let gtc = OrderType::Gtc;
        let fok = OrderType::Fok;

        assert_eq!(serde_json::to_string(&gtc).unwrap(), "\"GTC\"");
        assert_eq!(serde_json::to_string(&fok).unwrap(), "\"FOK\"");
    }

    #[test]
    fn test_signature_type_serialization() {
        let eoa = SignatureType::Eoa;
        let proxy = SignatureType::Proxy;

        // SignatureType serializes as variant names
        assert_eq!(serde_json::to_string(&eoa).unwrap(), "\"Eoa\"");
        assert_eq!(serde_json::to_string(&proxy).unwrap(), "\"Proxy\"");
    }

    #[test]
    fn test_unsigned_order_structure() {
        let order = UnsignedOrder {
            salt: "12345".to_string(),
            maker: "0xmaker".to_string(),
            signer: "0xsigner".to_string(),
            taker: "0x0000000000000000000000000000000000000000".to_string(),
            token_id: "token123".to_string(),
            maker_amount: "1000000".to_string(),
            taker_amount: "650000".to_string(),
            expiration: "1735689600".to_string(),
            nonce: "1".to_string(),
            fee_rate_bps: "0".to_string(),
            side: OrderSide::Buy,
            signature_type: SignatureType::Proxy,
        };

        assert_eq!(order.salt, "12345");
        assert_eq!(order.maker_amount, "1000000");
    }

    // ==================== Client Creation Tests ====================

    #[test]
    fn test_gamma_client_creation() {
        let client = GammaClient::new();
        // Client should be creatable without errors
        assert!(std::mem::size_of_val(&client) > 0);
    }

    #[test]
    fn test_clob_client_creation() {
        let client = ClobClient::new();
        // Client should be creatable without authentication
        assert!(std::mem::size_of_val(&client) > 0);
    }

    #[test]
    fn test_clob_client_with_credentials() {
        use crate::auth::ApiCredentials;

        let creds = ApiCredentials {
            api_key: "test_key".to_string(),
            api_secret: "dGVzdF9zZWNyZXQ=".to_string(), // base64
            api_passphrase: "test_pass".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        let mut client = ClobClient::new();
        client.set_credentials(&creds);

        // Client should accept credentials
        assert!(std::mem::size_of_val(&client) > 0);
    }

    // ==================== Price History Tests ====================

    #[test]
    fn test_price_point_deserialization() {
        use crate::types::PricePoint;

        let json = r#"{"t": 1704067200, "p": 0.65}"#;
        let point: PricePoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.t, 1704067200);
        assert!((point.p - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_price_history_response_deserialization() {
        use crate::types::PriceHistoryResponse;

        let json = r#"{
            "history": [
                {"t": 1704067200, "p": 0.65},
                {"t": 1704153600, "p": 0.70}
            ]
        }"#;

        let response: PriceHistoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.history.len(), 2);
        assert_eq!(response.history[0].t, 1704067200);
        assert!((response.history[0].p - 0.65).abs() < 0.001);
    }
}
