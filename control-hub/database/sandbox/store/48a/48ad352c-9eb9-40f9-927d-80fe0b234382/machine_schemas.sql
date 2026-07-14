ATTACH TABLE _ UUID 'c541c0a0-8adc-40dd-955d-9c2d92fddce4'
(
    `ident_vendor` UInt16,
    `ident_machine` UInt16,
    `data` JSON
)
ENGINE = MergeTree
ORDER BY (ident_vendor, ident_machine)
SETTINGS index_granularity = 8192
