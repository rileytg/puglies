// AIDEV-NOTE: CLOB WebSocket client for order book depth data
// Connects to wss://ws-subscriptions-clob.polymarket.com

use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

use crate::types::{ClobTrade, ConnectionState, OrderBookLevel, OrderBookSnapshot, PriceUpdate};
use super::events::EventEmitter;
use super::manager::{ReconnectConfig, WebSocketManager};

const CLOB_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// CLOB WebSocket client for order book data
pub struct ClobWebSocket<E: EventEmitter> {
    manager: Arc<WebSocketManager<E>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl<E: EventEmitter> ClobWebSocket<E> {
    pub fn new(manager: Arc<WebSocketManager<E>>) -> Self {
        Self {
            manager,
            shutdown_tx: None,
        }
    }

    /// Start the CLOB WebSocket connection for specific token IDs
    pub async fn connect(&mut self, token_ids: Vec<String>) {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let manager = self.manager.clone();
        let token_ids = token_ids.clone();

        tokio::spawn(async move {
            let config = ReconnectConfig::default();

            loop {
                manager.set_clob_state(ConnectionState::Connecting);

                match Self::connect_and_run(&manager, &token_ids, &mut shutdown_rx).await {
                    Ok(()) => {
                        info!("CLOB connection closed gracefully");
                        break;
                    }
                    Err(e) => {
                        error!("CLOB connection error: {}", e);

                        let attempts = manager.increment_clob_reconnect();

                        if let Some(max) = config.max_attempts {
                            if attempts >= max {
                                manager.set_clob_state(ConnectionState::Failed);
                                error!("CLOB max reconnect attempts ({}) reached", max);
                                break;
                            }
                        }

                        manager.set_clob_state(ConnectionState::Reconnecting);
                        let delay = WebSocketManager::<E>::calculate_reconnect_delay(attempts, &config);
                        info!("CLOB reconnecting in {:?} (attempt {})", delay, attempts);

                        tokio::select! {
                            _ = tokio::time::sleep(delay) => continue,
                            _ = shutdown_rx.recv() => {
                                info!("CLOB shutdown during reconnect delay");
                                break;
                            }
                        }
                    }
                }
            }

            manager.set_clob_state(ConnectionState::Disconnected);
        });
    }

    async fn connect_and_run(
        manager: &Arc<WebSocketManager<E>>,
        token_ids: &[String],
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Connecting to CLOB WS: {}", CLOB_WS_URL);

        let (ws_stream, _) = connect_async(CLOB_WS_URL).await?;
        let (mut write, mut read) = ws_stream.split();

        manager.set_clob_state(ConnectionState::Connected);
        info!("CLOB WebSocket connected successfully");

        // Subscribe to order books for each token
        for token_id in token_ids {
            let subscribe_msg = ClobSubscribe {
                auth: None,
                markets: vec![],
                assets_ids: vec![token_id.clone()],
                msg_type: "subscribe".to_string(),
            };

            let msg = serde_json::to_string(&subscribe_msg)?;
            write.send(Message::Text(msg)).await?;
            debug!("Subscribed to order book: {}", token_id);
        }

        // Handle incoming messages
        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            manager.record_clob_message();
                            Self::handle_message(manager.emitter(), &text);
                        }
                        Some(Ok(Message::Ping(data))) => {
                            write.send(Message::Pong(data)).await?;
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("CLOB server closed connection");
                            return Ok(());
                        }
                        Some(Err(e)) => {
                            return Err(Box::new(e));
                        }
                        None => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("CLOB shutdown requested");
                    let _ = write.send(Message::Close(None)).await;
                    return Ok(());
                }
            }
        }
    }

    fn handle_message(emitter: &Arc<E>, text: &str) {
        // AIDEV-NOTE: Log first message to debug format issues
        let preview = if text.len() > 200 { &text[..200] } else { text };
        debug!("CLOB raw message ({}): {}", text.len(), preview);

        // Try to parse as generic JSON to check event_type
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            debug!("Failed to parse CLOB message as JSON: {}", preview);
            return;
        };

        // Handle array of messages (initial snapshots)
        // AIDEV-NOTE: Initial order book snapshots come as array without event_type field
        if let Some(arr) = value.as_array() {
            for item in arr {
                // Check if it has order book fields (bids, asks, asset_id)
                if item.get("bids").is_some() && item.get("asks").is_some() {
                    if let Ok(raw) = serde_json::from_value::<RawOrderBookSnapshot>(item.clone()) {
                        let snapshot = Self::convert_snapshot(raw);
                        debug!("Order book snapshot for {} ({} bids, {} asks)",
                               snapshot.asset_id, snapshot.bids.len(), snapshot.asks.len());
                        emitter.emit_orderbook_snapshot(&snapshot);
                    } else {
                        debug!("Failed to parse order book from array item: {:?}", item);
                    }
                } else if let Some(event_type) = item.get("event_type").and_then(|v| v.as_str()) {
                    // Handle typed events within arrays
                    if event_type == "book" {
                        if let Ok(raw) = serde_json::from_value::<RawOrderBookSnapshot>(item.clone()) {
                            let snapshot = Self::convert_snapshot(raw);
                            debug!("Order book snapshot for {}", snapshot.asset_id);
                            emitter.emit_orderbook_snapshot(&snapshot);
                        }
                    }
                }
            }
            return;
        }

        // Handle single message
        let event_type = value.get("event_type").and_then(|v| v.as_str());

        match event_type {
            Some("book") => {
                if let Ok(raw) = serde_json::from_value::<RawOrderBookSnapshot>(value) {
                    let snapshot = Self::convert_snapshot(raw);
                    debug!("Order book snapshot for {}", snapshot.asset_id);
                    emitter.emit_orderbook_snapshot(&snapshot);
                }
            }
            Some("price_change") => {
                // AIDEV-NOTE: price_change has price_changes array with best_bid/best_ask
                if let Ok(price_event) = serde_json::from_value::<ClobPriceChangeEvent>(value) {
                    for change in &price_event.price_changes {
                        // Emit price update using best_bid as the price
                        if let Ok(price) = change.best_bid.parse::<f64>() {
                            let update = PriceUpdate {
                                market: price_event.market.clone(),
                                asset_id: change.asset_id.clone(),
                                price,
                                timestamp: price_event.timestamp,
                            };
                            debug!("Price update: {} -> {}", change.asset_id, price);
                            emitter.emit_price_update(&update);
                        }
                    }
                }
            }
            Some("trade") => {
                if let Ok(trade) = serde_json::from_value::<ClobTrade>(value) {
                    debug!("CLOB trade: {:?}", trade);
                    emitter.emit_trade(&trade);
                }
            }
            _ => {
                let preview = if text.len() > 100 { &text[..100] } else { text };
                debug!("Unknown CLOB message: {}", preview);
            }
        }
    }

    /// Convert raw snapshot (with String timestamp) to our OrderBookSnapshot
    fn convert_snapshot(raw: RawOrderBookSnapshot) -> OrderBookSnapshot {
        OrderBookSnapshot {
            event_type: raw.event_type,
            asset_id: raw.asset_id,
            market: raw.market,
            hash: raw.hash,
            timestamp: raw.timestamp,
            bids: raw.bids,
            asks: raw.asks,
            last_trade_price: raw.last_trade_price,
        }
    }

    /// Disconnect from CLOB WebSocket
    pub fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

