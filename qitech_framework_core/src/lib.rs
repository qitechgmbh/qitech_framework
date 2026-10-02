#[cfg(feature = "schema")]
pub mod schema;

// exposes with_uom!() for macro calls that operate on uom units
// generates the macro from the units.toml file using build.rs
include!(concat!(env!("OUT_DIR"), "/with_uom.rs"));

mod value;
pub use value::ScalarValue;
pub use value::ScalarValueKind;
pub use value::ScalarValueTypeMismatchError;

pub mod ident;
pub mod link;
pub mod report;

pub mod request;
pub use request::RuntimeRequest;
pub use request::RuntimeRequestError;
pub use request::RuntimeRequestKind;
pub use request::RuntimeResponse;
