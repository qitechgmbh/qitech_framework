ATTACH TABLE _ UUID '743f6778-579b-470f-8abf-7a1a21607faa'
(
    `version` UInt64,
    `applied_at` DateTime64(3) DEFAULT now()
)
ENGINE = MergeTree
ORDER BY version
SETTINGS index_granularity = 8192
