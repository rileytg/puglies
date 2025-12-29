// AIDEV-NOTE: Auth module - app-specific auth concerns
// All auth types come from polymarket_rs; keyring.rs kept for future secure storage option

// NOTE: keyring.rs exists but is not currently used (we use SQLite via db.rs)
// Uncomment when ready to use OS keychain for more secure credential storage:
// mod keyring;
// pub use keyring::CredentialStore;
