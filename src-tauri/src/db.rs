// AIDEV-NOTE: SQLite database for persisting user data (credentials, settings)
// In dev mode, stores in local-db/plgui.db; in prod uses app data directory

use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info};

use crate::auth::ApiCredentials;
use crate::error::AppError;

/// Database manager for SQLite persistence
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Initialize database with automatic path selection
    /// Dev: local-db/plgui.db
    /// Prod: OS app data directory
    pub fn new() -> Result<Self, AppError> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("Failed to create db directory: {}", e)))?;
        }

        info!("Opening database at: {:?}", db_path);

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        // Initialize schema
        db.init_schema()?;

        Ok(db)
    }

    /// Get the database path based on environment
    fn get_db_path() -> Result<PathBuf, AppError> {
        // Check if we're in dev mode (local-db directory exists or we're in src-tauri)
        let local_db = PathBuf::from("local-db");
        let src_tauri_local = PathBuf::from("src-tauri/local-db");

        // Try local-db first (when running from src-tauri)
        if local_db.exists() || std::env::var("TAURI_DEV").is_ok() {
            std::fs::create_dir_all(&local_db)
                .map_err(|e| AppError::Internal(format!("Failed to create local-db: {}", e)))?;
            return Ok(local_db.join("plgui.db"));
        }

        // Try src-tauri/local-db (when running from project root)
        if src_tauri_local.exists() {
            return Ok(src_tauri_local.join("plgui.db"));
        }

        // Check for dev build
        #[cfg(debug_assertions)]
        {
            std::fs::create_dir_all(&local_db)
                .map_err(|e| AppError::Internal(format!("Failed to create local-db: {}", e)))?;
            return Ok(local_db.join("plgui.db"));
        }

        // Production: use app data directory
        #[cfg(not(debug_assertions))]
        {
            let proj_dirs = directories::ProjectDirs::from("com", "rileytg", "plgui")
                .ok_or_else(|| AppError::Internal("Could not find app data directory".to_string()))?;

            let data_dir = proj_dirs.data_dir();
            std::fs::create_dir_all(data_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create data dir: {}", e)))?;

            Ok(data_dir.join("plgui.db"))
        }
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            -- User credentials table
            CREATE TABLE IF NOT EXISTS credentials (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                api_key TEXT NOT NULL,
                api_secret TEXT NOT NULL,
                api_passphrase TEXT NOT NULL,
                address TEXT NOT NULL,
                polymarket_address TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- User settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- AIDEV-NOTE: Price history cache - stores historical price data per token
            -- token_id is the CLOB token ID (long numeric string)
            -- timestamp is Unix epoch seconds, price is 0.0-1.0
            CREATE TABLE IF NOT EXISTS price_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                price REAL NOT NULL,
                fetched_at TEXT DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(token_id, timestamp)
            );

            -- Index for efficient queries by token
            CREATE INDEX IF NOT EXISTS idx_price_history_token_time
                ON price_history(token_id, timestamp DESC);
            "#,
        )
        .map_err(|e| AppError::Internal(format!("Failed to init schema: {}", e)))?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Store credentials (replaces existing)
    pub fn store_credentials(&self, creds: &ApiCredentials, polymarket_address: Option<&str>) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO credentials
                (id, api_key, api_secret, api_passphrase, address, polymarket_address, updated_at)
            VALUES (1, ?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
            "#,
            (
                &creds.api_key,
                &creds.api_secret,
                &creds.api_passphrase,
                &creds.address,
                polymarket_address,
            ),
        )
        .map_err(|e| AppError::Internal(format!("Failed to store credentials: {}", e)))?;

        info!("Credentials stored in database");
        Ok(())
    }

    /// Load credentials
    pub fn load_credentials(&self) -> Result<Option<(ApiCredentials, Option<String>)>, AppError> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT api_key, api_secret, api_passphrase, address, polymarket_address FROM credentials WHERE id = 1",
            [],
            |row| {
                Ok((
                    ApiCredentials {
                        api_key: row.get(0)?,
                        api_secret: row.get(1)?,
                        api_passphrase: row.get(2)?,
                        address: row.get(3)?,
                    },
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        );

        match result {
            Ok((creds, polymarket_addr)) => {
                debug!("Credentials loaded from database");
                Ok(Some((creds, polymarket_addr)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                debug!("No credentials found in database");
                Ok(None)
            }
            Err(e) => Err(AppError::Internal(format!("Failed to load credentials: {}", e))),
        }
    }

    /// Delete credentials
    pub fn delete_credentials(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM credentials WHERE id = 1", [])
            .map_err(|e| AppError::Internal(format!("Failed to delete credentials: {}", e)))?;

        info!("Credentials deleted from database");
        Ok(())
    }

    /// Update Polymarket address
    pub fn update_polymarket_address(&self, address: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE credentials SET polymarket_address = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            [address],
        )
        .map_err(|e| AppError::Internal(format!("Failed to update polymarket address: {}", e)))?;

        debug!("Polymarket address updated");
        Ok(())
    }

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Internal(format!("Failed to get setting: {}", e))),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            [key, value],
        )
        .map_err(|e| AppError::Internal(format!("Failed to set setting: {}", e)))?;

        Ok(())
    }

    // ========== Price History Methods ==========

    /// Store price history points for a token (upserts to avoid duplicates)
    pub fn store_price_history(&self, token_id: &str, points: &[(i64, f64)]) -> Result<usize, AppError> {
        let conn = self.conn.lock().unwrap();

        let mut inserted = 0;
        for (timestamp, price) in points {
            let result = conn.execute(
                "INSERT OR IGNORE INTO price_history (token_id, timestamp, price) VALUES (?1, ?2, ?3)",
                (token_id, timestamp, price),
            );

            if let Ok(count) = result {
                inserted += count;
            }
        }

        debug!("Stored {} new price history points for {}", inserted, token_id);
        Ok(inserted)
    }

    /// Get cached price history for a token within a time range
    /// Returns Vec<(timestamp, price)> sorted by timestamp ascending
    pub fn get_price_history(
        &self,
        token_id: &str,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<Vec<(i64, f64)>, AppError> {
        let conn = self.conn.lock().unwrap();

        let mut sql = "SELECT timestamp, price FROM price_history WHERE token_id = ?1".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(token_id.to_string())];

        if let Some(start) = start_ts {
            sql.push_str(" AND timestamp >= ?2");
            params.push(Box::new(start));
        }

        if let Some(end) = end_ts {
            let param_num = params.len() + 1;
            sql.push_str(&format!(" AND timestamp <= ?{}", param_num));
            params.push(Box::new(end));
        }

        sql.push_str(" ORDER BY timestamp ASC");

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
            })
            .map_err(|e| AppError::Internal(format!("Failed to query price history: {}", e)))?;

        let mut result = Vec::new();
        for row in rows {
            if let Ok(point) = row {
                result.push(point);
            }
        }

        debug!("Retrieved {} price history points for {}", result.len(), token_id);
        Ok(result)
    }

    /// Get the most recent cached timestamp for a token (to know where to resume fetching)
    pub fn get_latest_price_timestamp(&self, token_id: &str) -> Result<Option<i64>, AppError> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT MAX(timestamp) FROM price_history WHERE token_id = ?1",
            [token_id],
            |row| row.get::<_, Option<i64>>(0),
        );

        match result {
            Ok(ts) => Ok(ts),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Internal(format!("Failed to get latest timestamp: {}", e))),
        }
    }

    /// Clear old price history (older than specified days)
    #[allow(dead_code)]
    pub fn cleanup_old_price_history(&self, days: i64) -> Result<usize, AppError> {
        let conn = self.conn.lock().unwrap();

        let cutoff = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);

        let deleted = conn
            .execute(
                "DELETE FROM price_history WHERE timestamp < ?1",
                [cutoff],
            )
            .map_err(|e| AppError::Internal(format!("Failed to cleanup price history: {}", e)))?;

        info!("Cleaned up {} old price history records", deleted);
        Ok(deleted)
    }
}
