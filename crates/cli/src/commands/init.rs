use anyhow::Result;
use suiscope_core::SuiScopeConfig;
use std::env;

use crate::output::{print_header, print_success, print_info};

pub fn execute() -> Result<()> {
    print_header("Initialize SuiScope");

    let cwd = env::current_dir()?;
    let scope_dir = SuiScopeConfig::init(&cwd)?;
    
    print_info(&format!("Created directory: {}", scope_dir.display()));
    print_success("SuiScope initialized successfully. Configuration is ready.");
    
    Ok(())
}
