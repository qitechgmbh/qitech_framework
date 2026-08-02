#[cfg(feature = "schema")]
pub mod schema;

// exposes with_uom!() for macro calls that operate on uom units
// generates the macro from the units.toml file using build.rs
include!(concat!(env!("OUT_DIR"), "/with_uom.rs"));

mod value;
pub use value::ScalarValue;
pub use value::ScalarValueKind;

pub mod ident;
pub mod report;
pub mod request;
pub mod session;

pub mod vendors {
    include!(concat!(env!("OUT_DIR"), "/vendors.rs"));

    pub use generated::Entry;
    pub const QITECH: Entry = generated::QITECH;

    pub const fn contains_id(id: u16) -> bool {
        get_name(id).is_some()
    }

    pub fn contains_name(name: &str) -> bool {
        get_id(name).is_some()
    }

    pub const fn get_name(id: u16) -> Option<&'static str> {
        generated::get_name(id)
    }

    pub fn get_id(name: &str) -> Option<u16> {
        generated::get_id(name)
    }
}
