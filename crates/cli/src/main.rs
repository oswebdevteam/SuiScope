use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod output;

use commands::*;

#[derive(Parser)]
#[command(
    name = "suiscope",
    about = "Move Contract Debug & Object Registry Tool for Sui Network",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize SuiScope in the current directory
    Init,
    /// Wrap `sui client publish` and track objects
    Publish {
        /// Path to the Move package (defaults to current directory)
        #[arg(long, short, default_value = ".")]
        path: PathBuf,
        /// Gas budget in MIST (overrides config)
        #[arg(long)]
        gas_budget: Option<u64>,
    },
    /// List all tracked objects for the current network
    List {
        /// Show full Object IDs (do not truncate)
        #[arg(long)]
        full: bool,
    },
    /// Resolve an alias to its Object ID
    Get {
        /// The alias to look up
        alias: String,
    },
    /// Assign a human-readable alias to an Object ID
    Tag {
        /// The 64-character Object ID
        object_id: String,
        /// The alias to assign
        alias: String,
    },
    /// Inspect the on-chain state of an object
    Inspect {
        /// The Object ID or alias to inspect
        id_or_alias: String,
    },
    /// Explain a Move abort code or cryptic RPC error
    Explain {
        /// The error string or code to explain
        error: String,
    },
    /// Sync your local registry to Walrus
    Sync,
    /// Launch the SuiScope web dashboard
    Dashboard,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init::execute()?,
        Commands::Publish { path, gas_budget } => publish::execute(path, gas_budget).await?,
        Commands::List { full } => list::execute(full)?,
        Commands::Get { alias } => get::execute(&alias)?,
        Commands::Tag { object_id, alias } => tag::execute(&object_id, &alias)?,
        Commands::Inspect { id_or_alias } => inspect::execute(&id_or_alias).await?,
        Commands::Explain { error } => explain::execute(&error)?,
        Commands::Sync => sync::execute()?,
        Commands::Dashboard => dashboard::execute()?,
    }

    Ok(())
}
