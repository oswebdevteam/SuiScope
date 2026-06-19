use crate::errors::{Result, SuiScopeError};
use serde_json::Value;

/// Client to interact with the Walrus decentralized storage network.
pub struct WalrusClient {
    client: reqwest::Client,
    publisher_url: String,
    aggregator_url: String,
}

impl WalrusClient {
    /// Create a new WalrusClient.
    pub fn new(publisher_url: &str, aggregator_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            publisher_url: publisher_url.trim_end_matches('/').to_string(),
            aggregator_url: aggregator_url.trim_end_matches('/').to_string(),
        }
    }

    /// Upload a raw data blob to the Walrus publisher. Returns the blob ID.
    pub async fn upload_blob(&self, data: &[u8]) -> Result<String> {
        let url = format!("{}/v1/blobs?epochs=5", self.publisher_url);
        
        let resp = self
            .client
            .put(&url)
            .body(data.to_vec())
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SuiScopeError::Parse(format!(
                "Walrus upload failed with status {status}: {text}"
            )));
        }

        let body_json: Value = resp.json().await?;
        
        if let Some(already_certified) = body_json.get("alreadyCertified") {
            if let Some(blob_id) = already_certified.get("blobId").and_then(|v| v.as_str()) {
                return Ok(blob_id.to_string());
            }
        }
        
        if let Some(newly_created) = body_json.get("newlyCreated") {
            if let Some(blob_id) = newly_created
                .get("blobObject")
                .and_then(|o| o.get("blobId"))
                .and_then(|v| v.as_str())
            {
                return Ok(blob_id.to_string());
            }
        }

        Err(SuiScopeError::Parse(format!(
            "Unexpected Walrus response body: {body_json:?}"
        )))
    }

    /// Download a raw data blob from the Walrus aggregator.
    pub async fn download_blob(&self, blob_id: &str) -> Result<Vec<u8>> {
        let url = format!("{}/v1/blobs/{}", self.aggregator_url, blob_id);
        
        let resp = self
            .client
            .get(&url)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SuiScopeError::Parse(format!(
                "Walrus download failed with status {status}: {text}"
            )));
        }

        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_newly_created_response() {
        let raw_json = r#"{
            "newlyCreated": {
                "blobObject": {
                    "blobId": "M4hsZGQ1oCktdzegB6HnI6Mi28S2nqOPHxK-W7_4BUk",
                    "id": "0x123",
                    "storedEpoch": 150,
                    "registeredEpoch": 34
                },
                "cost": 1000
            }
        }"#;

        let body_json: Value = serde_json::from_str(raw_json).unwrap();
        let blob_id = body_json
            .get("newlyCreated")
            .and_then(|n| n.get("blobObject"))
            .and_then(|o| o.get("blobId"))
            .and_then(|v| v.as_str())
            .unwrap();

        assert_eq!(blob_id, "M4hsZGQ1oCktdzegB6HnI6Mi28S2nqOPHxK-W7_4BUk");
    }

    #[test]
    fn test_parse_already_certified_response() {
        let raw_json = r#"{
            "alreadyCertified": {
                "blobId": "M4hsZGQ1oCktdzegB6HnI6Mi28S2nqOPHxK-W7_4BUk",
                "event": {
                    "txDigest": "0x456"
                },
                "endEpoch": 12345
            }
        }"#;

        let body_json: Value = serde_json::from_str(raw_json).unwrap();
        let blob_id = body_json
            .get("alreadyCertified")
            .and_then(|a| a.get("blobId"))
            .and_then(|v| v.as_str())
            .unwrap();

        assert_eq!(blob_id, "M4hsZGQ1oCktdzegB6HnI6Mi28S2nqOPHxK-W7_4BUk");
    }
}
