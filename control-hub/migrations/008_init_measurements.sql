CREATE TABLE control_hub.measurements(
    timestamp       DateTime64(3, 'UTC'),

    ident_vendor    UInt16,
    ident_machine   UInt16,
    ident_serial    UInt32,

    name            LowCardinality(String),
    value           Nullable(Float64),

    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree

PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (ident_vendor, ident_machine, ident_serial, name, timestamp)

-- expiry time for data (30 Days)
TTL timestamp + INTERVAL 30 DAY
