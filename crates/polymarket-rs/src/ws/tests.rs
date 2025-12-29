// AIDEV-NOTE: Tests for WebSocket module - message parsing, mock emitter, manager

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crate::types::{
        ClobTrade, ConnectionState, ConnectionStatus, OrderBookLevel, OrderBookSnapshot, PriceUpdate,
    };
    use crate::ws::events::{EventEmitter, RtdsTrade};
    use crate::ws::manager::WebSocketManager;

    // ==================== Mock EventEmitter ====================

    /// Mock emitter for testing WebSocket event emissions
    #[derive(Default)]
    struct MockEmitter {
        price_update_count: AtomicUsize,
        orderbook_count: AtomicUsize,
        trade_count: AtomicUsize,
        rtds_trade_count: AtomicUsize,
        connection_count: AtomicUsize,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self::default()
        }

        fn price_updates(&self) -> usize {
            self.price_update_count.load(Ordering::SeqCst)
        }

        fn orderbook_updates(&self) -> usize {
            self.orderbook_count.load(Ordering::SeqCst)
        }

        fn trades(&self) -> usize {
            self.trade_count.load(Ordering::SeqCst)
        }
    }

    impl EventEmitter for MockEmitter {
        fn emit_price_update(&self, _update: &PriceUpdate) {
            self.price_update_count.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_orderbook_snapshot(&self, _snapshot: &OrderBookSnapshot) {
            self.orderbook_count.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_trade(&self, _trade: &ClobTrade) {
            self.trade_count.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_trade_update(&self, _trade: &RtdsTrade) {
            self.rtds_trade_count.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_connection_status(&self, _status: &ConnectionStatus) {
            self.connection_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    // ==================== Type Parsing Tests ====================

    #[test]
    fn test_price_update_deserialization() {
        let json = r#"{
            "market": "0xmarket",
            "asset_id": "token123",
            "price": 0.65
        }"#;

        let update: PriceUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.market, "0xmarket");
        assert_eq!(update.asset_id, "token123");
        assert!((update.price - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_price_update_with_timestamp() {
        let json = r#"{
            "market": "0xmarket",
            "asset_id": "token123",
            "price": 0.65,
            "timestamp": 1704067200
        }"#;

        let update: PriceUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.timestamp, Some(1704067200));
    }

    #[test]
    fn test_orderbook_snapshot_deserialization() {
        let json = r#"{
            "asset_id": "token456",
            "bids": [
                {"price": "0.60", "size": "100"},
                {"price": "0.55", "size": "200"}
            ],
            "asks": [
                {"price": "0.65", "size": "150"},
                {"price": "0.70", "size": "250"}
            ]
        }"#;

        let snapshot: OrderBookSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.asset_id, "token456");
        assert_eq!(snapshot.bids.len(), 2);
        assert_eq!(snapshot.asks.len(), 2);
        assert_eq!(snapshot.bids[0].price, "0.60");
        assert_eq!(snapshot.asks[0].size, "150");
    }

    #[test]
    fn test_orderbook_level_deserialization() {
        let json = r#"{"price": "0.75", "size": "500.25"}"#;
        let level: OrderBookLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level.price, "0.75");
        assert_eq!(level.size, "500.25");
    }

    #[test]
    fn test_clob_trade_deserialization() {
        let json = r#"{
            "asset_id": "token789",
            "price": "0.70",
            "size": "100.5",
            "side": "BUY"
        }"#;

        let trade: ClobTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.asset_id, "token789");
        assert_eq!(trade.price, "0.70");
        assert_eq!(trade.side, "BUY");
    }

    #[test]
    fn test_rtds_trade_deserialization() {
        let json = r#"{
            "type": "trade",
            "market": "0xmarket",
            "price": 0.65,
            "size": 100.0,
            "side": "buy",
            "timestamp": 1704067200
        }"#;

        let trade: RtdsTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.market, "0xmarket");
        assert!((trade.price - 0.65).abs() < 0.001);
        assert_eq!(trade.side, "buy");
    }

    #[test]
    fn test_connection_status_deserialization() {
        let json = r#"{
            "clob": "connected",
            "rtds": "disconnected"
        }"#;

        let status: ConnectionStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.clob, ConnectionState::Connected);
        assert_eq!(status.rtds, ConnectionState::Disconnected);
    }

    // ==================== Mock Emitter Tests ====================

    #[test]
    fn test_mock_emitter_counts_events() {
        let emitter = MockEmitter::new();

        // Initially all counts are zero
        assert_eq!(emitter.price_updates(), 0);
        assert_eq!(emitter.orderbook_updates(), 0);
        assert_eq!(emitter.trades(), 0);

        // Emit some events
        let price = PriceUpdate {
            market: "0xmarket".to_string(),
            asset_id: "test".to_string(),
            price: 0.5,
            timestamp: Some(1000),
        };
        emitter.emit_price_update(&price);
        emitter.emit_price_update(&price);

        let orderbook = OrderBookSnapshot {
            event_type: None,
            asset_id: "test".to_string(),
            market: None,
            hash: None,
            timestamp: Some(1000),
            bids: vec![],
            asks: vec![],
            last_trade_price: None,
        };
        emitter.emit_orderbook_snapshot(&orderbook);

        // Verify counts
        assert_eq!(emitter.price_updates(), 2);
        assert_eq!(emitter.orderbook_updates(), 1);
        assert_eq!(emitter.trades(), 0);
    }

    // ==================== WebSocket Manager Tests ====================

    #[test]
    fn test_manager_creation() {
        let emitter = Arc::new(MockEmitter::new());
        let manager = WebSocketManager::new(emitter);

        // Initially disconnected
        assert_eq!(manager.rtds_state(), ConnectionState::Disconnected);
        assert_eq!(manager.clob_state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_manager_emitter_access() {
        let emitter = Arc::new(MockEmitter::new());
        let manager = WebSocketManager::new(emitter.clone());

        let update = PriceUpdate {
            market: "0xmarket".to_string(),
            asset_id: "test".to_string(),
            price: 0.65,
            timestamp: Some(1000),
        };

        // Access emitter through manager and emit
        manager.emitter().emit_price_update(&update);
        assert_eq!(emitter.price_updates(), 1);
    }

    #[test]
    fn test_manager_state_transitions() {
        let emitter = Arc::new(MockEmitter::new());
        let manager = WebSocketManager::new(emitter);

        // Start disconnected
        assert_eq!(manager.rtds_state(), ConnectionState::Disconnected);
        assert_eq!(manager.clob_state(), ConnectionState::Disconnected);

        // Transition to connecting
        manager.set_rtds_state(ConnectionState::Connecting);
        assert_eq!(manager.rtds_state(), ConnectionState::Connecting);

        // Transition to connected
        manager.set_rtds_state(ConnectionState::Connected);
        assert_eq!(manager.rtds_state(), ConnectionState::Connected);
    }

    #[test]
    fn test_manager_reconnect_counter() {
        let emitter = Arc::new(MockEmitter::new());
        let manager = WebSocketManager::new(emitter);

        // Increment reconnect counter
        let count1 = manager.increment_rtds_reconnect();
        let count2 = manager.increment_rtds_reconnect();
        let count3 = manager.increment_clob_reconnect();

        assert_eq!(count1, 1);
        assert_eq!(count2, 2);
        assert_eq!(count3, 1); // Separate counter for clob
    }

    // ==================== Connection State Tests ====================

    #[test]
    fn test_connection_state_enum() {
        // Test that all variants are distinct
        assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connecting, ConnectionState::Connected);
        assert_ne!(ConnectionState::Reconnecting, ConnectionState::Connected);
    }

    #[test]
    fn test_connection_state_serialization() {
        // Note: ConnectionState uses lowercase serialization
        assert_eq!(
            serde_json::to_string(&ConnectionState::Disconnected).unwrap(),
            "\"disconnected\""
        );
        assert_eq!(
            serde_json::to_string(&ConnectionState::Connecting).unwrap(),
            "\"connecting\""
        );
        assert_eq!(
            serde_json::to_string(&ConnectionState::Connected).unwrap(),
            "\"connected\""
        );
    }

    // ==================== Reconnect Config Tests ====================

    #[test]
    fn test_reconnect_delay_calculation() {
        use crate::ws::manager::ReconnectConfig;
        use std::time::Duration;

        let config = ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_attempts: Some(10),
        };

        // First attempt: 1s
        let delay1 = WebSocketManager::<MockEmitter>::calculate_reconnect_delay(1, &config);
        assert_eq!(delay1, Duration::from_secs(1));

        // Second attempt: 2s (exponential backoff)
        let delay2 = WebSocketManager::<MockEmitter>::calculate_reconnect_delay(2, &config);
        assert_eq!(delay2, Duration::from_secs(2));

        // Third attempt: 4s
        let delay3 = WebSocketManager::<MockEmitter>::calculate_reconnect_delay(3, &config);
        assert_eq!(delay3, Duration::from_secs(4));

        // Should cap at max_delay
        let delay_max = WebSocketManager::<MockEmitter>::calculate_reconnect_delay(10, &config);
        assert_eq!(delay_max, Duration::from_secs(60));
    }
}
