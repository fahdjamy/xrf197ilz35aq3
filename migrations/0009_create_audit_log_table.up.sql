-- An ENUM for the type of action performed.
CREATE TYPE audit_event_type AS ENUM ('CREATE', 'UPDATE', 'DELETE');

CREATE TABLE audit_log
(
    request_user_agent TEXT,
    request_id         VARCHAR(255),
    request_ip         VARCHAR(255),
    entity_type        VARCHAR(255)             NOT NULL,
    id                 VARCHAR(255)             NOT NULL,
    entity_id          VARCHAR(255)             NOT NULL,
    user_fp            VARCHAR(255)             NOT NULL,
    -- The type of event that occurred.
    audit_type         audit_event_type         NOT NULL,
    -- A JSONB column to store the actual data that changed.
    changes            JSONB                    NOT NULL,
    creation_time      TIMESTAMP WITH TIME ZONE NOT NULL
);

-- An index to make querying for a specific entity's history fast.
CREATE INDEX idx_audit_log_on_entity ON audit_log (entity_type, entity_id);
