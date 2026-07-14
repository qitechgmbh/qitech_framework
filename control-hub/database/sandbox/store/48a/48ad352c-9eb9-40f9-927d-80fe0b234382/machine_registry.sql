ATTACH TABLE _ UUID '2befcf82-049e-43cc-be0b-7e78e8225f3a'
(
    `ident_vendor` UInt16,
    `ident_machine` UInt16,
    `ident_serial` UInt16
)
ENGINE = MergeTree
ORDER BY (ident_vendor, ident_machine, ident_serial)
SETTINGS index_granularity = 8192
