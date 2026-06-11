use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use suiscope_core::{
    parse_publish_output, Registry, SuiScopeConfig, TrackedObject, Transaction, ObjectChange
};
use tabled::{settings::Style, Table, Tabled};
use colored::*;

use crate::output::{print_error, print_header, print_success, print_info};

#[derive(Tabled)]
struct PublishedRow {
    #[tabled(rename = "Alias")]
    alias: String,
    #[tabled(rename = "Object ID")]
    object_id: String,
    #[tabled(rename = "Type")]
    object_type: String,
    #[tabled(rename = "Owner")]
    owner: String,
}

pub async fn execute(path: PathBuf, gas_budget: Option<u64>) -> Result<()> {
    print_header("Publish Move Package");

    let config = SuiScopeConfig::load()?;
    let budget = gas_budget.unwrap_or(config.gas_budget);
    let sui_bin = config.sui_binary_path();

    print_info(&format!("Network: {}", config.network));
    print_info(&format!("Gas Budget: {} MIST", budget));
    print_info(&format!("Target path: {}", path.display()));
    println!();

    // 1. Run `sui client publish --json`
    let mut cmd = Command::new(sui_bin);
    cmd.arg("client")
        .arg("publish")
        .arg("--json")
        .arg("--gas-budget")
        .arg(budget.to_string())
        .arg(&path);

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| anyhow!("Failed to execute '{}'. Is it on your PATH? Error: {}", sui_bin, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If exit status is not success, it might be a compilation error or execution error
    if !output.status.success() {
        print_error("sui client publish failed");
        
        // Try to parse the stdout as JSON error if possible, otherwise print stderr
        if stdout.trim().starts_with('{') {
             match parse_publish_output(&stdout) {
                 Ok(res) if res.status != "success" => {
                      println!("{}", "Transaction reverted on-chain.".red());
                      println!("Run `suiscope explain \"<error>\"` to diagnose.");
                 }
                 _ => {}
             }
        }
        
        if !stderr.is_empty() {
            println!("\n{}", stderr);
        } else {
             println!("\n{}", stdout);
        }
        return Err(anyhow!("Publish failed."));
    }

    // 2. Parse the output
    if stdout.trim().is_empty() {
        return Err(anyhow!("Received empty output from sui client publish"));
    }

    let result = parse_publish_output(&stdout)
        .map_err(|e| anyhow!("Failed to parse JSON output: {}", e))?;

    if result.status != "success" {
        print_error("Publish transaction failed on-chain.");
        println!("Digest: {}", result.tx_digest);
        return Err(anyhow!("Transaction status: {}", result.status));
    }

    // 3. Save to registry
    let db_path = SuiScopeConfig::db_path()?;
    let registry = Registry::open(&db_path)?;
    let network_str = config.sui_network().as_str().to_string();

    let tx = Transaction {
        id: None,
        tx_digest: result.tx_digest.clone(),
        command: Some("publish".to_string()),
        status: result.status.clone(),
        gas_used: result.gas_used,
        gas_owner: None, // Sender would be parsed if needed
        package_id: result.package_id.clone(),
        module_name: None,
        function: None,
        raw_response: Some(result.raw_json),
        network: network_str.clone(),
        created_at: None,
    };
    registry.insert_transaction(&tx)?;

    let mut rows: Vec<PublishedRow> = Vec::new();

    for obj in &result.created_objects {
        // Auto-assign an alias based on type, if possible.
        let default_alias = auto_alias(obj);
        
        let tracked = TrackedObject {
            id: None,
            object_id: obj.object_id.clone(),
            object_type: Some(obj.object_type.clone()),
            alias: default_alias.clone(),
            owner: obj.owner.clone(),
            package_id: result.package_id.clone(),
            version: obj.version.clone(),
            digest: obj.digest.clone(),
            tx_digest: Some(result.tx_digest.clone()),
            network: network_str.clone(),
            created_at: None,
            updated_at: None,
        };

        registry.upsert_object(&tracked)?;

        rows.push(PublishedRow {
            alias: default_alias.unwrap_or_else(|| "-".to_string()),
            object_id: obj.object_id.clone(),
            object_type: obj.object_type.clone(),
            owner: obj.owner.clone().unwrap_or_else(|| "-".to_string()),
        });
    }

    // 4. Render output
    print_success("Package published and objects recorded.");
    println!("Transaction Digest: {}", result.tx_digest.bold());
    println!("Explorer: {}", config.sui_network().explorer_tx_url(&result.tx_digest).blue().underline());
    println!();
    
    if !rows.is_empty() {
        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{}", table);
    }

    Ok(())
}

/// Very simple heuristic to generate a default alias.
fn auto_alias(obj: &ObjectChange) -> Option<String> {
    if obj.change_type == "published" {
        return Some("package".to_string());
    }
    
    // Look for UpgradeCap
    if obj.object_type.contains("UpgradeCap") {
        return Some("upgrade_cap".to_string());
    }

    None
}
