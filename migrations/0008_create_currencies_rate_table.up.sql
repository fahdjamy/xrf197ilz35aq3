CREATE TABLE IF NOT EXISTS currency_rates
(
    currencies_hash VARCHAR(255)             NOT NULL,
    app_id          VARCHAR(255)             NOT NULL,
    base_currency   currency_enum            NOT NULL, -- The currency you are converting from
    quote_currency  currency_enum            NOT NULL,
    rate            NUMERIC(18, 4)           NOT NULL,
    recorded_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (currencies_hash, recorded_at)         -- Composite primary key
);

-- Create an index to quickly find the latest rate for a currency pair.
CREATE INDEX idx_latest_currency_rate
    ON currency_rates (currencies_hash, recorded_at DESC);
