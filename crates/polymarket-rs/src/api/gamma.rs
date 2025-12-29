// AIDEV-NOTE: Gamma API client for market metadata (public, no auth)

use reqwest::Client;
use tracing::{debug, error, instrument};

use crate::error::ApiError;
use crate::types::{Event, Market, RawMarket};

const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

/// Client for the Polymarket Gamma API (market metadata)
#[derive(Clone)]
pub struct GammaClient {
    client: Client,
    base_url: String,
}

// AIDEV-NOTE: API returns arrays directly, not wrapped objects

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
    ) -> Result<Vec<Market>, ApiError> {
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

        // Sort by volume descending (API uses camelCase)
        params.push("order=volumeNum".to_string());
        params.push("ascending=false".to_string());

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        debug!("Fetching markets from: {}", url);

        let response = self.client.get(&url).send().await?;
        let text = response.text().await?;

        let raw_markets: Vec<RawMarket> = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to parse markets: {}", e);
                tracing::error!("Response text (first 500 chars): {}", &text[..text.len().min(500)]);
                return Err(ApiError::Json(e));
            }
        };

        debug!("Parsed {} markets successfully", raw_markets.len());
        let markets: Vec<Market> = raw_markets.into_iter().map(Market::from).collect();

        Ok(markets)
    }

    /// Fetch a single market by its internal ID
    #[instrument(skip(self))]
    pub async fn get_market(&self, market_id: &str) -> Result<Market, ApiError> {
        // AIDEV-NOTE: Gamma API uses internal numeric ID in path, not condition_id
        let url = format!("{}/markets/{}", self.base_url, market_id);

        debug!("Fetching market: {}", url);

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        debug!("Market response status: {}", status);

        if status == 404 {
            return Err(ApiError::MarketNotFound(market_id.to_string()));
        }

        let body = response.text().await?;
        debug!("Market response body length: {} chars", body.len());

        let raw_market: RawMarket = serde_json::from_str(&body).map_err(|e| {
            error!("Failed to parse market JSON: {}", e);
            debug!("Raw response: {}", &body[..body.len().min(500)]);
            ApiError::Api(format!("Failed to parse market: {}", e))
        })?;

        Ok(Market::from(raw_market))
    }

    /// Fetch events (market collections)
    #[instrument(skip(self))]
    pub async fn get_events(&self, limit: Option<u32>) -> Result<Vec<Event>, ApiError> {
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
    pub async fn search_markets(&self, query: &str) -> Result<Vec<Market>, ApiError> {
        // Use the text_query parameter for search
        let url = format!(
            "{}/markets?text_query={}&active=true&closed=false&limit=20",
            self.base_url,
            urlencoding::encode(query)
        );

        debug!("Searching markets: {}", url);

        let response = self.client.get(&url).send().await?;
        let raw_markets: Vec<RawMarket> = response.json().await?;
        let markets: Vec<Market> = raw_markets.into_iter().map(Market::from).collect();

        Ok(markets)
    }
}

impl Default for GammaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "hits real API"]
    async fn test_get_markets() {
        let client = GammaClient::new();
        let markets = client.get_markets(None, Some(5), None).await.unwrap();
        assert!(markets.len() <= 5);
        assert!(!markets.is_empty());
    }

    #[tokio::test]
    #[ignore = "hits real API"]
    async fn test_search_markets() {
        let client = GammaClient::new();
        let markets = client.search_markets("bitcoin").await.unwrap();
        // Should find some bitcoin-related markets
        assert!(markets.iter().any(|m|
            m.question.to_lowercase().contains("bitcoin") ||
            m.question.to_lowercase().contains("btc")
        ));
    }
}
