use crate::core::Currency;
use crate::error::OrchestrateError;
use rust_decimal::Decimal;

pub fn convert_amount(
    amount: Decimal,
    from_curr: Currency,
    to_curr: Currency,
) -> Result<Decimal, OrchestrateError> {
    if to_curr == from_curr {
        return Ok(amount);
    }
    // TODO: Get from/to currency rate
    let rate = get_rate(&to_curr, &from_curr)?;

    Ok(amount * rate)
}

fn get_rate(to_currency: &Currency, from_currency: &Currency) -> Result<Decimal, OrchestrateError> {
    unimplemented!()
}
