use crate::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "currency_enum")]
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
            Currency::XRFQ => write!(f, "XRFQ"),
            Currency::SOL => write!(f, "SOL"),
            Currency::BTC => write!(f, "BTC"),
            Currency::USDT => write!(f, "USDT"),
            Currency::ADA => write!(f, "ADA"),
            Currency::ETH => write!(f, "ETH"),
            Currency::BNB => write!(f, "BNB"),
        }
    }
}
