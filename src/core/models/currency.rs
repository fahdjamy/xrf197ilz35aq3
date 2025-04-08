#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Currency {
    #[strum(
        serialize = "USD",
        serialize = "usd",
        serialize = "US Dollar",
        serialize = "us dollar"
    )]
    USD,
    #[strum(
        serialize = "EUR",
        serialize = "eur",
        serialize = "Euro",
        serialize = "euro"
    )]
    EUR,
    #[strum(
        serialize = "XRP",
        serialize = "xrp",
        serialize = "Ripple",
        serialize = "ripple"
    )]
    XRP,
    #[strum(
        serialize = "RUB",
        serialize = "rub",
        serialize = "Russian Ruble",
        serialize = "russian ruble"
    )]
    RUB,
    #[strum(
        serialize = "ARS",
        serialize = "ars",
        serialize = "Argentine Peso",
        serialize = "argentine peso"
    )]
    ARS,
    #[strum(
        serialize = "BRL",
        serialize = "brl",
        serialize = "Brazilian Real",
        serialize = "brazilian real"
    )]
    BRL,
    #[strum(
        serialize = "CNY",
        serialize = "cny",
        serialize = "Chinese Yuan",
        serialize = "chinese yuan"
    )]
    CNY,
    #[strum(
        serialize = "GBP",
        serialize = "gbp",
        serialize = "British Pound",
        serialize = "british pound",
        serialize = "Pound Sterling",
        serialize = "pound sterling"
    )]
    GBP,
    #[strum(
        serialize = "MXN",
        serialize = "mxn",
        serialize = "Mexican Peso",
        serialize = "mexican peso"
    )]
    MXN,
    #[strum(
        serialize = "QAR",
        serialize = "qar",
        serialize = "Qatari Rial",
        serialize = "qatari rial"
    )]
    QAR,
    #[strum(
        serialize = "JPY",
        serialize = "jpy",
        serialize = "Japanese Yen",
        serialize = "japanese yen"
    )]
    JPY,
    ////////// CRYPTO Currencies
    #[strum(
        serialize = "DOGE",
        serialize = "doge",
        serialize = "Dogecoin",
        serialize = "dogecoin"
    )]
    DOGE,
    #[strum(serialize = "XRFQ", serialize = "xrfq")]
    // Assuming this is a made-up or very specific currency code
    XRFQ,
    #[strum(
        serialize = "SOL",
        serialize = "sol",
        serialize = "Solana",
        serialize = "solana",
        serialize = "SOLANA"
    )]
    SOL,
    #[strum(
        serialize = "BTC",
        serialize = "btc",
        serialize = "Bitcoin",
        serialize = "bitcoin",
        serialize = "BITCOIN"
    )]
    BTC,
    #[strum(
        serialize = "ETH",
        serialize = "eth",
        serialize = "Ethereum",
        serialize = "ethereum",
        serialize = "ETHEREUM"
    )]
    ETH,
    #[strum(
        serialize = "ADA",
        serialize = "ada",
        serialize = "Cardano",
        serialize = "cardano",
        serialize = "CARDANO"
    )]
    ADA,
    #[strum(
        serialize = "USDT",
        serialize = "usdt",
        serialize = "Tether",
        serialize = "tether",
        serialize = "TETHER"
    )]
    USDT,
    #[strum(
        serialize = "BNB",
        serialize = "bnb",
        serialize = "Binance Coin",
        serialize = "binance coin",
        serialize = "BNB Coin",
        serialize = "BinanceCoin"
    )]
    BNB,
}
