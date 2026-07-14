ATTACH TABLE _ UUID '1de5260c-8ff4-4e9c-b06b-d6e557e771cf'
(
    `timestamp` DateTime64(3, 'UTC'),
    `ident_vendor` UInt16,
    `ident_machine` UInt16,
    `ident_serial` UInt32,
    `name` LowCardinality(String),
    `value` Nullable(Float64),
    PROJECTION by_time
    (
        SELECT *
        ORDER BY timestamp
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (ident_vendor, ident_machine, ident_serial, name, timestamp)
TTL timestamp + toIntervalDay(30)
SETTINGS index_granularity = 8192
