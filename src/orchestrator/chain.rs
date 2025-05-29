use crate::core::chain_stamp::ChainStamp;
use crate::error::OrchestrateError;
use crate::storage::save_chain_stamp;
use sqlx::{Executor, Postgres};

pub async fn create_chain_stamp<'a, E>(
    pool: E,
    root_cs: Option<ChainStamp>,
) -> Result<ChainStamp, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let chain_stamp = ChainStamp::build(root_cs);
    let cs_created = save_chain_stamp(pool, &chain_stamp).await?;

    if !cs_created {
        return Err(OrchestrateError::ServerError(format!(
            "could not save chain stamp in database {}",
            chain_stamp
        )));
    }
    Ok(chain_stamp)
}
