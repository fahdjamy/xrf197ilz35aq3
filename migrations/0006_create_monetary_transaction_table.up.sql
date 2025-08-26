CREATE TYPE monetary_tx_status AS ENUM ('Failed', 'Pending', 'Rejected', 'Reverted', 'Completed');
CREATE TYPE monetary_tx_type AS ENUM ('Payment', 'Transfer', 'Reversal', 'Commission', 'Correction', 'Initialization');

CREATE TABLE IF NOT EXISTS monetary_transaction
(
    account_id        VARCHAR(100)             NOT NULL,
    amount            NUMERIC(25, 4)           NOT NULL,
    timestamp         TIMESTAMP WITH TIME ZONE NOT NULL,
    modification_date TIMESTAMP WITH TIME ZONE NOT NULL,
    transaction_type  monetary_tx_type         NOT NULL,
    status            monetary_tx_status       NOT NULL,
    transaction_id    VARCHAR(500)             NOT NULL PRIMARY KEY
)
