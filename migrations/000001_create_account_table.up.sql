CREATE TYPE currency_enum AS ENUM (
    'USD',
    'EUR',
    'XRP',
    'RUB',
    'ARS',
    'BRL',
    'CNY',
    'GBP',
    'MXN',
    'QAR',
    'JPY',
    'DOGE',
    'XRFQ',
    'SOL',
    'BTC',
    'ETH',
    'ADA',
    'USDT',
    'BNB');

CREATE TYPE account_status AS ENUM ('Frozen', 'Active', 'Inactive');

CREATE TYPE account_type AS ENUM ('Normal', 'Wallet', 'Escrow', 'SystemFee');

CREATE TABLE IF NOT EXISTS account
(
    id                VARCHAR(255)             NOT NULL PRIMARY KEY,
    locked            BOOLEAN                  NOT NULL DEFAULT FALSE,
    timezone          VARCHAR(255)             NOT NULL,
    user_fp           VARCHAR(255)             NOT NULL,
    currency          currency_enum            NOT NULL,
    creation_time     TIMESTAMP WITH TIME ZONE NOT NULL,
    modification_time TIMESTAMP WITH TIME ZONE NOT NULL,
    type              account_type             NOT NULL DEFAULT 'Normal',
    status            account_status           NOT NULL DEFAULT 'Active'
)
