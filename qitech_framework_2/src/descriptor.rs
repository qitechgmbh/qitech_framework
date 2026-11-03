/// Provides static information for a machine.
pub trait MachineDescriptor {
    /// The machine's KDL schema, embedded at compile time.
    const SCHEMA: &'static str;
}
