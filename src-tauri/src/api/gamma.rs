use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, instrument};

use crate::error::AppError;
use crate::types::{Event, Market};

const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

/// Client for the Polymarket Gamma API (market metadata)
#[derive(Clone)]
pub struct GammaClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct MarketsResponse(Vec<Market>);

#[derive(Debug, Deserialize)]
struct EventsResponse(Vec<Event>);

impl GammaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: GAMMA_API_BASE.to_string(),
        }
    }

    /// Fetch markets with optional filtering
    #[instrument(skip(self))]
    pub async fn get_markets(
        &self,
        query: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Market>, AppError> {
        let mut url = format!("{}/markets", self.base_url);
        let mut params = Vec::new();

        // Only show active, non-closed markets by default
        params.push("active=true".to_string());
        params.push("closed=false".to_string());
        params.push("archived=false".to_string());

        if let Some(q) = query {
            if !q.is_empty() {
                params.push(format!("slug_contains={}", urlencoding::encode(q)));
            }
        }

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        } else {
            params.push("limit=50".to_string());
        }

        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        // Sort by volume descending
        params.push("order=volume_num".to_string());
        params.push("ascending=false".to_string());

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        debug!("Fetching markets from: {}", url);

        let response = self.client.get(&url).send().await?;
        let markets: Vec<Market> = response.json().await?;

        Ok(markets)
    }

    /// Fetch a single market by condition ID
    #[instrument(skip(self))]
    pub async fn get_market(&self, condition_id: &str) -> Result<Market, AppError> {
        let url = format!("{}/markets/{}", self.base_url, condition_id);

        debug!("Fetching market: {}", url);

        let response = self.client.get(&url).send().await?;

        if response.status() == 404 {
            return Err(AppError::MarketNotFound(condition_id.to_string()));
        }

        let market: Market = response.json().await?;
        Ok(market)
    }

    /// Fetch events (market collections)
    #[instrument(skip(self))]
    pub async fn get_events(&self, limit: Option<u32>) -> Result<Vec<Event>, AppError> {
        let mut url = format!("{}/events", self.base_url);
        let mut params = Vec::new();

        params.push("active=true".to_string());
        params.push("closed=false".to_string());

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        } else {
            params.push("limit=20".to_string());
        }

        // Sort by volume
        params.push("order=volume".to_string());
        params.push("ascending=false".to_string());

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        debug!("Fetching events from: {}", url);

        let response = self.client.get(&url).send().await?;
        let events: Vec<Event> = response.json().await?;

        Ok(events)
    }

    /// Search markets by text query
    #[instrument(skip(self))]
    pub async fn search_markets(&self, query: &str) -> Result<Vec<Market>, AppError> {
        // Use the text_query parameter for search
        let url = format!(
            "{}/markets?text_query={}&active=true&closed=false&limit=20",
            self.base_url,
            urlencoding::encode(query)
        );

        debug!("Searching markets: {}", url);

        let response = self.client.get(&url).send().await?;
        let markets: Vec<Market> = response.json().await?;

        Ok(markets)
    }
}

impl Default for GammaClient {
    fn default() -> Self {
        Self::new()
    }
}
