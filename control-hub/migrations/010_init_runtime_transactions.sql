CREATE TABLE control_hub.runtime_transactions
(
    transaction_id UInt64,
    client_ip IPv4,

    started DateTime64(3, 'UTC'),
    completed DateTime64(3, 'UTC'),

    status_code Int32,
    message String
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(started)
ORDER BY (client_ip, transaction_id);