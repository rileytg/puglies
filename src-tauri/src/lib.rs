mod api;
mod auth;
mod commands;
mod db;
mod error;
mod types;
mod websocket;

use std::sync::Arc;
use api::{GammaClient, ClobClient};
use auth::ApiCredentials;
use db::Database;
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

/// Shared state for authentication
pub struct AuthState {
    pub credentials: RwLock<Option<ApiCredentials>>,
    pub clob_client: RwLock<ClobClient>,
    pub database: Arc<Database>,
    pub polymarket_address: RwLock<Option<String>>,
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

            // Initialize database and load existing credentials
            let database = Arc::new(Database::new()
                .expect("Failed to initialize database"));

            let (credentials, clob_client, polymarket_address) = match database.load_credentials() {
                Ok(Some((creds, poly_addr))) => {
                    tracing::info!("Found existing credentials for {}", creds.address);
                    let client = ClobClient::with_credentials(&creds);
                    (Some(creds), client, poly_addr)
                }
                Ok(None) => {
                    tracing::debug!("No stored credentials found");
                    (None, ClobClient::new(), None)
                }
                Err(e) => {
                    tracing::warn!("Failed to retrieve credentials: {}", e);
                    (None, ClobClient::new(), None)
                }
            };

            let auth_state = AuthState {
                credentials: RwLock::new(credentials),
                clob_client: RwLock::new(clob_client),
                database,
                polymarket_address: RwLock::new(polymarket_address),
            };
            app.manage(auth_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Market commands
            commands::get_markets,
            commands::get_market,
            commands::get_events,
            commands::search_markets,
            commands::get_price_history,
            // WebSocket commands
            commands::connect_rtds,
            commands::disconnect_rtds,
            commands::connect_clob,
            commands::disconnect_clob,
            commands::get_connection_status,
            // Auth commands
            commands::get_auth_status,
            commands::login,
            commands::logout,
            commands::set_polymarket_address,
            commands::get_balance,
            commands::get_positions,
            commands::get_orders,
            // Trading commands
            commands::place_order,
            commands::cancel_order,
            commands::cancel_all_orders,
            commands::cancel_market_orders,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
