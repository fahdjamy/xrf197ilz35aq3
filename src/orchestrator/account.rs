use crate::context::ApplicationContext;
use crate::core::{
    Account, AccountType, Block, BlockRegion, Currency, EntryType, LedgerEntry, WalletHolding,
};
use crate::{ChainStamp, DomainError};
use std::str::FromStr;

pub fn create_account(
    user_fp: String,
    currency: String,
    acct_type: String,
    timezone: String,
    app_cxt: ApplicationContext,
) -> Result<(Account, WalletHolding), DomainError> {
    let curr = Currency::from_str(&currency)?;
    let acct_type = AccountType::from_str(&acct_type)?;
    let block_region = BlockRegion::from_str(&app_cxt.region)?;

    // 1. create an account
    let account = Account::new(user_fp, timezone, curr, acct_type);
    // 2. create wallet that belongs to the account
    let wallet_holding = WalletHolding::new(account.id.clone());
    // 3. Create the initialization transaction. should have a ledger for record keeping
    let description = Some("initialization for newly created account".to_string());
    let ledger = LedgerEntry::new(account.id.clone(), description, EntryType::Credit);
    let mut entry_ids = Vec::new();
    entry_ids.push(ledger.id.clone());

    // 4. Create block for ledger-entry grouping. This block will contain the root chain_stamp
    let _ = Block::build(
        app_cxt.app_id.to_string(),
        block_region,
        entry_ids,
        ChainStamp::build(None),
    )?;

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
            ApplicationContext::load_test_ctx(11),
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
