# SuiScope

**Move Contract Debug & Object Registry Tool for Sui Network**

Built with Rust ·

---

## The Problem

Every developer building on Sui faces the same friction after `sui client publish`:
- **Object ID chaos** — Package IDs and Object IDs scattered across terminals and text files
- **Cryptic errors** — `"invalid object reference"` with zero context
- **No state visibility** — No lightweight way to inspect object fields
- **Team coordination** — Teammates have no way to know new IDs without manual sharing

SuiScope closes this gap entirely.

## Quick Start

```bash
# Install globally from crates.io
cargo install suiscope-cli

# Initialize SuiScope in your Move project
suiscope init

# Publish a Move package — objects auto-registered
suiscope publish --gas-budget 100000000

# List all tracked objects
suiscope list

# Tag an object with a human-readable alias
suiscope tag 0xabcd...1234 my-package

# Resolve alias to Object ID (scriptable)
suiscope get my-package

# Inspect an object's on-chain state
suiscope inspect my-package

# Explain a cryptic error in plain English
suiscope explain "invalid object reference"

# Launch the web dashboard
suiscope dashboard
```

## Architecture

```
suiscope/
├── crates/
│   ├── core/       →  Shared engine: SQLite registry, Sui RPC client, parser, error dictionary
│   ├── cli/        →  Terminal interface: clap-based CLI with publish interceptor
│   └── dashboard/  →  Web UI: Axum server + static frontend
├── Cargo.toml      →  Workspace manifest
└── README.md
```

| Crate | Name | Responsibility |
|-------|------|----------------|
| `core` | Shared Engine | SQLite registry, Sui RPC client, object parser, error dictionary |
| `cli` | Terminal Interface | clap-based CLI: publish wrapper, registry commands, inspect, explain |
| `dashboard` | Web UI | Axum server + static frontend, REST API over the same SQLite |

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| CLI | `clap` | Argument parsing and subcommand routing |
| CLI | `colored` + `tabled` | Professional terminal output |
| Core | `reqwest` + `serde_json` | Sui JSON-RPC calls and response parsing |
| Core | `rusqlite` + SQLite | Local object ID registry, zero external daemons |
| Dashboard | `axum` | Lightweight REST API server |
| Sync | Walrus | Cross-machine registry sync via decentralized storage |
| Chain | Sui Testnet | Live deployment and demo environment |

## Installation

The easiest way to install SuiScope is via crates.io:

```bash
cargo install suiscope-cli
```

## Building from Source

If you want to build from source or contribute to the project:

```bash
# Build all crates
cargo build --workspace

# Run with clippy checks
cargo clippy --workspace -- -D warnings
```

## Configuration

SuiScope stores project-local data in `.suiscope/`:

```
your-move-project/
├── .suiscope/
│   ├── config.toml    # Network, gas budget, Sui binary path
│   └── registry.db    # SQLite object registry
├── sources/
├── Move.toml
└── ...
```

### Default Config (`config.toml`)

```toml
network = "testnet"
gas_budget = 100000000
```

## License

MIT

---

**SuiScope · Built with intention. Shipped in Rust.**
