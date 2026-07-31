use std::time::Duration;

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub bind_addr: String,
    pub upload_dir: String,
    pub session_ttl: Duration,
    pub enroll_code_ttl: Duration,
    pub cors_origin: String,
    pub sheets: SheetsConfig,
}

/// Mirroring the trail into a Google spreadsheet. Off unless a spreadsheet id
/// and a service account key are both configured.
#[derive(Debug, Clone)]
pub struct SheetsConfig {
    pub enabled: bool,
    pub spreadsheet_id: String,
    /// Path to the service account JSON downloaded from Google Cloud.
    pub key_path: String,
    pub every: Duration,
    /// Overridable so a test can point the export at a local stub instead of
    /// Google.
    pub api_base: String,
    pub token_url: String,
}

impl SheetsConfig {
    fn from_env() -> Self {
        let spreadsheet_id = optional("SHEETS_SPREADSHEET_ID", "");
        let key_path = optional("SHEETS_KEY_PATH", "");
        let asked = optional("SHEETS_ENABLED", "false").eq_ignore_ascii_case("true");
        Self {
            // Asking for it is not enough: without both of these every sync
            // would fail on the first request.
            enabled: asked && !spreadsheet_id.is_empty() && !key_path.is_empty(),
            spreadsheet_id,
            key_path,
            every: Duration::from_secs(optional_secs("SHEETS_SYNC_SECS", 300).max(30)),
            api_base: optional("SHEETS_API_BASE", "https://sheets.googleapis.com"),
            token_url: optional("SHEETS_TOKEN_URL", "https://oauth2.googleapis.com/token"),
        }
    }
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            jwt_secret: required("JWT_SECRET")?,
            bind_addr: optional("BIND_ADDR", "127.0.0.1:8080"),
            upload_dir: optional("UPLOAD_DIR", "./uploads"),
            session_ttl: Duration::from_secs(optional_secs("SESSION_TTL_SECS", 120)),
            enroll_code_ttl: Duration::from_secs(optional_secs("ENROLL_CODE_TTL_SECS", 300)),
            cors_origin: optional("CORS_ORIGIN", "http://localhost:5173"),
            sheets: SheetsConfig::from_env(),
        })
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("missing required env var {key}"))
}

fn optional(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn optional_secs(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
