ATTACH TABLE _ UUID 'c84c6133-9d1d-4c80-931d-2b9d264d0abb'
(
    `timestamp` DateTime64(3, 'UTC'),
    `ident_vendor` UInt16,
    `ident_machine` UInt16,
    `ident_serial` UInt16,
    `name` LowCardinality(String),
    `value_type` Enum8('String' = 1, 'IntegerSigned' = 2, 'IntegerUnsigned' = 3, 'Float' = 4, 'Boolean' = 5),
    `value_string` Nullable(String),
    `value_int_signed` Nullable(Int64),
    `value_int_unsigned` Nullable(UInt64),
    `value_float` Nullable(Float64),
    `value_bool` Nullable(Bool),
    `origin` Enum8('User' = 1, 'Machine' = 2),
    `result` Enum8('Success' = 1, 'OutOfBounds' = 2),
    PROJECTION by_time
    (
        SELECT *
        ORDER BY timestamp
    )
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (ident_vendor, ident_machine, ident_serial, timestamp)
SETTINGS index_granularity = 8192
