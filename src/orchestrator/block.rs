use crate::context::ApplicationContext;
use crate::core::chain_stamp::ChainStamp;
use crate::core::Block;
use std::collections::HashSet;
use tracing::error;

pub fn create_block(
    app_ctx: ApplicationContext,
    ledger_entry_ids: HashSet<String>,
    ancestor_stamp: Option<ChainStamp>,
) -> Result<Block, String> {
    if ledger_entry_ids.is_empty() {
        return Err("No ledger_entry_ids given".to_string());
    }

    let entry_ids = ledger_entry_ids.into_iter().collect::<Vec<String>>();

    let block_chain_stamp = ChainStamp::build(ancestor_stamp);

    let block = Block::build(
        app_ctx.app_id.to_string(),
        app_ctx.block_region,
        entry_ids,
        block_chain_stamp.stamp,
    )
    .map_err(|err| {
        error!("Error creating block: {}", err);
        err.to_string()
    })?;

    Ok(block)
}
