-- Add up a migration script here

CREATE TYPE entry_type AS ENUM ('Debit', 'Credit');

CREATE TABLE IF NOT EXISTS ledger_entry
(
    id              VARCHAR(255)             NOT NULL PRIMARY KEY,
    account_id      VARCHAR(255)             NOT NULL,
    description     TEXT,
    sequence_number BIGINT                   NOT NULL DEFAULT 0,
    timestamp       TIMESTAMP WITH TIME ZONE NOT NULL,
    entry_type      entry_type               NOT NULL,
    FOREIGN KEY (account_id) REFERENCES user_account (id)
)
