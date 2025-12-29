// AIDEV-NOTE: Order structures for Polymarket CTF Exchange trading
// These types are used for EIP-712 order signing and CLOB API requests

use serde::{Deserialize, Serialize};

/// Side of the order (matches Polymarket enum: Buy=0, Sell=1)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

impl OrderSide {
    pub fn as_u8(&self) -> u8 {
        match self {
            OrderSide::Buy => 0,
            OrderSide::Sell => 1,
        }
    }
}

/// Signature type for orders (matches Polymarket enum)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SignatureType {
    /// EIP712 signature signed by an EOA
    Eoa = 0,
    /// EIP712 signature signed by Polymarket proxy wallet
    Proxy = 1,
    /// EIP712 signature signed by Gnosis Safe
    GnosisSafe = 2,
}

impl SignatureType {
    pub fn as_u8(&self) -> u8 {
        match self {
            SignatureType::Eoa => 0,
            SignatureType::Proxy => 1,
            SignatureType::GnosisSafe => 2,
        }
    }
}

/// Order type for time-in-force
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good-til-cancelled
    Gtc,
    /// Fill-or-kill
    Fok,
    /// Good-til-date
    Gtd,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Gtc => write!(f, "GTC"),
            OrderType::Fok => write!(f, "FOK"),
            OrderType::Gtd => write!(f, "GTD"),
        }
    }
}

/// Unsigned order structure (before EIP-712 signing)
/// AIDEV-NOTE: Field order and types must match CTF Exchange contract exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedOrder {
    /// Random salt for uniqueness (uint256 as string)
    pub salt: String,
    /// Maker/funder address
    pub maker: String,
    /// Signer address (usually same as maker for EOA)
    pub signer: String,
    /// Taker address (0x0 for open orders)
    pub taker: String,
    /// ERC1155 token ID of conditional token
    pub token_id: String,
    /// Amount maker is offering (in wei, 6 decimals)
    pub maker_amount: String,
    /// Amount maker wants in return (in wei, 6 decimals)
    pub taker_amount: String,
    /// Unix expiration timestamp
    pub expiration: String,
    /// Unique nonce for this order
    pub nonce: String,
    /// Fee rate in basis points
    pub fee_rate_bps: String,
    /// Buy or Sell
    pub side: OrderSide,
    /// Signature type enum
    pub signature_type: SignatureType,
}

/// Signed order with EIP-712 signature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrder {
    /// The order data
    #[serde(flatten)]
    pub order: UnsignedOrder,
    /// Hex-encoded signature (0x-prefixed, 65 bytes)
    pub signature: String,
}

/// Request payload for POST /order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    /// The signed order
    pub order: SignedOrder,
    /// API key owner address
    pub owner: String,
    /// Order type (GTC, FOK, GTD)
    pub order_type: OrderType,
}

/// Response from POST /order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderResponse {
    /// Whether the order was accepted
    pub success: bool,
    /// Error message if failed
    #[serde(default)]
    pub error_msg: Option<String>,
    /// Order ID if successful
    #[serde(default)]
    pub order_id: Option<String>,
    /// Order hashes
    #[serde(default)]
    pub order_hashes: Option<Vec<String>>,
    /// Order status: "matched", "live", "delayed", or "unmatched"
    #[serde(default)]
    pub status: Option<String>,
}

/// User-facing order parameters (before conversion to wire format)
/// AIDEV-NOTE: This is what the frontend sends - we convert to UnsignedOrder
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderParams {
    /// Token ID to trade
    pub token_id: String,
    /// Buy or Sell
    pub side: OrderSide,
    /// Limit price (0.0-1.0, e.g., 0.65 = 65 cents)
    pub price: f64,
    /// Number of shares
    pub size: f64,
    /// Order type (GTC, FOK, GTD)
    pub order_type: OrderType,
    /// Seconds until expiration (None = 30 days default)
    #[serde(default)]
    pub expiration_secs: Option<u64>,
}

/// Response for cancel operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    /// Successfully canceled order IDs
    #[serde(default)]
    pub canceled: Vec<String>,
    /// Orders that failed to cancel with reasons
    #[serde(default)]
    pub not_canceled: std::collections::HashMap<String, String>,
}
