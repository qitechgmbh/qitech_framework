ATTACH TABLE _ UUID '3967c5c8-de0b-4aec-8d79-cba58c0b6961'
(
    `timestamp` DateTime64(3, 'UTC'),
    `origin` UInt64,
    `name` LowCardinality(String),
    `value` JSON,
    PROJECTION by_time
    (
        SELECT *
        ORDER BY timestamp
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (origin, name, timestamp)
SETTINGS index_granularity = 8192
