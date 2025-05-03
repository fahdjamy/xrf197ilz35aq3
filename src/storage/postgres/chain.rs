use crate::{ChainStamp, PgDatabaseError};
use sqlx::{Executor, Postgres};
use tracing::info;

#[tracing::instrument(level = "debug", skip(pg_pool, chain), name = "Create chain stamp")]
pub async fn save_chain_stamp<'a, E>(
    pg_pool: E,
    chain: &ChainStamp,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("saving chain stamp to DB :: acct={}", chain);
    let result = sqlx::query!(
        "
    INSERT INTO chain_stamp
        (chain_stamp_id, timestamp, modification_time, version, root_stamp, child_stamp)
        VALUES ($1, $2, $3, $4, $5, $6)
    ",
        chain.stamp,
        chain.timestamp,
        chain.modification_time,
        &chain.version.to_string(),
        chain.root_stamp,
        chain.child_stamp
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
