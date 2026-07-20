mod resource;
mod conversion;

include!(concat!(env!("OUT_DIR"), "/with_uom.rs"));
pub(crate) use with_uom;


mod property;