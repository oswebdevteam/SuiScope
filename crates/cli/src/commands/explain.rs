use anyhow::Result;
use colored::*;
use suiscope_core::{ErrorDictionary, ErrorEntry, Registry, SuiScopeConfig};

use crate::output::{print_header, print_info, print_warning};

pub fn execute(error_string: &str) -> Result<()> {
    print_header("Error Explanation");
    
    // Log it quietly in the background if possible
    if let Ok(config) = SuiScopeConfig::load() {
        if let Ok(db_path) = SuiScopeConfig::db_path() {
            if let Ok(registry) = Registry::open(&db_path) {
                let entry = ErrorEntry {
                    id: None,
                    error_code: None,
                    error_message: error_string.to_string(),
                    module_id: None,
                    explanation: None, // We will update this if we find a match
                    tx_digest: None,
                    network: config.network,
                    created_at: None,
                };
                let _ = registry.insert_error(&entry);
            }
        }
    }

    if let Some(explanation) = ErrorDictionary::lookup(error_string) {
        println!("{}", explanation.title.bold().red());
        println!("\n{}", explanation.plain_english);
        
        println!("\n{}", "Likely Causes:".bold().yellow());
        for cause in explanation.likely_causes {
            println!("  {} {}", "•".yellow(), cause);
        }

        println!("\n{}", "Suggested Fixes:".bold().green());
        for fix in explanation.suggested_fixes {
            println!("  {} {}", "•".green(), fix);
        }
    } else {
        print_warning("No exact match found in the error dictionary.");
        println!("\nInput: {}", error_string);
        
        print_info("\nDebugging Tips:");
        println!("  • If this is a Move Abort, check the module source code for the exact abort code.");
        println!("  • If this is an RPC error, check your network configuration and object ownership.");
        println!("  • Try running `suiscope inspect <object_id>` to check object state.");
    }

    Ok(())
}
