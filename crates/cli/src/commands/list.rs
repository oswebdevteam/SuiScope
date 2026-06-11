use anyhow::Result;
use chrono::{DateTime, Utc};
use suiscope_core::{Registry, SuiScopeConfig};
use tabled::{settings::Style, Table, Tabled};

use crate::output::{print_header, print_info, truncate_id};

#[derive(Tabled)]
struct ListRow {
    #[tabled(rename = "Alias")]
    alias: String,
    #[tabled(rename = "Object ID")]
    object_id: String,
    #[tabled(rename = "Type")]
    object_type: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

pub fn execute(full_ids: bool) -> Result<()> {
    print_header("Tracked Objects");

    let config = SuiScopeConfig::load()?;
    let network = config.sui_network().as_str().to_string();
    
    let db_path = SuiScopeConfig::db_path()?;
    let registry = Registry::open(&db_path)?;

    let objects = registry.list_objects(&network)?;

    if objects.is_empty() {
        print_info(&format!("No objects tracked for network '{}'. Run `suiscope publish`.", network));
        return Ok(());
    }

    let mut rows = Vec::new();
    for obj in objects {
        let display_id = if full_ids {
            obj.object_id.clone()
        } else {
            truncate_id(&obj.object_id, 4)
        };

        // Format relative time if possible, otherwise use the raw SQLite string
        let display_time = if let Some(ref created) = obj.created_at {
            if let Ok(dt) = created.parse::<DateTime<Utc>>() {
                let now = Utc::now();
                let diff = now.signed_duration_since(dt);
                if diff.num_minutes() < 60 {
                    format!("{}m ago", diff.num_minutes())
                } else if diff.num_hours() < 24 {
                    format!("{}h ago", diff.num_hours())
                } else {
                    format!("{}d ago", diff.num_days())
                }
            } else {
                created.clone()
            }
        } else {
            "-".to_string()
        };

        rows.push(ListRow {
            alias: obj.alias.unwrap_or_else(|| "-".to_string()),
            object_id: display_id,
            object_type: obj.object_type.unwrap_or_else(|| "-".to_string()),
            created_at: display_time,
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    Ok(())
}
