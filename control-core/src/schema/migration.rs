// use crate::schema::latest::;

pub enum ConfigRevisionMigration {
    RemoveField { field: String },
    ChangeName { field: String, new_name: String },
    ChangeType { field: String, new_type: () },
}

pub enum StateRevisionMigration {
    RemoveField { field: String },
    ChangeName { field: String, new_name: String },
    ChangeType { field: String, new_type: () },
}

pub struct SchemaRevisionMigration {
    from_revision: u16,
    to_revision: u16,

    // changelog
    config: Vec<ConfigRevisionMigration>,
}

// MachineRegistry, .add("laser_v1.yaml", vec![ SchemaRevisionMigration {} ])
// ControlHub::new()