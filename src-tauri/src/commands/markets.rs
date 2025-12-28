use tauri::State;
use tracing::instrument;

use crate::api::GammaClient;
use crate::error::AppError;
use crate::types::{Event, Market};

// AIDEV-NOTE: Commands are invoked from frontend via invoke("command_name", { args })
// Keep command signatures in sync with src/lib/tauri.ts

/// Fetch markets from Gamma API
#[tauri::command]
#[instrument(skip(gamma_client))]
pub async fn get_markets(
    gamma_client: State<'_, GammaClient>,
    query: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Market>, AppError> {
    gamma_client
        .get_markets(query.as_deref(), limit, offset)
        .await
}

/// Fetch a single market by condition ID
#[tauri::command]
#[instrument(skip(gamma_client))]
pub async fn get_market(
    gamma_client: State<'_, GammaClient>,
    condition_id: String,
) -> Result<Market, AppError> {
    gamma_client.get_market(&condition_id).await
}

/// Fetch events (market collections)
#[tauri::command]
#[instrument(skip(gamma_client))]
pub async fn get_events(
    gamma_client: State<'_, GammaClient>,
    limit: Option<u32>,
) -> Result<Vec<Event>, AppError> {
    gamma_client.get_events(limit).await
}

/// Search markets by text query
#[tauri::command]
#[instrument(skip(gamma_client))]
pub async fn search_markets(
    gamma_client: State<'_, GammaClient>,
    query: String,
) -> Result<Vec<Market>, AppError> {
    gamma_client.search_markets(&query).await
}
