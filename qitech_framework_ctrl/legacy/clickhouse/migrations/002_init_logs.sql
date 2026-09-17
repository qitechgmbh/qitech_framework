CREATE TABLE control_hub.logs(
    timestamp DateTime64(3, 'UTC'),

    level Enum8(
        'Trace' = 1,
        'Debug' = 2,
        'Info'  = 3,
        'Warn'  = 4,
        'Error' = 5
    ),

    origin UInt64,
    message String,
    attributes Map(String, String),

    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (level, timestamp)

-- expiry time for data (30 Days)
TTL timestamp + INTERVAL 30 DAY
