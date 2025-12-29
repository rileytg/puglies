// AIDEV-NOTE: RTDS WebSocket client for real-time market activity (prices, trades)
// Connects to wss://ws-live-data.polymarket.com (no /ws suffix!)
// Subscription format: { action, subscriptions: [{ topic, type, filters }] }

use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

use super::manager::{ConnectionState, ReconnectConfig, WebSocketManager};

// AIDEV-NOTE: URL must NOT have /ws suffix - that returns 403
const RTDS_URL: &str = "wss://ws-live-data.polymarket.com";

/// RTDS WebSocket client for real-time market data
pub struct RtdsClient {
    manager: Arc<WebSocketManager>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl RtdsClient {
    pub fn new(manager: Arc<WebSocketManager>) -> Self {
        Self {
            manager,
            shutdown_tx: None,
        }
    }

    /// Start the RTDS WebSocket connection
    pub async fn connect(&mut self, markets: Vec<String>) {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let manager = self.manager.clone();
        let markets = markets.clone();

        tokio::spawn(async move {
            let config = ReconnectConfig::default();

            loop {
                manager.set_rtds_state(ConnectionState::Connecting);

                match Self::connect_and_run(&manager, &markets, &mut shutdown_rx).await {
                    Ok(()) => {
                        info!("RTDS connection closed gracefully");
                        break;
                    }
                    Err(e) => {
                        error!("RTDS connection error: {}", e);

                        let attempts = manager.increment_rtds_reconnect();

                        if let Some(max) = config.max_attempts {
                            if attempts >= max {
                                manager.set_rtds_state(ConnectionState::Failed);
                                error!("RTDS max reconnect attempts ({}) reached", max);
                                break;
                            }
                        }

                        manager.set_rtds_state(ConnectionState::Reconnecting);
                        let delay = WebSocketManager::calculate_reconnect_delay(attempts, &config);
                        info!("RTDS reconnecting in {:?} (attempt {})", delay, attempts);

                        tokio::select! {
                            _ = tokio::time::sleep(delay) => continue,
                            _ = shutdown_rx.recv() => {
                                info!("RTDS shutdown during reconnect delay");
                                break;
                            }
                        }
                    }
                }
            }

            manager.set_rtds_state(ConnectionState::Disconnected);
        });
    }

    async fn connect_and_run(
        manager: &Arc<WebSocketManager>,
        markets: &[String],
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Connecting to RTDS: {}", RTDS_URL);

        let (ws_stream, _) = connect_async(RTDS_URL).await?;
        let (mut write, mut read) = ws_stream.split();

        manager.set_rtds_state(ConnectionState::Connected);
        info!("RTDS connected successfully");

        // Subscribe to markets using token IDs
        // AIDEV-NOTE: filters is a JSON array string of token IDs
        if !markets.is_empty() {
            let filters = serde_json::to_string(&markets)?;
            let subscribe_msg = RtdsSubscribe {
                action: "subscribe".to_string(),
                subscriptions: vec![RtdsSubscription {
                    topic: "clob_market".to_string(),
                    msg_type: "price_change".to_string(),
                    filters,
                }],
            };

            let msg = serde_json::to_string(&subscribe_msg)?;
            debug!("RTDS subscribe message: {}", msg);
            write.send(Message::Text(msg)).await?;
            info!("Subscribed to {} markets", markets.len());
        }

        // Handle incoming messages
        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            manager.record_rtds_message();
                            Self::handle_message(manager.app(), &text);
                        }
                        Some(Ok(Message::Ping(data))) => {
                            write.send(Message::Pong(data)).await?;
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("RTDS server closed connection");
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
                    info!("RTDS shutdown requested");
                    let _ = write.send(Message::Close(None)).await;
                    return Ok(());
                }
            }
        }
    }

    // AIDEV-NOTE: RTDS uses abbreviated field names: m=market, pc=price_changes, a=asset_id, etc.
    fn handle_message(app: &AppHandle, text: &str) {
        // Skip empty messages (acknowledgments/heartbeats)
        if text.is_empty() || text == "{}" {
            return;
        }

        // Try to parse as wrapped RTDS message with abbreviated fields
        // Format: { connection_id, payload: { m: market, pc: [{ a, p, s, b, k, h }] } }
        match serde_json::from_str::<RtdsMessageWrapper>(text) {
            Ok(wrapper) => {
                if let Some(payload) = wrapper.payload {
                    let market = payload.m;
                    for change in payload.pc {
                        // Try to get price from best_bid (b), fall back to price (p)
                        let price_str = change.b.as_ref().or(change.p.as_ref());
                        if let Some(price_str) = price_str {
                            if let Ok(price) = price_str.parse::<f64>() {
                                let update = PriceUpdate {
                                    market: market.clone(),
                                    asset_id: change.a.clone(),
                                    price,
                                    timestamp: None, // RTDS doesn't include timestamp in this format
                                };
                                debug!("RTDS price update: {} -> {:.4}", change.a, price);
                                if let Err(e) = app.emit("price_update", &update) {
                                    error!("Failed to emit price_update: {}", e);
                                }
                            }
                        }
                    }
                }
                return;
            }
            Err(e) => {
                // Log parsing error for debugging with more detail
                let preview = if text.len() > 500 { &text[..500] } else { text };
                debug!("RTDS wrapper parse failed: {} - msg: {}", e, preview);
            }
        }

        // Try to parse as generic JSON for other message types
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            // Check for array of price changes
            if let Some(arr) = value.as_array() {
                for item in arr {
                    if let Ok(update) = serde_json::from_value::<RtdsPriceUpdate>(item.clone()) {
                        debug!("Price update: {:?}", update);
                        if let Err(e) = app.emit("price_update", &update) {
                            error!("Failed to emit price_update: {}", e);
                        }
                    }
                }
                return;
            }

            // Single price update object
            if let Ok(price_update) = serde_json::from_value::<RtdsPriceUpdate>(value.clone()) {
                debug!("Price update: {:?}", price_update);
                if let Err(e) = app.emit("price_update", &price_update) {
                    error!("Failed to emit price_update: {}", e);
                }
                return;
            }

            // Try to parse as trade
            if let Ok(trade) = serde_json::from_value::<RtdsTrade>(value.clone()) {
                debug!("Trade: {:?}", trade);
                if let Err(e) = app.emit("trade_update", &trade) {
                    error!("Failed to emit trade_update: {}", e);
                }
                return;
            }

            // Log unknown message structure (first 200 chars)
            let preview = if text.len() > 200 { &text[..200] } else { text };
            debug!("Unknown RTDS message structure: {}", preview);
        } else {
            debug!("Failed to parse RTDS message as JSON: {}", &text[..text.len().min(100)]);
        }
    }

    /// Subscribe to additional markets while connected
    pub async fn subscribe(&self, _market_ids: Vec<String>) -> Result<(), String> {
        // TODO: Implement runtime subscription - requires keeping the write half accessible
        Ok(())
    }

    /// Disconnect from RTDS
    pub fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

