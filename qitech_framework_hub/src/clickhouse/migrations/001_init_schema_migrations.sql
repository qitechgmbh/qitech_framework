CREATE TABLE control_hub.schema_migrations
(
    version UInt64,
    applied_at DateTime64 DEFAULT now()
)
ENGINE = MergeTree
ORDER BY version