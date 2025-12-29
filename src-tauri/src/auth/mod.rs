// AIDEV-NOTE: Auth module handles Polymarket authentication (EIP-712, HMAC, keyring)

mod credentials;
mod eip712;
mod hmac;
mod keyring;

pub use credentials::{ApiCredentials, AuthStatus};
pub use eip712::{L1Headers, PolymarketSigner};
pub use hmac::HmacAuth;
pub use keyring::CredentialStore;
