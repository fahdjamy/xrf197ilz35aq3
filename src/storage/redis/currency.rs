use crate::core::Currency;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{info, warn};

pub fn get_exchange_rate(
    currency_hash: String,
    from_curr: &Currency,
    to_currency: &Currency,
) -> Option<Decimal> {
    // TODO: This should be coming from redis DB
    let mut hash_set: HashMap<String, Decimal> = HashMap::new();
    if !hash_set.contains_key(currency_hash.as_str()) {
        hash_set.insert(currency_hash.clone(), Decimal::from(0));
    }

    match hash_set.get(currency_hash.as_str()) {
        Some(hash) => Some(*hash),
        None => {
            warn!("no currency found for {} to {}", from_curr, to_currency);
            None
        }
    }
}
