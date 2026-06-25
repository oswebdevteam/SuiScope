use anyhow::Result;

use crate::output::print_info;

const DEFAULT_PORT: u16 = 7731;

pub async fn execute() -> Result<()> {
    print_info(&format!(
        "Starting dashboard on http://127.0.0.1:{DEFAULT_PORT} — press Ctrl-C to stop."
    ));
    suiscope_dashboard::start(DEFAULT_PORT).await
}
