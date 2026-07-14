CREATE TABLE control_hub.state_mutations
(
    timestamp           DateTime64(3, 'UTC'),
    ident_vendor        UInt16,
    ident_machine       UInt16,
    ident_serial        UInt16,
    name                LowCardinality(String),

    -- value --
    value_type          Enum8('String'=1,'IntegerSigned'=2,'IntegerUnsigned'=3,'Float'=4,'Boolean'=5),
    value_string        Nullable(String),
    value_int_signed    Nullable(Int64),
    value_int_unsigned  Nullable(UInt64),
    value_float         Nullable(Float64),
    value_bool          Nullable(Bool),

    -- store duplicate ordered by timestamp only for history queries of all machines
    PROJECTION by_time
    (
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (ident_vendor, ident_machine, ident_serial, timestamp)
