use anyhow::{Context, Result};
use std::fs;
use crate::output::{print_info, print_success};
use suiscope_core::{SuiScopeConfig, WalrusClient, Registry};

pub async fn execute(import: Option<String>) -> Result<()> {
    // Load config
    let config = SuiScopeConfig::load().context("Failed to load SuiScope configuration")?;
    
    // Resolve registry path
    let db_path = SuiScopeConfig::db_path().context("Failed to get registry database path")?;
    
    // Initialize Walrus client
    let walrus_client = WalrusClient::new(&config.walrus_publisher, &config.walrus_aggregator);

    if let Some(blob_id) = import {
        print_info(&format!("Downloading registry database from Walrus (Blob ID: {})...", blob_id));
        
        let bytes = walrus_client.download_blob(&blob_id).await
            .context("Failed to download registry database from Walrus")?;
            
        print_info("Importing and merging downloaded registry into local registry...");
        
        // Write the downloaded bytes to a temporary database file
        let temp_db_path = db_path.parent()
            .map(|p| p.join("temp_import.db"))
            .unwrap_or_else(|| std::path::PathBuf::from("temp_import.db"));
            
        fs::write(&temp_db_path, bytes).context("Failed to write temporary import database file")?;
        
        // Load the local database (or initialize it if it doesn't exist)
        let registry = Registry::open(&db_path).context("Failed to open local registry database")?;
        
        // Merge!
        let merge_res = registry.merge_from_db_file(&temp_db_path);
        
        // Clean up the temporary file
        let _ = fs::remove_file(&temp_db_path);
        
        merge_res.context("Failed to merge the imported registry database")?;
        
        print_success("Registry imported and merged successfully!");
    } else {
        print_info("Uploading local registry database to Walrus...");
        
        if !db_path.exists() {
            return Err(anyhow::anyhow!("Local registry database does not exist. Run `suiscope publish` first."));
        }
        
        // Read raw database bytes
        let db_bytes = fs::read(&db_path).context("Failed to read local registry database file")?;
        
        let blob_id = walrus_client.upload_blob(&db_bytes).await
            .context("Failed to upload registry database to Walrus")?;
            
        print_success("Registry successfully uploaded to Walrus!");
        println!("\n  Blob ID: {}", blob_id);
        println!("  To import this registry, run:\n    suiscope sync --import {}", blob_id);
    }
    
    Ok(())
}
