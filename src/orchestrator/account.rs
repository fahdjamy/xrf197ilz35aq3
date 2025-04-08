use crate::core::{Account, AccountType, Currency, WalletHolding};
use crate::DomainError;
use std::str::FromStr;

pub fn create_account(
    user_fp: String,
    currency: String,
    acct_type: String,
    tz: String,
) -> Result<(Account, WalletHolding), DomainError> {
    let curr = Currency::from_str(&currency)?;
    let acct_type = AccountType::from_str(&acct_type)?;

    let account = Account::new(user_fp, tz, curr, acct_type);
    let wallet_holding = WalletHolding::new(account.id.clone());

    Ok((account, wallet_holding))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    pub fn test_create_account() {
        let result = create_account(
            Uuid::new_v4().to_string(),
            "BTC".to_string(),
            "Normal".to_string(),
            "UTC".to_string(),
        );

        assert!(result.is_ok());

        let (account, wallet_holding) = result.unwrap();

        assert_eq!(account.freeze, false);
        assert!(wallet_holding.balance.is_zero());
        assert_eq!(account.currency, Currency::BTC);
        assert!(wallet_holding.last_entry_id.is_none());
        assert_eq!(wallet_holding.account_id, account.id);
        assert_eq!(account.account_type, AccountType::Normal);
    }
}
