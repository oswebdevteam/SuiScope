use anyhow::{anyhow, Result};
use suiscope_core::{Registry, SuiScopeConfig};

pub fn execute(alias: &str) -> Result<()> {
    let db_path = SuiScopeConfig::db_path()?;
    let registry = Registry::open(&db_path)?;

    if let Some(obj) = registry.get_by_alias(alias)? {
        // Output *only* the ID to stdout so it can be captured by scripts
        // e.g. `sui client call --package $(suiscope get my-pkg)`
        println!("{}", obj.object_id);
        Ok(())
    } else {
        // Errors go to stderr
        Err(anyhow!("Alias '{}' not found in registry.", alias))
    }
}
