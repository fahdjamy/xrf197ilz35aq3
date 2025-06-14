use crate::core::{Account, AccountStatus, AccountType, Currency};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};
use tracing::info;

#[tracing::instrument(level = "debug", skip(pg_pool, account), name = "Create new account")]
pub async fn save_account<'a, E>(pg_pool: E, account: &Account) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("creating new account :: acct={}", account);
    let result = sqlx::query!(
        "
INSERT INTO user_account (
                     id,
                     status,
                     locked,
                     user_fp,
                     timezone,
                     currency,
                     acct_type,
                     creation_time,
                     modification_time
                     )
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
",
        account.id,
        account.status.clone() as AccountStatus,
        account.locked,
        account.user_fp,
        account.timezone,
        account.currency.clone() as Currency,
        account.account_type.clone() as AccountType,
        account.creation_time,
        account.modification_time
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
