use crate::core::chain_stamp::{ChainStamp, ChainStampVersion};
use crate::PgDatabaseError;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres};
use std::str::FromStr;
use tracing::{info, warn};

struct ChainStampDO {
    chain_stamp_id: String,
    version: String,
    timestamp: DateTime<Utc>,
    root_stamp: Option<String>,
    child_stamp: Option<String>,
    modification_time: DateTime<Utc>,
}

impl From<ChainStampDO> for ChainStamp {
    fn from(value: ChainStampDO) -> Self {
        let stamp_version = match ChainStampVersion::from_str(&value.version) {
            Ok(v) => v,
            Err(_) => {
                warn!("Invalid chain stamp version found in DB: {}", value.version);
                ChainStampVersion::V1
            }
        };

        ChainStamp {
            root_stamp: None,
            child_stamp: None,
            version: stamp_version,
            timestamp: value.timestamp,
            stamp: value.chain_stamp_id,
            modification_time: value.modification_time,
        }
    }
}

#[tracing::instrument(level = "debug", skip(pool, chain), name = "Create chain stamp")]
pub async fn save_chain_stamp<'a, E>(pool: E, chain: &ChainStamp) -> Result<bool, PgDatabaseError>
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
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(level = "debug", skip(pool, chain_id), name = "Find chain stamp by id")]
pub async fn find_chain_stamp_by_id<'a, E>(
    pool: E,
    chain_id: &str,
) -> Result<Option<ChainStamp>, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("finding chain stamp by id");
    let result = sqlx::query_as!(
        ChainStampDO,
        "
SELECT chain_stamp_id, timestamp, modification_time, version, root_stamp, child_stamp
FROM chain_stamp
WHERE chain_stamp_id = $1",
        chain_id
    )
    .fetch_optional(pool)
    .await?;

    match result {
        Some(chain_stamp) => Ok(Some(chain_stamp.into())),
        None => Ok(None),
    }
}
