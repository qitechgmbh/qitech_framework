CREATE TABLE control_hub.machine_schemas(
    ident_vendor  UInt16,
    ident_machine UInt16,
    data JSON
)
ENGINE = MergeTree
ORDER BY (ident_vendor, ident_machine);