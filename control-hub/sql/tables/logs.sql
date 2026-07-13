CREATE TABLE logs
(
    timestamp               DateTime64(3, 'UTC'),

    level                   Enum8(
                                'Trace' = 1,
                                'Debug' = 2,
                                'Info'  = 3,
                                'Warn'  = 4,
                                'Error' = 5
                            ),

    -- LogOrigin: discriminator + nullable ident fields (only set when origin_type = 'Machine')
    origin_type             Enum8('Machine' = 1, 'MainLoop' = 2),
    origin_ident_vendor     Nullable(UInt16),
    origin_ident_machine    Nullable(UInt16),
    origin_ident_serial     Nullable(UInt32),

    message                 String,
    attributes              Map(String, String),

    PROJECTION by_time
    (
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (severity, timestamp)
TTL timestamp + INTERVAL 30 DAY