ATTACH TABLE _ UUID 'e63e0982-3c51-45a3-aecb-ec5ae092d40b'
(
    `timestamp` DateTime64(3, 'UTC'),
    `level` Enum8('Trace' = 1, 'Debug' = 2, 'Info' = 3, 'Warn' = 4, 'Error' = 5),
    `origin` UInt64,
    `message` String,
    `attributes` Map(String, String),
    PROJECTION by_time
    (
        SELECT *
        ORDER BY timestamp
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (level, timestamp)
TTL timestamp + toIntervalDay(30)
SETTINGS index_granularity = 8192
