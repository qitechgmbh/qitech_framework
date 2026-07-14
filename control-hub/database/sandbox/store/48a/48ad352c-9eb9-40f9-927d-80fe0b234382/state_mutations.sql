ATTACH TABLE _ UUID '2ce5d5a8-904e-4911-8c60-5f29f4ff93aa'
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
