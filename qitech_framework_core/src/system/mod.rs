mod parser;
mod schema;

#[cfg(test)]
mod tests;

pub use schema::Binding;
pub use schema::ModbusRtuBus;
pub use schema::ModbusRtuDevice;
pub use schema::Parity;
pub use schema::SystemSchema;

pub use crate::schema::Issue;
pub use crate::schema::SchemaError as SystemError;
