// AIDEV-NOTE: TauriEventEmitter - implements polymarket_rs::EventEmitter for Tauri
use polymarket_rs::{
    ws::RtdsTrade, ClobTrade, ConnectionStatus, EventEmitter, OrderBookSnapshot, PriceUpdate,
};
use tauri::{AppHandle, Emitter};
use tracing::error;

/// Tauri implementation of EventEmitter
/// Bridges WebSocket events to Tauri frontend
pub struct TauriEventEmitter(pub AppHandle);

impl EventEmitter for TauriEventEmitter {
    fn emit_price_update(&self, update: &PriceUpdate) {
        if let Err(e) = self.0.emit("price_update", update) {
            error!("Failed to emit price_update: {}", e);
        }
    }

    fn emit_orderbook_snapshot(&self, snapshot: &OrderBookSnapshot) {
        if let Err(e) = self.0.emit("orderbook_snapshot", snapshot) {
            error!("Failed to emit orderbook_snapshot: {}", e);
        }
    }

    fn emit_trade(&self, trade: &ClobTrade) {
        if let Err(e) = self.0.emit("clob_trade", trade) {
            error!("Failed to emit clob_trade: {}", e);
        }
    }

    fn emit_trade_update(&self, trade: &RtdsTrade) {
        if let Err(e) = self.0.emit("trade_update", trade) {
            error!("Failed to emit trade_update: {}", e);
        }
    }

    fn emit_connection_status(&self, status: &ConnectionStatus) {
        if let Err(e) = self.0.emit("connection_status", status) {
            error!("Failed to emit connection_status: {}", e);
        }
    }
}
