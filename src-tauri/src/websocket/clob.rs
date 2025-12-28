// AIDEV-NOTE: CLOB WebSocket client for order book depth data
// Connects to wss://ws-subscriptions-clob.polymarket.com

use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

use super::manager::{ConnectionState, ReconnectConfig, WebSocketManager};

const CLOB_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// CLOB WebSocket client for order book data
pub struct ClobWebSocket {
    manager: Arc<WebSocketManager>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ClobWebSocket {
    pub fn new(manager: Arc<WebSocketManager>) -> Self {
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
                        let delay = WebSocketManager::calculate_reconnect_delay(attempts, &config);
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
        manager: &Arc<WebSocketManager>,
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
                            Self::handle_message(manager.app(), &text);
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

    fn handle_message(app: &AppHandle, text: &str) {
        // Try to parse as order book snapshot
        if let Ok(snapshot) = serde_json::from_str::<OrderBookSnapshot>(text) {
            if snapshot.event_type.as_deref() == Some("book") {
                debug!("Order book snapshot for {}", snapshot.asset_id);
                if let Err(e) = app.emit("orderbook_snapshot", &snapshot) {
                    error!("Failed to emit orderbook_snapshot: {}", e);
                }
                return;
            }
        }

        // Try to parse as order book delta
        if let Ok(delta) = serde_json::from_str::<OrderBookDelta>(text) {
            if delta.event_type.as_deref() == Some("price_change") {
                debug!("Order book delta for {}", delta.asset_id);
                if let Err(e) = app.emit("orderbook_delta", &delta) {
                    error!("Failed to emit orderbook_delta: {}", e);
                }
                return;
            }
        }

        // Try to parse as trade event
        if let Ok(trade) = serde_json::from_str::<ClobTrade>(text) {
            if trade.event_type.as_deref() == Some("trade") {
                debug!("CLOB trade: {:?}", trade);
                if let Err(e) = app.emit("clob_trade", &trade) {
                    error!("Failed to emit clob_trade: {}", e);
                }
                return;
            }
        }

        debug!("Unknown CLOB message: {}", text);
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

/// Order book snapshot from CLOB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    pub asset_id: String,
    pub market: Option<String>,
    pub hash: Option<String>,
    pub timestamp: Option<i64>,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

/// Single level in the order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: String,
    pub size: String,
}

/// Order book delta (incremental update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDelta {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    pub asset_id: String,
    pub market: Option<String>,
    pub side: String,
    pub price: String,
    pub size: String,
    pub timestamp: Option<i64>,
}

/// Trade event from CLOB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobTrade {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    pub asset_id: String,
    pub market: Option<String>,
    pub price: String,
    pub size: String,
    pub side: String,
    pub timestamp: Option<i64>,
    pub trade_id: Option<String>,
}
