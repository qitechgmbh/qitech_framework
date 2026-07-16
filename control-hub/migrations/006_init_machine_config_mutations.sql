CREATE TABLE control_hub.machine_config_mutations(
    timestamp       DateTime64(3, 'UTC'),
    identity        UInt64,
    name            LowCardinality(String),

    -- value --
    value_type      Enum8('String', 'Enum', 'Integer', 'Float', 'Boolean'),
    value_enum      LowCardinality(String),
    value_string    Nullable(String),
    value_int       Nullable(Int64),
    value_float     Nullable(Float64),
    value_bool      Nullable(Bool),
    origin          UInt64,
    result          Enum8('Success', 'OutOfBounds', 'InvalidInput'),

    -- store duplicate ordered by timestamp only for history queries of all machines
    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (identity, timestamp)
