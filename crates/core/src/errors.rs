use thiserror::Error;

/// Unified error type for the SuiScope core library.
#[derive(Error, Debug)]
pub enum SuiScopeError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("RPC request failed: {0}")]
    Rpc(#[from] reqwest::Error),

    #[error("RPC error response (code {code}): {message}")]
    RpcResponse { code: i64, message: String },

    #[error("Failed to parse output: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Sui CLI error: {0}")]
    SuiCli(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience alias used throughout the core crate.
pub type Result<T> = std::result::Result<T, SuiScopeError>;
