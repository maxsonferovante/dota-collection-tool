//! `logs`: query recorded happenings with filters, once or following.
//!
//! Reads come from the store; operational notes go to stderr. The auth
//! token never appears here — happenings carry none.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;

use crate::cli::LogsArgs;
use crate::paths;
use dct_store::{HappeningFilter, Store, StoredHappening};

/// One text line: tick, kind, actor and compact detail.
pub fn format_text(row: &StoredHappening) -> String {
    let actor = row.actor.as_deref().unwrap_or("-");
    format!("#{} {} {actor} {}", row.tick, row.kind, row.detail)
}

/// One JSON line carrying the full row.
pub fn format_json(row: &StoredHappening) -> String {
    serde_json::json!({
        "id": row.id,
        "match_id": row.match_id,
        "tick": row.tick,
        "kind": row.kind,
        "actor": row.actor,
        "detail": row.detail,
    })
    .to_string()
}

fn filter_from(args: &LogsArgs, after_id: Option<i64>) -> HappeningFilter {
    HappeningFilter {
        match_id: args.match_id.clone(),
        kind: args.kind.clone(),
        limit: args.limit.max(1),
        after_id,
    }
}

async fn print_batch(store: &Store, filter: &HappeningFilter, as_json: bool) -> Result<i64> {
    let rows = store
        .query_happenings(filter)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let mut last_id = filter.after_id.unwrap_or(0);
    for row in &rows {
        if as_json {
            println!("{}", format_json(row));
        } else {
            println!("{}", format_text(row));
        }
        last_id = last_id.max(row.id);
    }
    Ok(last_id)
}

/// Run `logs`: dump matching happenings, then optionally follow new ones
/// until Ctrl-C.
pub async fn run(args: &LogsArgs, db_override: Option<PathBuf>) -> Result<()> {
    let paths = paths::resolve(db_override)?;
    let store = Store::open(&paths.db)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let mut last_id = print_batch(&store, &filter_from(args, None), args.json).await?;
    if !args.follow {
        return Ok(());
    }
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                last_id = print_batch(&store, &filter_from(args, Some(last_id)), args.json).await?;
            }
        }
    }
    Ok(())
}
