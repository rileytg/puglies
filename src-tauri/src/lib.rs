mod api;
mod commands;
mod error;
mod types;

use api::GammaClient;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        .invoke_handler(tauri::generate_handler![
            commands::get_markets,
            commands::get_market,
            commands::get_events,
            commands::search_markets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
