use serde::{Deserialize, Serialize};
use std::fmt;


/// Supported Sui networks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SuiNetwork {
    Mainnet,
    #[default]
    Testnet,
    Devnet,
    #[serde(untagged)]
    Custom(String),
}

impl SuiNetwork {
    /// Full-node JSON-RPC URL for this network.
    pub fn rpc_url(&self) -> &str {
        match self {
            Self::Mainnet => "https://fullnode.mainnet.sui.io:443",
            Self::Testnet => "https://fullnode.testnet.sui.io:443",
            Self::Devnet => "https://fullnode.devnet.sui.io:443",
            Self::Custom(url) => url.as_str(),
        }
    }

    /// Sui Explorer transaction URL.
    pub fn explorer_tx_url(&self, digest: &str) -> String {
        match self {
            Self::Mainnet => format!("https://suiscan.xyz/mainnet/tx/{digest}"),
            Self::Testnet => format!("https://suiscan.xyz/testnet/tx/{digest}"),
            Self::Devnet => format!("https://suiscan.xyz/devnet/tx/{digest}"),
            Self::Custom(_) => format!("(local) tx/{digest}"),
        }
    }

    /// Sui Explorer object URL.
    pub fn explorer_object_url(&self, object_id: &str) -> String {
        match self {
            Self::Mainnet => format!("https://suiscan.xyz/mainnet/object/{object_id}"),
            Self::Testnet => format!("https://suiscan.xyz/testnet/object/{object_id}"),
            Self::Devnet => format!("https://suiscan.xyz/devnet/object/{object_id}"),
            Self::Custom(_) => format!("(local) object/{object_id}"),
        }
    }

    /// Short string identifier.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Devnet => "devnet",
            Self::Custom(url) => url.as_str(),
        }
    }

    /// Parse from a string — accepts network names or full URLs.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mainnet" => Self::Mainnet,
            "testnet" => Self::Testnet,
            "devnet" => Self::Devnet,
            other => {
                if other.starts_with("http") {
                    Self::Custom(other.to_string())
                } else {
                    Self::Testnet // safe default
                }
            }
        }
    }
}

impl fmt::Display for SuiNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}




// Registry types (mirror SQLite rows)

/// A tracked on-chain object stored in the local registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedObject {
    pub id: Option<i64>,
    pub object_id: String,
    pub object_type: Option<String>,
    pub alias: Option<String>,
    pub owner: Option<String>,
    pub package_id: Option<String>,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub tx_digest: Option<String>,
    pub network: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// A recorded transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Option<i64>,
    pub tx_digest: String,
    pub command: Option<String>,
    pub status: String,
    pub gas_used: Option<i64>,
    pub gas_owner: Option<String>,
    pub package_id: Option<String>,
    pub module_name: Option<String>,
    pub function: Option<String>,
    pub raw_response: Option<String>,
    pub network: String,
    pub created_at: Option<String>,
}

/// A logged error for the explain command and dashboard panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    pub id: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: String,
    pub module_id: Option<String>,
    pub explanation: Option<String>,
    pub tx_digest: Option<String>,
    pub network: String,
    pub created_at: Option<String>,
}


// Publish output types

/// A single object change extracted from `sui client publish --json` output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectChange {
    pub object_id: String,
    pub object_type: String,
    /// Normalised owner: "AddressOwner(0x…)", "Shared", "Immutable", etc.
    pub owner: Option<String>,
    pub version: Option<String>,
    pub digest: Option<String>,
    /// "published", "created", "mutated", "deleted"
    pub change_type: String,
}

/// Full result of parsing a `sui client publish --json` invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub tx_digest: String,
    pub status: String,
    pub gas_used: Option<i64>,
    pub package_id: Option<String>,
    pub created_objects: Vec<ObjectChange>,
    pub raw_json: String,
}


// Inspect types

/// Structured result of inspecting an object via JSON-RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    pub object_id: String,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub object_type: Option<String>,
    pub owner: Option<String>,
    pub previous_transaction: Option<String>,
    pub storage_rebate: Option<String>,
    /// Raw Move struct fields as JSON — None for package objects.
    pub content: Option<serde_json::Value>,
}

// Error explanation types

/// A human-readable explanation for a cryptic error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorExplanation {
    pub title: String,
    pub plain_english: String,
    pub likely_causes: Vec<String>,
    pub suggested_fixes: Vec<String>,
}
