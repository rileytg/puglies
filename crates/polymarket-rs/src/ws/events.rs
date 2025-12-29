// AIDEV-NOTE: EventEmitter trait - abstracts event emission for WebSocket clients
// Allows polymarket-rs to be used without Tauri dependency

use crate::types::{ConnectionStatus, ClobTrade, OrderBookSnapshot, PriceUpdate};

/// Trait for emitting WebSocket events
/// Implement this trait to receive events from WebSocket clients
pub trait EventEmitter: Send + Sync + 'static {
    /// Emit a price update event
    fn emit_price_update(&self, update: &PriceUpdate);

    /// Emit an order book snapshot
    fn emit_orderbook_snapshot(&self, snapshot: &OrderBookSnapshot);

    /// Emit a CLOB trade event
    fn emit_trade(&self, trade: &ClobTrade);

    /// Emit a trade update from RTDS
    fn emit_trade_update(&self, trade: &RtdsTrade);

    /// Emit connection status update
    fn emit_connection_status(&self, status: &ConnectionStatus);
}

/// Trade from RTDS (different format than ClobTrade)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RtdsTrade {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub market: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
    pub timestamp: Option<i64>,
}

/// No-op event emitter for testing or headless operation
pub struct NoOpEmitter;

impl EventEmitter for NoOpEmitter {
    fn emit_price_update(&self, _update: &PriceUpdate) {}
    fn emit_orderbook_snapshot(&self, _snapshot: &OrderBookSnapshot) {}
    fn emit_trade(&self, _trade: &ClobTrade) {}
    fn emit_trade_update(&self, _trade: &RtdsTrade) {}
    fn emit_connection_status(&self, _status: &ConnectionStatus) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Mock emitter that counts events for testing
    pub struct MockEmitter {
        pub price_updates: AtomicUsize,
        pub orderbook_snapshots: AtomicUsize,
        pub trades: AtomicUsize,
        pub connection_updates: AtomicUsize,
    }

    impl MockEmitter {
        pub fn new() -> Self {
            Self {
                price_updates: AtomicUsize::new(0),
                orderbook_snapshots: AtomicUsize::new(0),
                trades: AtomicUsize::new(0),
                connection_updates: AtomicUsize::new(0),
            }
        }
    }

    impl EventEmitter for MockEmitter {
        fn emit_price_update(&self, _update: &PriceUpdate) {
            self.price_updates.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_orderbook_snapshot(&self, _snapshot: &OrderBookSnapshot) {
            self.orderbook_snapshots.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_trade(&self, _trade: &ClobTrade) {
            self.trades.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_trade_update(&self, _trade: &RtdsTrade) {
            self.trades.fetch_add(1, Ordering::SeqCst);
        }

        fn emit_connection_status(&self, _status: &ConnectionStatus) {
            self.connection_updates.fetch_add(1, Ordering::SeqCst);
        }
    }

    // Allow Arc<MockEmitter> to be used as EventEmitter
    impl EventEmitter for Arc<MockEmitter> {
        fn emit_price_update(&self, update: &PriceUpdate) {
            (**self).emit_price_update(update);
        }
        fn emit_orderbook_snapshot(&self, snapshot: &OrderBookSnapshot) {
            (**self).emit_orderbook_snapshot(snapshot);
        }
        fn emit_trade(&self, trade: &ClobTrade) {
            (**self).emit_trade(trade);
        }
        fn emit_trade_update(&self, trade: &RtdsTrade) {
            (**self).emit_trade_update(trade);
        }
        fn emit_connection_status(&self, status: &ConnectionStatus) {
            (**self).emit_connection_status(status);
        }
    }

    #[test]
    fn test_mock_emitter() {
        let emitter = MockEmitter::new();

        let update = PriceUpdate {
            market: "test".to_string(),
            asset_id: "123".to_string(),
            price: 0.5,
            timestamp: None,
        };

        emitter.emit_price_update(&update);
        assert_eq!(emitter.price_updates.load(Ordering::SeqCst), 1);
    }
}
