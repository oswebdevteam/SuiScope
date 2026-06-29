use crate::errors::{Result, SuiScopeError};
use crate::types::{ObjectChange, PublishResult};
use serde_json::Value;

/// Parse the JSON output from `sui client publish --json`.
///
/// The expected structure contains:
/// - `digest` — transaction digest
/// - `effects.status.status` — "success" or "failure"
/// - `effects.gasUsed` — computation/storage costs
/// - `objectChanges[]` — array with "published" and "created" entries
pub fn parse_publish_output(json_str: &str) -> Result<PublishResult> {
    let root: Value =
        serde_json::from_str(json_str).map_err(|e| SuiScopeError::Parse(format!(
            "Invalid JSON from sui client publish: {e}"
        )))?;

    // Transaction digest
    let tx_digest = root
        .get("digest")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SuiScopeError::Parse("Missing 'digest' in publish output".into()))?
        .to_string();

    // Status
    let status = root
        .pointer("/effects/status/status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Gas used (sum computation + storage − rebate)
    let gas_used = parse_gas_used(&root);

    // Object changes
    let mut package_id: Option<String> = None;
    let mut created_objects: Vec<ObjectChange> = Vec::new();

    if let Some(changes) = root.get("objectChanges").and_then(|v| v.as_array()) {
        for change in changes {
            let change_type = change
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            match change_type {
                "published" => {
                    let pkg_id = change
                        .get("packageId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    package_id = Some(pkg_id.clone());

                    // Build the list of published modules for display
                    let modules = change
                        .get("modules")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| m.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();

                    created_objects.push(ObjectChange {
                        object_id: pkg_id,
                        object_type: format!("package [{}]", modules),
                        owner: Some("Immutable".to_string()),
                        version: change.get("version").and_then(|v| v.as_str()).map(String::from),
                        digest: change.get("digest").and_then(|v| v.as_str()).map(String::from),
                        change_type: "published".to_string(),
                    });
                }
                "created" => {
                    let obj_id = change
                        .get("objectId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let obj_type = change
                        .get("objectType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let owner = normalize_change_owner(change.get("owner"));

                    created_objects.push(ObjectChange {
                        object_id: obj_id,
                        object_type: obj_type,
                        owner,
                        version: change.get("version").and_then(|v| v.as_str()).map(String::from),
                        digest: change.get("digest").and_then(|v| v.as_str()).map(String::from),
                        change_type: "created".to_string(),
                    });
                }
                "mutated" => {
                    let obj_id = change
                        .get("objectId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let obj_type = change
                        .get("objectType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let owner = normalize_change_owner(change.get("owner"));

                    created_objects.push(ObjectChange {
                        object_id: obj_id,
                        object_type: obj_type,
                        owner,
                        version: change.get("version").and_then(|v| v.as_str()).map(String::from),
                        digest: change.get("digest").and_then(|v| v.as_str()).map(String::from),
                        change_type: "mutated".to_string(),
                    });
                }
                _ => {
                    tracing::debug!(change_type, "Skipping unknown object change type");
                }
            }
        }
    }

    Ok(PublishResult {
        tx_digest,
        status,
        gas_used,
        package_id,
        created_objects,
        raw_json: json_str.to_string(),
    })
}

/// Sum computation cost + storage cost − storage rebate.
fn parse_gas_used(root: &Value) -> Option<i64> {
    let gas = root.pointer("/effects/gasUsed")?;

    let computation: i64 = gas
        .get("computationCost")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let storage: i64 = gas
        .get("storageCost")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let rebate: i64 = gas
        .get("storageRebate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Some(computation + storage - rebate)
}

/// Normalise the owner object from an `objectChanges` entry.
fn normalize_change_owner(owner: Option<&Value>) -> Option<String> {
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


// Tests

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture: realistic `sui client publish --json` output.
    const SAMPLE_PUBLISH_OUTPUT: &str = r#"{
        "digest": "8dKcGBDYHEzBqxNmHRFY9LTJqVdP3vW5X1234567890a",
        "effects": {
            "status": { "status": "success" },
            "gasUsed": {
                "computationCost": "1000000",
                "storageCost": "5000000",
                "storageRebate": "900000",
                "nonRefundableStorageFee": "0"
            }
        },
        "objectChanges": [
            {
                "type": "published",
                "packageId": "0xabc123def456abc123def456abc123def456abc123def456abc123def456abc1",
                "version": "1",
                "digest": "pkg_digest_001",
                "modules": ["counter", "admin"]
            },
            {
                "type": "created",
                "sender": "0xsender001",
                "owner": { "AddressOwner": "0xsender001" },
                "objectType": "0xabc123::counter::Counter",
                "objectId": "0xobj_001_created_by_init",
                "version": "1",
                "digest": "obj_digest_001"
            },
            {
                "type": "created",
                "sender": "0xsender001",
                "owner": { "AddressOwner": "0xsender001" },
                "objectType": "0x2::package::UpgradeCap",
                "objectId": "0xobj_002_upgrade_cap",
                "version": "1",
                "digest": "obj_digest_002"
            },
            {
                "type": "mutated",
                "sender": "0xsender001",
                "owner": { "AddressOwner": "0xsender001" },
                "objectType": "0x2::coin::Coin<0x2::sui::SUI>",
                "objectId": "0xgas_coin",
                "version": "2",
                "digest": "gas_digest"
            }
        ],
        "confirmedLocalExecution": true
    }"#;

    #[test]
    fn test_parse_publish_output_success() {
        let result = parse_publish_output(SAMPLE_PUBLISH_OUTPUT).unwrap();

        assert_eq!(result.tx_digest, "8dKcGBDYHEzBqxNmHRFY9LTJqVdP3vW5X1234567890a");
        assert_eq!(result.status, "success");

        // Gas: 1_000_000 + 5_000_000 - 900_000 = 5_100_000
        assert_eq!(result.gas_used, Some(5_100_000));

        assert_eq!(
            result.package_id.as_deref(),
            Some("0xabc123def456abc123def456abc123def456abc123def456abc123def456abc1")
        );

        // 1 published + 2 created + 1 mutated = 4
        assert_eq!(result.created_objects.len(), 4);

        // First entry is the published package
        assert_eq!(result.created_objects[0].change_type, "published");
        assert!(result.created_objects[0].object_type.contains("counter"));
        assert!(result.created_objects[0].object_type.contains("admin"));

        // Second entry is the Counter object
        assert_eq!(result.created_objects[1].change_type, "created");
        assert!(result.created_objects[1].object_type.contains("Counter"));
    }

    #[test]
    fn test_parse_publish_output_invalid_json() {
        let result = parse_publish_output("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_publish_output_missing_digest() {
        let result = parse_publish_output("{}");
        assert!(result.is_err());
    }
}
