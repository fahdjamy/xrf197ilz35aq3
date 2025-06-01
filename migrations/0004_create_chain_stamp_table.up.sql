-- Add up a migration script here
CREATE TABLE IF NOT EXISTS chain_stamp
(
    chain_stamp_id    VARCHAR(500)             NOT NULL PRIMARY KEY,
    timestamp         TIMESTAMP WITH TIME ZONE NOT NULL,
    modification_time TIMESTAMP WITH TIME ZONE NOT NULL,
    version           VARCHAR(10)              NOT NULL,
    root_stamp        VARCHAR(10),
    child_stamp       VARCHAR(500)
)
