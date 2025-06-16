-- Add up a migration script here
CREATE TABLE IF NOT EXISTS wallet
(
    currency          currency_enum            NOT NULL,
    balance           NUMERIC(18, 4)           NOT NULL,
    modification_time TIMESTAMP WITH TIME ZONE NOT NULL,
    account_id        VARCHAR(255)             NOT NULL,
    PRIMARY KEY (account_id, currency),
    FOREIGN KEY (account_id) REFERENCES user_account (id) ON DELETE CASCADE
)
