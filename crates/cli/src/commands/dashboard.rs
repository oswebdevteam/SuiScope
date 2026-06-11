use anyhow::Result;

use crate::output::print_info;

pub fn execute() -> Result<()> {
    print_info("Dashboard server not yet implemented.");
    // TODO (Engineer 3):
    // 1. Either spawn the axum binary here as a child process.
    // 2. Or compile the axum server directly into this command if preferred.
    Ok(())
}
