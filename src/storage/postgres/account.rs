use crate::context::ApplicationContext;
use crate::core::{Account, AccountStatus, AccountType, BeneficiaryAccount, Currency};
use crate::PgDatabaseError;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres};
use tracing::info;

#[derive(sqlx::FromRow)]
struct AccountDO {
    pub id: String,
    pub locked: bool,
    pub user_fp: String,
    pub timezone: String,
    pub currency: Currency,
    pub status: AccountStatus,
    pub acct_type: AccountType,
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
}

impl From<AccountDO> for Account {
    fn from(db_acct: AccountDO) -> Self {
        Account {
            id: db_acct.id,
            status: db_acct.status,
            locked: db_acct.locked,
            user_fp: db_acct.user_fp,
            timezone: db_acct.timezone,
            currency: db_acct.currency,
            account_type: db_acct.acct_type,
            creation_time: db_acct.creation_time,
            modification_time: db_acct.modification_time,
        }
    }
}

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

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, account_id),
    name = "Find account by id"
)]
pub async fn find_account_by_id<'a, E>(
    pg_pool: E,
    account_id: &str,
) -> Result<Account, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("finding account by id");
    let saved_account = sqlx::query_as!(
        AccountDO,
        r#"
SELECT  id,
        locked,
        user_fp,
        timezone,
        status as "status: _",
        currency as "currency: _",
        creation_time,
        modification_time,
        acct_type as "acct_type: _"
FROM user_account WHERE id = $1
    "#,
        account_id
    )
    .fetch_one(pg_pool)
    .await?;

    Ok(saved_account.into())
}

#[tracing::instrument(
    level = "debug",
    // skip(pg_pool, app_ctx),
    name = "Save beneficiary account"
)]
pub async fn save_beneficiary_account<'a, E>(
    _: E,
    _: &BeneficiaryAccount,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("finding beneficiary account with id");
    unimplemented!()
}
