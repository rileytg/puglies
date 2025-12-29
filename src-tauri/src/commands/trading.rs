// AIDEV-NOTE: Tauri commands for trading - place/cancel orders
// Orders require EIP-712 signing with CTF Exchange domain

use tauri::State;

use polymarket_rs::api::order::{
    CancelResponse, OrderParams, OrderSide, PlaceOrderResponse,
    SignatureType, UnsignedOrder,
};
use polymarket_rs::OrderSigner;
use crate::error::AppError;
use crate::AuthState;

/// Place a new order
/// AIDEV-NOTE: Each order requires a fresh signature, so we need the private key
#[tauri::command]
pub async fn place_order(
    params: OrderParams,
    private_key: String,
    state: State<'_, AuthState>,
) -> Result<PlaceOrderResponse, AppError> {
    tracing::info!("Placing order: side={:?}, price={}, size={}", params.side, params.price, params.size);

    // Validate params
    if params.price <= 0.0 || params.price >= 1.0 {
        return Err(AppError::Internal(format!(
            "Invalid price: must be between 0 and 1, got {}", params.price
        )));
    }
    if params.size <= 0.0 {
        return Err(AppError::Internal("Invalid size: must be positive".to_string()));
    }

    // Get owner address from credentials
    let owner = {
        let credentials = state.credentials.read();
        credentials.as_ref()
            .map(|c| c.address.clone())
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?
    };

    // Create order signer
    let signer = OrderSigner::from_private_key(&private_key)?;
    let signer_address = signer.address_string();

    tracing::debug!("Signer address: {}, Owner address: {}", signer_address, owner);

    // Build unsigned order from params
    let unsigned_order = build_order_from_params(&params, &owner, &signer_address)?;

    tracing::debug!("Built order: salt={}, maker_amount={}, taker_amount={}",
        unsigned_order.salt, unsigned_order.maker_amount, unsigned_order.taker_amount);

    // Sign the order using EIP-712
    let signed_order = signer.sign_order(&unsigned_order).await?;

    tracing::debug!("Order signed: {}", signed_order.signature);

    // Place via API
    let client = state.clob_client.read().clone();
    let result = client.place_order(signed_order, &owner, params.order_type).await?;

    if result.success {
        tracing::info!("Order placed successfully: {:?}", result.order_id);
    } else {
        tracing::warn!("Order placement failed: {:?}", result.error_msg);
    }

    Ok(result)
}

/// Cancel a specific order by ID
#[tauri::command]
pub async fn cancel_order(
    order_id: String,
    state: State<'_, AuthState>,
) -> Result<CancelResponse, AppError> {
    tracing::info!("Cancelling order: {}", order_id);

    let client = state.clob_client.read().clone();
    client.cancel_order(&order_id).await.map_err(AppError::from)
}

/// Cancel all open orders
#[tauri::command]
pub async fn cancel_all_orders(
    state: State<'_, AuthState>,
) -> Result<CancelResponse, AppError> {
    tracing::info!("Cancelling all orders");

    let client = state.clob_client.read().clone();
    client.cancel_all_orders().await.map_err(AppError::from)
}

/// Cancel all orders for a specific market
#[tauri::command]
pub async fn cancel_market_orders(
    market_id: String,
    state: State<'_, AuthState>,
) -> Result<CancelResponse, AppError> {
    tracing::info!("Cancelling orders for market: {}", market_id);

    let client = state.clob_client.read().clone();
    client.cancel_market_orders(&market_id).await.map_err(AppError::from)
}

/// Build an unsigned order from user-friendly parameters
/// AIDEV-NOTE: Converts price/size to makerAmount/takerAmount based on side
fn build_order_from_params(
    params: &OrderParams,
    owner: &str,
    signer_address: &str,
) -> Result<UnsignedOrder, AppError> {
    use rand::Rng;

    // Generate random salt (128-bit for sufficient uniqueness)
    let salt: u128 = rand::thread_rng().gen();

    // AIDEV-NOTE: Polymarket uses 6 decimals for both USDC and share amounts
    let decimals: f64 = 1_000_000.0; // 10^6

    // Calculate maker/taker amounts based on side
    // For BUY: maker offers USDC, gets shares
    // For SELL: maker offers shares, gets USDC
    let (maker_amount, taker_amount) = match params.side {
        OrderSide::Buy => {
            // Buying: spend USDC to get shares
            // maker_amount = price * size (USDC we're spending)
            // taker_amount = size (shares we're getting)
            let usdc_amount = (params.price * params.size * decimals).round() as u64;
            let share_amount = (params.size * decimals).round() as u64;
            (usdc_amount, share_amount)
        }
        OrderSide::Sell => {
            // Selling: spend shares to get USDC
            // maker_amount = size (shares we're spending)
            // taker_amount = price * size (USDC we're getting)
            let share_amount = (params.size * decimals).round() as u64;
            let usdc_amount = (params.price * params.size * decimals).round() as u64;
            (share_amount, usdc_amount)
        }
    };

    // Expiration: default 30 days from now
    let expiration_secs = params.expiration_secs.unwrap_or(30 * 24 * 60 * 60);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("Time error: {}", e)))?
        .as_secs();
    let expiration = now + expiration_secs;

    // Nonce: use current timestamp in milliseconds for uniqueness
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("Time error: {}", e)))?
        .as_millis() as u64;

    Ok(UnsignedOrder {
        salt: salt.to_string(),
        maker: owner.to_string(),
        signer: signer_address.to_string(),
        // Open order: any taker can fill
        taker: "0x0000000000000000000000000000000000000000".to_string(),
        token_id: params.token_id.clone(),
        maker_amount: maker_amount.to_string(),
        taker_amount: taker_amount.to_string(),
        expiration: expiration.to_string(),
        nonce: nonce.to_string(),
        // AIDEV-NOTE: Fee rate defaults to 0, Polymarket may add their own
        fee_rate_bps: "0".to_string(),
        side: params.side,
        // AIDEV-NOTE: Using Proxy signature type for Polymarket proxy wallets
        signature_type: SignatureType::Proxy,
    })
}
