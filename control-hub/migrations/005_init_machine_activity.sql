CREATE TABLE control_hub.machine_activity(
    identity   UInt64,
    updated_at DateTime64(3, 'UTC')
)
ENGINE = ReplacingMergeTree(updated_at)
ORDER BY identity;
