use crate::core::{Account, AccountStatus, AccountType, Currency};
use crate::PgDatabaseError;
use sqlx::PgPool;
use tracing::info;

#[tracing::instrument(level = "debug", skip(pg_pool, account), name = "Create new account")]
pub async fn create_account(pg_pool: &PgPool, account: &Account) -> Result<bool, PgDatabaseError> {
    info!("creating new account :: acct={}", account);
    let result = sqlx::query!(
        "
INSERT INTO account (
                     id,
                     type,
                     status,
                     locked,
                     user_fp,
                     timezone,
                     currency,
                     creation_time,
                     modification_time
                     )
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
",
        account.id,
        account.account_type.clone() as AccountType,
        account.status.clone() as AccountStatus,
        account.locked,
        account.user_fp,
        account.timezone,
        account.currency.clone() as Currency,
        account.creation_time,
        account.modification_time
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
