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
) -> Result<Option<Account>, PgDatabaseError>
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
    .await;

    match saved_account {
        Ok(account) => Ok(Some(account.into())),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, ben_acct),
    name = "Save beneficiary account"
)]
pub async fn save_beneficiary_account<'a, E>(
    pg_pool: E,
    ben_acct: &BeneficiaryAccount,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("create new beneficiary account");
    let result = sqlx::query!(
        "
INSERT INTO beneficiary_account (
                                 id,
                                 locked,
                                 app_id,
                                 creation_time,
                                 modification_time,
                                 admin_user_fps,
                                 holders_user_fps,
                                 status,
                                 acct_type
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
",
        ben_acct.id,
        ben_acct.locked,
        ben_acct.app_id,
        ben_acct.creation_time,
        ben_acct.modification_time,
        &ben_acct.account_admins,
        &ben_acct.account_holders,
        ben_acct.status.clone() as AccountStatus,
        ben_acct.account_type.clone() as AccountType
    )
    .execute(pg_pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, user_fp),
    name = "Find all user accounts"
)]
pub async fn fetch_user_accounts_by_currencies_and_types<'a, E>(
    pg_pool: E,
    user_fp: &str,
    currencies: &[Currency],
    acct_types: &[AccountType],
) -> Result<Vec<Account>, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("fetching user accounts");

    let result: Vec<AccountDO> = sqlx::query_as!(
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
FROM user_account
WHERE user_fp = $1
    AND (array_length($2::account_type[], 1) IS NULL OR acct_type = ANY($2::account_type[]))
"#,
        user_fp,
        acct_types as &[AccountType]
    )
    .fetch_all(pg_pool)
    .await?;

    let result: Vec<Account> = result.into_iter().map(Account::from).collect();

    Ok(result)
}
