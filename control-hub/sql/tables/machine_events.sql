CREATE TABLE machine_events
(
    timestamp       DateTime64(3, 'UTC'),

    ident_vendor    UInt16,
    ident_machine   UInt16,
    ident_serial    UInt16,

    name            LowCardinality(String),
    data            JSON,

    PROJECTION by_time
    (
        SELECT *
        ORDER BY ts
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(ts)
ORDER BY (ident_vendor, ident_machine, ident_serial, ts)
TTL ts + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;