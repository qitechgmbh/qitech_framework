CREATE TABLE control_hub.config_mutations(
    timestamp           DateTime64(3, 'UTC'),
    identity            UInt64,
    name                LowCardinality(String),

    -- value --
    value_type          Enum8('String', 'IntegerSigned', 'IntegerUnsigned', 'Float', 'Boolean'),
    value_string        Nullable(String),
    value_int_signed    Nullable(Int64),
    value_int_unsigned  Nullable(UInt64),
    value_float         Nullable(Float64),
    value_bool          Nullable(Bool),
    origin              Nullable(UInt64),
    result              Enum8('Success', 'OutOfBounds', 'InvalidInput'),

    -- store duplicate ordered by timestamp only for history queries of all machines
    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (ident_vendor, ident_machine, ident_serial, timestamp)
