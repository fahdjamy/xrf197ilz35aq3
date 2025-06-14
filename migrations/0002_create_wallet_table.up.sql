-- Add up a migration script here
CREATE TABLE IF NOT EXISTS wallet
(
    last_entry_id     VARCHAR(255)             NOT NULL,
    balance           NUMERIC(18, 4)           NOT NULL,
    modification_time TIMESTAMP WITH TIME ZONE NOT NULL,
    account_id        VARCHAR(255)             NOT NULL PRIMARY KEY,
    FOREIGN KEY (account_id) REFERENCES user_account (id) ON DELETE CASCADE
)
