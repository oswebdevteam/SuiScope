use crate::errors::{Result, SuiScopeError};
use crate::types::InspectResult;
use serde_json::{json, Value};

/// Async client for Sui JSON-RPC.
///
/// **Note:** Sui JSON-RPC is deprecated (EOL July 2026). This client is designed
/// to be swappable — all public methods return domain types, not raw JSON.
/// A future GraphQL/gRPC implementation can implement the same signatures.
pub struct SuiRpcClient {
    client: reqwest::Client,
    rpc_url: String,
}

impl SuiRpcClient {
    /// Create a new RPC client pointing at `rpc_url`.
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            rpc_url: rpc_url.to_string(),
        }
    }

    /// Generic JSON-RPC 2.0 caller.
    async fn call_rpc(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        tracing::debug!(%method, "RPC request");

        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        // Check for JSON-RPC error
        if let Some(error) = resp.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown RPC error")
                .to_string();
            return Err(SuiScopeError::RpcResponse { code, message });
        }

        resp.get("result")
            .cloned()
            .ok_or_else(|| SuiScopeError::Parse("RPC response missing 'result' field".into()))
    }

    /// Fetch an object's current state from the chain.
    pub async fn get_object(&self, object_id: &str) -> Result<InspectResult> {
        let params = json!([
            object_id,
            {
                "showType": true,
                "showOwner": true,
                "showContent": true,
                "showPreviousTransaction": true,
                "showStorageRebate": true,
            }
        ]);

        let result = self.call_rpc("sui_getObject", params).await?;

        // Check if the object exists
        if let Some(err) = result.get("error") {
            let code = err
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown");
            return Err(SuiScopeError::NotFound(format!(
                "Object {object_id} not found: {code}"
            )));
        }

        let data = result.get("data").ok_or_else(|| {
            SuiScopeError::NotFound(format!("Object {object_id} — no data in response"))
        })?;

        let owner = Self::normalize_owner(data.get("owner"));

        let content = data
            .get("content")
            .and_then(|c| c.get("fields"))
            .cloned();

        Ok(InspectResult {
            object_id: data
                .get("objectId")
                .and_then(|v| v.as_str())
                .unwrap_or(object_id)
                .to_string(),
            version: data.get("version").and_then(|v| v.as_str()).map(String::from),
            digest: data.get("digest").and_then(|v| v.as_str()).map(String::from),
            object_type: data
                .get("type")
                .and_then(|v| v.as_str())
                .map(String::from),
            owner,
            previous_transaction: data
                .get("previousTransaction")
                .and_then(|v| v.as_str())
                .map(String::from),
            storage_rebate: data
                .get("storageRebate")
                .and_then(|v| v.as_str())
                .map(String::from),
            content,
        })
    }

    /// Fetch a transaction block by digest.
    pub async fn get_transaction(&self, digest: &str) -> Result<Value> {
        let params = json!([
            digest,
            {
                "showInput": true,
                "showEffects": true,
                "showEvents": true,
                "showObjectChanges": true,
            }
        ]);
        self.call_rpc("sui_getTransactionBlock", params).await
    }

    /// Normalise the `owner` field from Sui RPC into a readable string.
    fn normalize_owner(owner: Option<&Value>) -> Option<String> {
        let owner = owner?;
        if let Some(addr) = owner.get("AddressOwner").and_then(|v| v.as_str()) {
            Some(format!("AddressOwner({addr})"))
        } else if let Some(obj) = owner.get("ObjectOwner").and_then(|v| v.as_str()) {
            Some(format!("ObjectOwner({obj})"))
        } else if owner.get("Shared").is_some() {
            Some("Shared".to_string())
        } else if owner.as_str() == Some("Immutable") {
            Some("Immutable".to_string())
        } else {
            Some(owner.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_owner_address() {
        let val = json!({"AddressOwner": "0xabc"});
        assert_eq!(
            SuiRpcClient::normalize_owner(Some(&val)),
            Some("AddressOwner(0xabc)".to_string())
        );
    }

    #[test]
    fn test_normalize_owner_shared() {
        let val = json!({"Shared": {"initial_shared_version": 1}});
        assert_eq!(
            SuiRpcClient::normalize_owner(Some(&val)),
            Some("Shared".to_string())
        );
    }

    #[test]
    fn test_normalize_owner_immutable() {
        let val = json!("Immutable");
        assert_eq!(
            SuiRpcClient::normalize_owner(Some(&val)),
            Some("Immutable".to_string())
        );
    }
}
