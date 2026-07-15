CREATE TABLE control_hub.events(
    timestamp       DateTime64(3, 'UTC'),
    origin          UInt64,
    name            LowCardinality(String),
    value           JSON,

    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (origin, name, timestamp)
