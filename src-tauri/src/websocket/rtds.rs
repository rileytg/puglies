// AIDEV-NOTE: RTDS WebSocket client for real-time market activity (prices, trades)
// Connects to wss://ws-live-data.polymarket.com (Socket.IO protocol)

use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

use super::manager::{ConnectionState, ReconnectConfig, WebSocketManager};

const RTDS_URL: &str = "wss://ws-live-data.polymarket.com/ws";

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

        // Subscribe to markets
        for market_id in markets {
            let subscribe_msg = RtdsSubscribe {
                action: "subscribe".to_string(),
                channel: "market".to_string(),
                market: market_id.clone(),
            };

            let msg = serde_json::to_string(&subscribe_msg)?;
            write.send(Message::Text(msg)).await?;
            debug!("Subscribed to market: {}", market_id);
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

    fn handle_message(app: &AppHandle, text: &str) {
        // Try to parse as price update
        if let Ok(price_update) = serde_json::from_str::<RtdsPriceUpdate>(text) {
            debug!("Price update: {:?}", price_update);
            if let Err(e) = app.emit("price_update", &price_update) {
                error!("Failed to emit price_update: {}", e);
            }
            return;
        }

        // Try to parse as trade
        if let Ok(trade) = serde_json::from_str::<RtdsTrade>(text) {
            debug!("Trade: {:?}", trade);
            if let Err(e) = app.emit("trade_update", &trade) {
                error!("Failed to emit trade_update: {}", e);
            }
            return;
        }

        // Unknown message type
        debug!("Unknown RTDS message: {}", text);
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
    channel: String,
    market: String,
}

/// Price update from RTDS
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
