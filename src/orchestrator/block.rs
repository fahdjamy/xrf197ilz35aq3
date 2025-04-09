use crate::context::ApplicationContext;
use crate::core::{Block, BlockRegion};
use std::collections::HashSet;
use std::str::FromStr;
use tracing::error;

pub fn create_block(
    ancestor_stamp: Option<u64>,
    app_config: ApplicationContext,
    ledger_entry_ids: HashSet<String>,
) -> Result<Block, String> {
    let block_region = BlockRegion::from_str(&app_config.region).map_err(|err| {
        error!("Error parsing region '{}': {}", app_config.region, err);
        err.to_string()
    })?;

    if ledger_entry_ids.is_empty() {
        return Err("No ledger_entry_ids given".to_string());
    }

    let entry_ids = ledger_entry_ids.into_iter().collect::<Vec<String>>();

    let ancestor_chain_stamp = calculate_ancestor_stamp(ancestor_stamp, app_config.app_id);
    let block = Block::new(
        app_config.app_id.to_string(),
        calculate_block_stamp(ancestor_chain_stamp),
        block_region,
        ancestor_chain_stamp,
        entry_ids,
    )
    .map_err(|err| {
        error!("Error creating block: {}", err);
        err.to_string()
    })?;

    Ok(block)
}

fn calculate_ancestor_stamp(ancestor_stamp: Option<u64>, app_id: u64) -> u64 {
    if let Some(ancestor_stamp) = ancestor_stamp {
        return ancestor_stamp;
    }
    1
}

fn calculate_block_stamp(ancestor_stamp: u64) -> u64 {
    ancestor_stamp + 1
}
