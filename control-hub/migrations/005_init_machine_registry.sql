CREATE TABLE control_hub.machine_registry(
    ident_vendor  UInt16,
    ident_machine UInt16,
    ident_serial  UInt16,
)
ENGINE = MergeTree
ORDER BY (ident_vendor, ident_machine, ident_serial);