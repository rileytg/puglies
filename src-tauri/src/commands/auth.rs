// AIDEV-NOTE: Tauri commands for authentication - login/logout/status/portfolio
// Uses SQLite database for credential persistence

use tauri::State;

use crate::api::{clob::{Balance, Order, Position}, ClobClient};
use crate::auth::{AuthStatus, PolymarketSigner};
use crate::error::AppError;
use crate::AuthState;

/// Extended auth status including polymarket address
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedAuthStatus {
    pub is_authenticated: bool,
    pub address: Option<String>,
    pub polymarket_address: Option<String>,
}

/// Get current authentication status
#[tauri::command]
pub async fn get_auth_status(state: State<'_, AuthState>) -> Result<ExtendedAuthStatus, AppError> {
    let credentials = state.credentials.read();
    let polymarket_address = state.polymarket_address.read();

    let status = ExtendedAuthStatus {
        is_authenticated: credentials.is_some(),
        address: credentials.as_ref().map(|c| c.address.clone()),
        polymarket_address: polymarket_address.clone(),
    };

    tracing::debug!("get_auth_status: authenticated={}, polymarket_addr={:?}",
        status.is_authenticated, status.polymarket_address);

    Ok(status)
}

/// Login with private key - derives API credentials and stores them
#[tauri::command]
pub async fn login(private_key: String, state: State<'_, AuthState>) -> Result<ExtendedAuthStatus, AppError> {
    tracing::info!("Starting login flow");

    // Clean up the private key - trim whitespace
    let clean_key = private_key.trim();

    // Validate key format
    let key_hex = clean_key.strip_prefix("0x").unwrap_or(clean_key);
    if key_hex.len() != 64 {
        return Err(AppError::Internal(format!(
            "Invalid private key: expected 64 hex characters, got {}",
            key_hex.len()
        )));
    }

    // Create signer from private key
    let signer = PolymarketSigner::from_private_key(clean_key)?;
    let address = signer.address_string();

    tracing::info!("Signing with address: {}", address);

    // Derive API credentials
    let clob_client = ClobClient::new();
    let credentials = clob_client.derive_api_key(&signer).await?;

    tracing::info!("API key derived successfully");

    // Get existing polymarket address if any
    let polymarket_address = state.polymarket_address.read().clone();

    // Store in database
    state.database.store_credentials(&credentials, polymarket_address.as_deref())?;

    // Update state
    {
        let mut creds = state.credentials.write();
        *creds = Some(credentials.clone());
    }

    {
        let mut client = state.clob_client.write();
        client.set_credentials(&credentials);
    }

    tracing::info!("Login successful for {}", address);

    Ok(ExtendedAuthStatus {
        is_authenticated: true,
        address: Some(address),
        polymarket_address,
    })
}

/// Logout - clear credentials from database and state
#[tauri::command]
pub async fn logout(state: State<'_, AuthState>) -> Result<ExtendedAuthStatus, AppError> {
    tracing::info!("Logging out");

    // Delete from database
    state.database.delete_credentials()?;

    // Clear state
    {
        let mut creds = state.credentials.write();
        *creds = None;
    }

    {
        let mut client = state.clob_client.write();
        *client = ClobClient::new();
    }

    {
        let mut poly_addr = state.polymarket_address.write();
        *poly_addr = None;
    }

    tracing::info!("Logout successful");

    Ok(ExtendedAuthStatus {
        is_authenticated: false,
        address: None,
        polymarket_address: None,
    })
}

/// Set/update the Polymarket address (for fetching positions)
#[tauri::command]
pub async fn set_polymarket_address(address: String, state: State<'_, AuthState>) -> Result<(), AppError> {
    tracing::info!("Setting polymarket address: {}", address);

    // Update in database if credentials exist
    if state.credentials.read().is_some() {
        state.database.update_polymarket_address(&address)?;
    }

    // Update state
    {
        let mut poly_addr = state.polymarket_address.write();
        *poly_addr = Some(address);
    }

    Ok(())
}

/// Get user's USDC balance
#[tauri::command]
pub async fn get_balance(state: State<'_, AuthState>) -> Result<Balance, AppError> {
    tracing::debug!("get_balance command called");

    // Debug: Check credentials
    if let Some(creds) = state.credentials.read().as_ref() {
        tracing::debug!(
            "Credentials: key_len={}, secret_len={}, passphrase_len={}, addr={}",
            creds.api_key.len(),
            creds.api_secret.len(),
            creds.api_passphrase.len(),
            creds.address
        );
    } else {
        tracing::warn!("No credentials in state!");
    }

    // Clone the client to avoid holding the guard across await
    let client = state.clob_client.read().clone();
    let result = client.get_balance().await;
    match &result {
        Ok(balance) => {
            tracing::debug!("Balance: {:?}", balance);
            tracing::info!("Returning balance to frontend: {}", balance.balance);
        },
        Err(e) => tracing::error!("Balance error: {:?}", e),
    }
    result
}

/// Get user's positions (requires Polymarket address, may differ from signing address)
#[tauri::command]
pub async fn get_positions(address: String, state: State<'_, AuthState>) -> Result<Vec<Position>, AppError> {
    // Clone the client to avoid holding the guard across await
    let client = state.clob_client.read().clone();
    client.get_positions(&address).await
}

/// Get user's open orders
#[tauri::command]
pub async fn get_orders(state: State<'_, AuthState>) -> Result<Vec<Order>, AppError> {
    // Clone the client to avoid holding the guard across await
    let client = state.clob_client.read().clone();
    client.get_orders().await
}