// RTDS Message Types

#[derive(Debug, Serialize)]
struct RtdsSubscribe {
    action: String,
    subscriptions: Vec<RtdsSubscription>,
}

#[derive(Debug, Serialize)]
struct RtdsSubscription {
    topic: String,
    #[serde(rename = "type")]
    msg_type: String,
    filters: String,
}

// AIDEV-NOTE: RTDS uses abbreviated field names to minimize bandwidth
// Full message format: { connection_id, payload: { m: market, pc: [{ a, p, s, b, k, h }] } }
// where: m=market, pc=price_changes, a=asset_id, p=price, s=size, b=best_bid, k=best_ask, h=hash

/// Wrapper for RTDS messages with connection_id
#[derive(Debug, Clone, Deserialize)]
struct RtdsMessageWrapper {
    #[allow(dead_code)]
    connection_id: Option<String>,
    payload: Option<RtdsPayload>,
}

/// Abbreviated payload structure
#[derive(Debug, Clone, Deserialize)]
struct RtdsPayload {
    m: String, // market (condition_id)
    pc: Vec<RtdsPriceChange>, // price_changes
}

/// Individual price change with abbreviated fields
/// All fields are optional except asset_id since RTDS doesn't always include all of them
#[derive(Debug, Clone, Deserialize)]
struct RtdsPriceChange {
    a: String,                      // asset_id (token_id) - always present
    p: Option<String>,              // price
    s: Option<String>,              // size
    b: Option<String>,              // best_bid
    k: Option<String>,              // best_ask
    h: Option<String>,              // hash
}

/// Price update emitted to frontend (matches clob.rs PriceUpdate)
#[derive(Debug, Clone, Serialize)]
pub struct PriceUpdate {
    pub market: String,
    pub asset_id: String,
    pub price: f64,
    pub timestamp: Option<i64>,
}

/// Price update from RTDS (legacy format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtdsPriceUpdate {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub market: String,
    pub price: f64,
    pub timestamp: Option<i64>,
}

/// Trade from RTDS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtdsTrade {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub market: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
    pub timestamp: Option<i64>,
}
