CREATE TABLE IF NOT EXISTS xrf_q3_block.block_chain
(
    id            text,                -- Regular column (should be unique, but not the primary query path)
    chain_id      text,
    app_id        text,                -- Partition Key: Groups data by application
    sequence_num  bigint,              -- Clustering Key: Orders blocks within an app_id
    entry_ids     list<text>,
    region        text,                -- Store enum variant as text
    version       text,
    creation_date timestamp,
    PRIMARY KEY (app_id, sequence_num) -- Partition by app_id, cluster by sequence_num
)
