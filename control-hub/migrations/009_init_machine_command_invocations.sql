CREATE TABLE control_hub.machine_command_invocations(
    timestamp       DateTime64(3, 'UTC'),
    identity        UInt64,
    name            LowCardinality(String),
    data            JSON,
    origin          UInt64,
    result          Enum8('Success', 'OutOfBounds', 'InvalidInput'),

    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (identity, name, timestamp)
