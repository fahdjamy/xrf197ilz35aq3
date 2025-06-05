use crate::core::chain_stamp::ChainStamp;
use crate::error::OrchestrateError;
use crate::storage::{add_child_cs_to_parent, save_chain_stamp};
use sqlx::{Postgres, Transaction};

pub async fn create_chain_stamp(
    db_tx: &mut Transaction<'_, Postgres>,
    root_cs: Option<ChainStamp>,
) -> Result<ChainStamp, OrchestrateError> {
    let chain_stamp = ChainStamp::build(root_cs.clone());
    let cs_created = save_chain_stamp(db_tx, &chain_stamp).await?;

    if !cs_created {
        return Err(OrchestrateError::ServerError(format!(
            "could not save chain stamp in database {}",
            chain_stamp
        )));
    }

    // add this chain stamp to parent
    if root_cs.is_some() {
        add_child_chain_stamp(&chain_stamp, root_cs.unwrap(), db_tx).await?;
    }
    Ok(chain_stamp)
}

async fn add_child_chain_stamp(
    child_cs: &ChainStamp,
    mut parent_chain_stamp: ChainStamp,
    db_tx: &mut Transaction<'_, Postgres>,
) -> Result<bool, OrchestrateError> {
    if parent_chain_stamp.is_child_chain(&child_cs) {
        // Child was already set as a child for this parent cs. No need to update
        return Ok(true);
    } else if !child_cs.has_parent() || !child_cs.is_parent(&parent_chain_stamp) {
        return Err(OrchestrateError::RowConstraintViolation(
            "root cs is not the parent for the child".to_string(),
        ));
    }

    parent_chain_stamp.child_stamp = Some(child_cs.stamp.clone());

    if !add_child_cs_to_parent(db_tx, parent_chain_stamp.stamp_id(), child_cs.stamp_id()).await? {
        return Err(OrchestrateError::ServerError(
            "failed to add chain stamp".to_string(),
        ));
    }

    Ok(true)
}
