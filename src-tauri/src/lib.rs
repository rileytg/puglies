mod api;
mod commands;
mod error;
mod types;
mod websocket;

use std::sync::Arc;
use api::GammaClient;
use parking_lot::RwLock;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use websocket::{WebSocketManager, RtdsClient, ClobWebSocket};

/// Shared state for WebSocket connections
pub struct WebSocketState {
    pub manager: Arc<WebSocketManager>,
    pub rtds: RwLock<Option<RtdsClient>>,
    pub clob: RwLock<Option<ClobWebSocket>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "plgui=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create API clients
    let gamma_client = GammaClient::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(gamma_client)
        .setup(|app| {
            // Initialize WebSocket manager after app is ready
            let ws_manager = Arc::new(WebSocketManager::new(app.handle().clone()));
            let ws_state = WebSocketState {
                manager: ws_manager.clone(),
                rtds: RwLock::new(None),
                clob: RwLock::new(None),
            };
            app.manage(ws_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_markets,
            commands::get_market,
            commands::get_events,
            commands::search_markets,
            commands::connect_rtds,
            commands::disconnect_rtds,
            commands::connect_clob,
            commands::disconnect_clob,
            commands::get_connection_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
