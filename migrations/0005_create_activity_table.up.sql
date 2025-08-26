-- Add up a migration script here
CREATE TABLE IF NOT EXISTS activity
(
    user_fp           VARCHAR(250)             NOT NULL,
    chain_id          VARCHAR(500)             NOT NULL,
    block_id          VARCHAR(100)             NOT NULL,
    description       TEXT                     NOT NULL,
    timestamp         TIMESTAMP WITH TIME ZONE NOT NULL,
    modification_time TIMESTAMP WITH TIME ZONE NOT NULL,
    transaction_id    VARCHAR(500)             NOT NULL PRIMARY KEY
)
