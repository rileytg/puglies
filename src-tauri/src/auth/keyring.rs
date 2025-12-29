// AIDEV-NOTE: Secure credential storage using OS keyring
// Stores API credentials in macOS Keychain / Windows Credential Manager / Linux Secret Service

use keyring::Entry;

use crate::auth::ApiCredentials;
use crate::error::AppError;

const SERVICE_NAME: &str = "plgui-polymarket";
const CREDENTIALS_KEY: &str = "api-credentials";

/// Secure credential storage using the OS keyring
pub struct CredentialStore {
    entry: Entry,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new() -> Result<Self, AppError> {
        let entry = Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| AppError::Internal(format!("Failed to access keyring: {}", e)))?;

        Ok(Self { entry })
    }

    /// Store credentials in the keyring
    pub fn store(&self, credentials: &ApiCredentials) -> Result<(), AppError> {
        let json = serde_json::to_string(credentials)
            .map_err(|e| AppError::Internal(format!("Failed to serialize credentials: {}", e)))?;

        self.entry
            .set_password(&json)
            .map_err(|e| AppError::Internal(format!("Failed to store credentials: {}", e)))?;

        tracing::info!("Credentials stored securely in keyring");
        Ok(())
    }

    /// Retrieve credentials from the keyring
    pub fn retrieve(&self) -> Result<Option<ApiCredentials>, AppError> {
        match self.entry.get_password() {
            Ok(json) => {
                let credentials: ApiCredentials = serde_json::from_str(&json)
                    .map_err(|e| AppError::Internal(format!("Failed to parse credentials: {}", e)))?;
                Ok(Some(credentials))
            }
            Err(keyring::Error::NoEntry) => {
                tracing::debug!("No credentials found in keyring");
                Ok(None)
            }
            Err(e) => {
                Err(AppError::Internal(format!("Failed to retrieve credentials: {}", e)))
            }
        }
    }

    /// Delete credentials from the keyring (logout)
    pub fn delete(&self) -> Result<(), AppError> {
        match self.entry.delete_credential() {
            Ok(()) => {
                tracing::info!("Credentials deleted from keyring");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                // Already deleted, that's fine
                Ok(())
            }
            Err(e) => {
                Err(AppError::Internal(format!("Failed to delete credentials: {}", e)))
            }
        }
    }

    /// Check if credentials exist
    pub fn has_credentials(&self) -> bool {
        self.entry.get_password().is_ok()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new().expect("Failed to create credential store")
    }
}
