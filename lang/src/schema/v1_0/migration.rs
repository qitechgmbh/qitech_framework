pub struct RevisionMigration {
    pub migration: Migration<u32>,
    pub operations: Vec<RevisionMigrationOperation>,
}

pub enum RevisionMigrationOperation {
    Identification(IdentificationMigration),
    Node(NodeMigration),
}

pub enum IdentificationMigration {
    ChangeName(Migration<String>),
    ChangeVendor(Migration<u16>),
    ChangeMachine(Migration<u16>),
}

pub enum ConfigPropertyMigration {
    UpdateNode(ResourceKind),
}

pub struct NodeMigration {
    kind: ResourceKind,
    path: String, // e.g 'diameter.target'
    op: NodeMigrationOperation,
}

pub enum ResourceKind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

pub enum NodeMigrationOperation<V> {
    Create { name: String, value: V },
    Remove { name: String },
    UpdateName(Migration<String>),
    UpdateType(Migration<V>),
    UpdateDescription(Migration<String>),
}

pub enum ConfigPropertyNodeMigration {
    Remove { name: String },
    Rename { migration: Migration<String> },

    ChangeName { field: String, new_name: String },
    ChangeType { field: String, new_type: () },
    ChangeDefault { field: String, new_type: () },
}

pub enum StatePropertyMigration {
    RemoveField { field: String },
    ChangeName { field: String, new_name: String },
    ChangeType { field: String, new_type: () },
}

pub enum MeasurementMigration {
    RemoveField { field: String },
    ChangeName { field: String, new_name: String },
    ChangeType { field: String, new_type: () },
}

pub struct Migration<T> {
    from: T,
    to: T,
}
