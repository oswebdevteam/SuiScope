use anyhow::{anyhow, Result};
use colored::*;
use suiscope_core::{Registry, SuiRpcClient, SuiScopeConfig};
use tabled::{settings::Style, Table, Tabled};

use crate::output::{print_error, print_header, print_info};

#[derive(Tabled)]
struct FieldRow {
    #[tabled(rename = "Field")]
    key: String,
    #[tabled(rename = "Value")]
    value: String,
}

pub async fn execute(id_or_alias: &str) -> Result<()> {
    print_header("Inspect Object State");

    let config = SuiScopeConfig::load()?;
    let rpc_url = config.rpc_url();
    let db_path = SuiScopeConfig::db_path()?;
    let registry = Registry::open(&db_path)?;

    print_info(&format!("RPC: {}", rpc_url));

    // 1. Resolve alias -> Object ID
    let object_id = if let Ok(Some(obj)) = registry.resolve(id_or_alias) {
        if let Some(alias) = obj.alias {
            println!("Resolved alias '{}' -> {}", alias.bold(), obj.object_id);
        }
        obj.object_id
    } else {
        // Fallback: maybe they passed a raw ID not in the registry
        id_or_alias.to_string()
    };

    if !object_id.starts_with("0x") {
        return Err(anyhow!("Invalid Object ID: {}", object_id));
    }

    // 2. Fetch from RPC
    println!("Fetching on-chain state...");
    let rpc_client = SuiRpcClient::new(&rpc_url);
    
    let state = match rpc_client.get_object(&object_id).await {
        Ok(s) => s,
        Err(e) => {
            print_error("Failed to fetch object.");
            return Err(anyhow!("{}", e));
        }
    };

    // 3. Render metadata
    println!();
    println!("{:<15} {}", "Object ID:", state.object_id.bold());
    println!("{:<15} {}", "Type:", state.object_type.as_deref().unwrap_or("unknown").cyan());
    println!("{:<15} {}", "Owner:", state.owner.as_deref().unwrap_or("unknown"));
    println!("{:<15} {}", "Version:", state.version.as_deref().unwrap_or("-"));
    
    if let Some(tx) = state.previous_transaction {
        let url = config.sui_network().explorer_tx_url(&tx);
        println!("{:<15} {} ({})", "Last Tx:", tx, url.underline().blue());
    }

    // 4. Render content fields if available
    println!();
    if let Some(content) = state.content {
        let mut rows = Vec::new();
        
        if let Some(obj_map) = content.as_object() {
            for (k, v) in obj_map {
                // Formatting JSON values to string
                let val_str = if v.is_string() {
                    v.as_str().unwrap().to_string()
                } else if v.is_object() || v.is_array() {
                    serde_json::to_string(v).unwrap_or_else(|_| "[complex object]".to_string())
                } else {
                    v.to_string()
                };

                rows.push(FieldRow {
                    key: k.clone(),
                    value: val_str,
                });
            }
        }

        if !rows.is_empty() {
            println!("{}", "Fields:".bold());
            let mut table = Table::new(rows);
            table.with(Style::rounded());
            println!("{}", table);
        } else {
            println!("No fields found (might be an empty struct).");
        }
    } else {
        println!("No content available (likely a package or unstructured object).");
    }

    Ok(())
}
