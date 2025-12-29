// AIDEV-NOTE: Auth module for Polymarket authentication (EIP-712, HMAC)
// NOTE: keyring module stays in src-tauri (OS-specific credential storage)

mod credentials;
mod eip712;
mod hmac;
mod order_eip712;

#[cfg(test)]
mod tests;

pub use credentials::{ApiCredentials, AuthStatus};
pub use eip712::{L1Headers, PolymarketSigner};
pub use hmac::{AuthHeaders, HmacAuth};
pub use order_eip712::OrderSigner;
