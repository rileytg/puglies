use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, instrument};

use crate::api::{GammaClient, ClobClient};
use crate::api::clob::PricePoint;
use crate::error::AppError;
use crate::types::{Event, Market};
use crate::AuthState;

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

/// Fetch a single market by internal ID
/// AIDEV-NOTE: Uses Gamma API internal ID (numeric), not condition_id (hex)
#[tauri::command]
#[instrument(skip(gamma_client))]
pub async fn get_market(
    gamma_client: State<'_, GammaClient>,
    market_id: String,
) -> Result<Market, AppError> {
    gamma_client.get_market(&market_id).await
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

// ========== Price History ==========

/// Price history request parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistoryParams {
    /// Token ID (CLOB token ID, long numeric string)
    pub token_id: String,
    /// Time interval: "1h", "6h", "1d", "1w", "max" (default: "max")
    #[serde(default)]
    pub interval: Option<String>,
    /// Resolution in minutes (e.g., 60 for hourly)
    #[serde(default)]
    pub fidelity: Option<u32>,
}

/// Price history response for frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistoryResult {
    /// Array of {t: timestamp, p: price} points
    pub history: Vec<PricePoint>,
    /// Number of cached points
    pub cached_count: usize,
    /// Number of freshly fetched points
    pub fetched_count: usize,
}

/// Fetch price history for a token with caching
/// AIDEV-NOTE: Checks DB cache first, fetches new data from API if needed
#[tauri::command]
#[instrument(skip(auth_state))]
pub async fn get_price_history(
    auth_state: State<'_, AuthState>,
    params: PriceHistoryParams,
) -> Result<PriceHistoryResult, AppError> {
    let token_id = &params.token_id;
    let db = &auth_state.database;

    // 1. Check cached data
    let cached = db.get_price_history(token_id, None, None)?;
    let cached_count = cached.len();
    debug!("Found {} cached price history points for {}", cached_count, token_id);

    // 2. Determine if we need to fetch new data
    let latest_cached_ts = db.get_latest_price_timestamp(token_id)?;
    let now = chrono::Utc::now().timestamp();

    // Fetch if no cache or cache is older than 5 minutes
    let should_fetch = match latest_cached_ts {
        None => true,
        Some(ts) => (now - ts) > 300, // 5 minutes
    };

    let mut fetched_count = 0;

    if should_fetch {
        // 3. Fetch from API
        // AIDEV-NOTE: Clone client to avoid holding lock across await
        let clob_client = auth_state.clob_client.read().clone();

        // Use startTs if we have cached data to get incremental updates
        let start_ts = latest_cached_ts.map(|ts| ts + 1);

        let api_result = clob_client
            .get_price_history(
                token_id,
                params.interval.as_deref(),
                params.fidelity,
                start_ts,
                None,
            )
            .await;

        match api_result {
            Ok(points) => {
                fetched_count = points.len();
                debug!("Fetched {} new price history points from API", fetched_count);

                if !points.is_empty() {
                    // 4. Store in cache
                    let tuples: Vec<(i64, f64)> = points.iter().map(|p| (p.t, p.p)).collect();
                    if let Err(e) = db.store_price_history(token_id, &tuples) {
                        debug!("Failed to cache price history: {}", e);
                    }
                }
            }
            Err(e) => {
                // Log but don't fail - return cached data if available
                debug!("Failed to fetch price history from API: {}", e);
                if cached_count == 0 {
                    return Err(e);
                }
            }
        }
    }

    // 5. Get final combined data from cache (now includes any new points)
    let final_data = db.get_price_history(token_id, None, None)?;

    // Convert to PricePoints
    let history: Vec<PricePoint> = final_data
        .into_iter()
        .map(|(t, p)| PricePoint { t, p })
        .collect();

    Ok(PriceHistoryResult {
        history,
        cached_count,
        fetched_count,
    })
}
