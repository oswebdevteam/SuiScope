use anyhow::{anyhow, Result};
use suiscope_core::{Registry, SuiScopeConfig};
use colored::*;

use crate::output::{print_success, print_header};

pub fn execute(object_id: &str, alias: &str) -> Result<()> {
    // Basic validation of Object ID format
    if !object_id.starts_with("0x") || object_id.len() < 3 {
        return Err(anyhow!("Invalid Object ID format. Must start with '0x'."));
    }

    let db_path = SuiScopeConfig::db_path()?;
    let registry = Registry::open(&db_path)?;

    // Set the alias (returns false if the object was not found in the registry)
    if !registry.set_alias(object_id, alias)? {
        return Err(anyhow!("Object ID {} is not tracked in the registry. Publish it first or manually insert.", object_id));
    }

    print_header("Tag Assigned");
    print_success(&format!("Assigned alias '{}' to {}", alias.bold().green(), object_id));

    Ok(())
}
