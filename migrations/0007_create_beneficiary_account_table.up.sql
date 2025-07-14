CREATE TABLE IF NOT EXISTS beneficiary_account
(
    id                VARCHAR(255)             NOT NULL PRIMARY KEY,
    locked            BOOLEAN                  NOT NULL DEFAULT FALSE,
    app_id            VARCHAR(255)             NOT NULL,
    creation_time     TIMESTAMP WITH TIME ZONE NOT NULL,
    modification_time TIMESTAMP WITH TIME ZONE NOT NULL,
    admin_user_fps    VARCHAR(255)[],
    holders_user_fps  VARCHAR(255)[],
    status            account_status           NOT NULL DEFAULT 'Active',
    acct_type         account_type             NOT NULL DEFAULT 'SystemFee',

    -- This constraint ensures the 'admin_user_fps' & 'holders_user_fps array must have at least one element.
    -- cardinality() returns the total number of elements in an array.
    CONSTRAINT admin_user_fps_must_not_be_empty CHECK (cardinality(admin_user_fps) > 0),
    CONSTRAINT holders_user_fps_must_not_be_empty CHECK (cardinality(holders_user_fps) > 0)
)
