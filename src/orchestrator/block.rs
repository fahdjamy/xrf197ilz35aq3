use crate::context::ApplicationContext;
use crate::core::{Block, BlockRegion};
use crate::ChainStamp;
use std::collections::HashSet;
use std::str::FromStr;
use tracing::error;

pub fn create_block(
    app_config: ApplicationContext,
    ledger_entry_ids: HashSet<String>,
    ancestor_stamp: Option<ChainStamp>,
) -> Result<Block, String> {
    let block_region = BlockRegion::from_str(&app_config.region).map_err(|err| {
        error!("Error parsing region '{}': {}", app_config.region, err);
        err.to_string()
    })?;

    if ledger_entry_ids.is_empty() {
        return Err("No ledger_entry_ids given".to_string());
    }

    let entry_ids = ledger_entry_ids.into_iter().collect::<Vec<String>>();

    let block_chain_stamp = ChainStamp::build(ancestor_stamp);

    let block = Block::build(
        app_config.app_id.to_string(),
        block_region,
        entry_ids,
        block_chain_stamp.stamp,
    )
    .map_err(|err| {
        error!("Error creating block: {}", err);
        err.to_string()
    })?;

    Ok(block)
}