// CLOB Message Types

#[derive(Debug, Serialize)]
struct ClobSubscribe {
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<String>,
    markets: Vec<String>,
    assets_ids: Vec<String>,
    #[serde(rename = "type")]
    msg_type: String,
}

/// Raw order book snapshot from CLOB (with String timestamp)
/// AIDEV-NOTE: timestamp comes as String from API, last_trade_price is optional
#[derive(Debug, Clone, Deserialize)]
struct RawOrderBookSnapshot {
    #[serde(rename = "event_type")]
    event_type: Option<String>,
    asset_id: String,
    market: Option<String>,
    hash: Option<String>,
    #[serde(default, deserialize_with = "deserialize_timestamp")]
    timestamp: Option<i64>,
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
    #[serde(default)]
    last_trade_price: Option<String>,
}

/// Deserialize timestamp from either String or i64
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::String(s)) => {
            s.parse::<i64>().map(Some).map_err(D::Error::custom)
        }
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

/// Price change event from CLOB (contains array of price changes)
#[derive(Debug, Clone, Deserialize)]
struct ClobPriceChangeEvent {
    #[serde(rename = "event_type")]
    #[allow(dead_code)]
    event_type: Option<String>,
    market: String,
    price_changes: Vec<ClobPriceChange>,
    timestamp: Option<i64>,
}

/// Individual price change within a price_change event
#[derive(Debug, Clone, Deserialize)]
struct ClobPriceChange {
    asset_id: String,
    #[allow(dead_code)]
    price: String,
    #[allow(dead_code)]
    size: String,
    #[allow(dead_code)]
    side: String,
    best_bid: String,
    #[allow(dead_code)]
    best_ask: String,
    #[allow(dead_code)]
    hash: Option<String>,
}
