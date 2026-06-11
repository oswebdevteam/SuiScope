use crate::errors::{Result, SuiScopeError};
use crate::types::SuiNetwork;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = ".suiscope";
const CONFIG_FILE: &str = "config.toml";
const DB_FILE: &str = "registry.db";

/// Per-project SuiScope configuration, stored in `.suiscope/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiScopeConfig {
    /// Network name: "testnet", "mainnet", "devnet", or a full URL.
    #[serde(default = "default_network")]
    pub network: String,

    /// Override RPC URL (takes precedence over `network`).
    #[serde(default)]
    pub rpc_url: Option<String>,

    /// Default gas budget in MIST for publish and other transactions.
    #[serde(default = "default_gas_budget")]
    pub gas_budget: u64,

    /// Path to the `sui` binary. Defaults to `"sui"` (assumes it's on PATH).
    #[serde(default)]
    pub sui_binary: Option<String>,
}

fn default_network() -> String {
    "testnet".to_string()
}
fn default_gas_budget() -> u64 {
    100_000_000
}

impl Default for SuiScopeConfig {
    fn default() -> Self {
        Self {
            network: default_network(),
            rpc_url: None,
            gas_budget: default_gas_budget(),
            sui_binary: None,
        }
    }
}

impl SuiScopeConfig {
    /// Resolve the configured network.
    pub fn sui_network(&self) -> SuiNetwork {
        if let Some(ref url) = self.rpc_url {
            SuiNetwork::Custom(url.clone())
        } else {
            SuiNetwork::parse(&self.network)
        }
    }

    /// Effective RPC URL (explicit override or derived from network).
    pub fn rpc_url(&self) -> String {
        if let Some(ref url) = self.rpc_url {
            url.clone()
        } else {
            self.sui_network().rpc_url().to_string()
        }
    }

    /// Path to the `sui` binary.
    pub fn sui_binary_path(&self) -> &str {
        self.sui_binary.as_deref().unwrap_or("sui")
    }

    /// Walk up from `cwd` to find the nearest `.suiscope/` directory.
    pub fn find_project_dir() -> Result<PathBuf> {
        let mut dir =
            std::env::current_dir().map_err(|e| SuiScopeError::Config(e.to_string()))?;
        loop {
            let candidate = dir.join(CONFIG_DIR);
            if candidate.exists() && candidate.is_dir() {
                return Ok(candidate);
            }
            if !dir.pop() {
                return Err(SuiScopeError::Config(
                    "No .suiscope directory found. Run `suiscope init` first.".to_string(),
                ));
            }
        }
    }

    /// Create `.suiscope/` in `base` and write a default `config.toml`.
    pub fn init(base: &Path) -> Result<PathBuf> {
        let scope_dir = base.join(CONFIG_DIR);
        std::fs::create_dir_all(&scope_dir)?;

        let config_path = scope_dir.join(CONFIG_FILE);
        if !config_path.exists() {
            let default_cfg = Self::default();
            let toml_str = toml::to_string_pretty(&default_cfg)
                .map_err(|e| SuiScopeError::Config(e.to_string()))?;
            std::fs::write(&config_path, toml_str)?;
        }

        Ok(scope_dir)
    }

    /// Load config from the nearest `.suiscope/config.toml`.
    pub fn load() -> Result<Self> {
        let scope_dir = Self::find_project_dir()?;
        let config_path = scope_dir.join(CONFIG_FILE);

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(|e| SuiScopeError::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// Get the path to the SQLite registry database.
    pub fn db_path() -> Result<PathBuf> {
        let scope_dir = Self::find_project_dir()?;
        Ok(scope_dir.join(DB_FILE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config_serialization() {
        let cfg = SuiScopeConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        assert!(toml_str.contains("testnet"));
        assert!(toml_str.contains("100000000"));

        // Round-trip
        let parsed: SuiScopeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.network, "testnet");
        assert_eq!(parsed.gas_budget, 100_000_000);
    }

    #[test]
    fn test_init_creates_directory() {
        let tmp = std::env::temp_dir().join("suiscope_test_init");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let scope_dir = SuiScopeConfig::init(&tmp).unwrap();
        assert!(scope_dir.exists());
        assert!(scope_dir.join("config.toml").exists());

        let _ = fs::remove_dir_all(&tmp);
    }
}
