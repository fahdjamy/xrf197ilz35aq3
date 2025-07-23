use crate::DomainError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "currency_enum")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum Currency {
    USD,
    EUR,
    XRP,
    RUB,
    ARS,
    BRL,
    CNY,
    GBP,
    MXN,
    QAR,
    JPY,
    ////////// CRYPTO Currencies
    XRFQ,
    SOL,
    BTC,
    ETH,
    ADA,
    USDT,
    BNB,
    /////// NOT Recognized
    NOTSUPPORTED,
}

impl Currency {
    pub fn is_crypto(&self) -> bool {
        match self {
            Currency::BTC
            | Currency::ETH
            | Currency::SOL
            | Currency::ADA
            | Currency::USDT
            | Currency::XRFQ
            | Currency::BNB => true,
            _ => false,
        }
    }
}

impl FromStr for Currency {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "XRP" => Ok(Currency::XRP),
            "RUB" => Ok(Currency::RUB),
            "ARS" => Ok(Currency::ARS),
            "BRL" => Ok(Currency::BRL),
            "CNY" => Ok(Currency::CNY),
            "GBP" => Ok(Currency::GBP),
            "MXN" => Ok(Currency::MXN),
            "QAR" => Ok(Currency::QAR),
            "JPY" => Ok(Currency::JPY),
            "SOL" => Ok(Currency::SOL),
            "BTC" => Ok(Currency::BTC),
            "ETH" => Ok(Currency::ETH),
            "ADA" => Ok(Currency::ADA),
            "BNB" => Ok(Currency::BNB),
            "XRFQ" => Ok(Currency::XRFQ),
            "USDT" => Ok(Currency::USDT),
            "NOTSUPPORTED" => Ok(Currency::NOTSUPPORTED),
            _ => Err(DomainError::ParseError("unrecognized currency".to_string())),
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::XRP => write!(f, "XRP"),
            Currency::RUB => write!(f, "RUB"),
            Currency::ARS => write!(f, "ARS"),
            Currency::BRL => write!(f, "BRL"),
            Currency::CNY => write!(f, "CNY"),
            Currency::GBP => write!(f, "GBP"),
            Currency::MXN => write!(f, "MXN"),
            Currency::QAR => write!(f, "QAR"),
            Currency::JPY => write!(f, "JPY"),
            Currency::SOL => write!(f, "SOL"),
            Currency::BTC => write!(f, "BTC"),
            Currency::ADA => write!(f, "ADA"),
            Currency::ETH => write!(f, "ETH"),
            Currency::BNB => write!(f, "BNB"),
            Currency::XRFQ => write!(f, "XRFQ"),
            Currency::USDT => write!(f, "USDT"),
            Currency::NOTSUPPORTED => write!(f, "NOTSUPPORTED"),
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct CurrencyRate {
    pub hash: String,
    pub rate: Decimal,
    pub app_id: String,
    pub base_currency: Currency,
    pub quote_currency: Currency,
    pub recorded_at: DateTime<Utc>,
}

impl Display for CurrencyRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "appId={} :: base_currency={} :: quote_currency={}",
            self.app_id, self.base_currency, self.quote_currency
        )
    }
}

/// Generates a unique identifier from two currency string slices.
///
/// This function creates a canonical representation of the two strings by joining them
/// with a null byte separator. This unambiguous representation is then hashed using
/// the SHA3-256 cryptographic algorithm.
///
/// The resulting 256-bit hash is returned as a 64-character hexadecimal string.
///
/// # Arguments
///
/// * `s1` - The first string slice.
/// * `s2` - The second string slice.
///
/// # Returns
///
/// A `String` representing the hexadecimal value of the SHA-256 hash. The probability
/// of a collision is negligible for any practical purpose.
///
/// # Panics
///
/// This function will not panic.
pub fn get_currency_hash(s1: &str, s2: &str) -> String {
    // 1. Create a new SHA-256 hasher instance.
    let mut hasher = Sha3_256::new();

    // 2. Create the unambiguous input.
    // We update the hasher with the bytes of the first string,
    // then a separator, then the bytes of the second string.
    // The separator is crucial to distinguish between inputs like
    // ("AB", "C") and ("A", "BC"). A null byte is a good choice.
    hasher.update(s1.as_bytes());
    hasher.update(b"\0");
    hasher.update(s2.as_bytes());

    // 3. Finalize the hash computation.
    // The `finalize()` method consumes the hasher and returns a GenericArray.
    let result = hasher.finalize();

    // 4. Format the raw byte array as a hexadecimal string for easy use.
    // The `Output` type from the digest crate (v0.10+) doesn't implement
    // formatting traits like `LowerHex` directly. We manually iterate over
    // the bytes and format them into a hex string.

    // hasher.finalize() call produces a GenericArray of bytes (e.g., [56, 193, 167, ...] for SHA3-256).
    // .iter() creates an iterator that lets us process each byte one by one
    result
        .iter()
        // - (x): Format the number as lowercase hexadecimal
        // - (2): Ensure the output string is at least 2 characters wide
        // - (0): If the output is less than 2 characters, pad it with a leading 0
        // Why is: 02x so important? A single byte can range from 0 to 255. In hexadecimal,
        // this is 00 to ff. A byte with the value 10 is 0a in hex. If we only used {:x}
        // it would format as "a". By using {:02x}, we guarantee it formats as "0a". This ensures
        // every byte becomes exactly two hex characters, giving us a correctly formatted, fixed-length hash string
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hash_code() {
        let str1 = get_currency_hash("test", "string");
        let str2 = get_currency_hash("test", "string");
        assert_eq!(
            str1, str2,
            "Hashing the same inputs should always produce the same output."
        );

        let str1 = get_currency_hash("hello", "world");
        let str2 = get_currency_hash("hello", "world");
        assert_eq!(
            str1, str2,
            "Hashing the same inputs should always produce the same output."
        );

        let str1 = get_currency_hash("helloworld", "");
        let str2 = get_currency_hash("hello", "world");
        assert_ne!(
            str1, str2,
            "Hashing the same inputs should always produce the same output."
        );
    }
}
