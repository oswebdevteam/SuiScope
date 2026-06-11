pub mod config;
pub mod db;
pub mod error_dict;
pub mod errors;
pub mod parser;
pub mod rpc;
pub mod types;

// Re-export common types for easier access from CLI and Dashboard.
pub use config::SuiScopeConfig;
pub use db::Registry;
pub use error_dict::ErrorDictionary;
pub use errors::{Result, SuiScopeError};
pub use parser::parse_publish_output;
pub use rpc::SuiRpcClient;
pub use types::{
    ErrorEntry, ErrorExplanation, InspectResult, ObjectChange, PublishResult, SuiNetwork,
    TrackedObject, Transaction,
};
