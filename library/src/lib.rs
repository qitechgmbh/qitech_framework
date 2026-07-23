#[macro_use]
extern crate uom as uom_crate;

pub use qitech_framework_common::MachineIdentification;
pub use qitech_framework_common::MachineIdentificationUnique;

pub mod uom;
pub mod machine;

mod types;
pub use types::BoundsError;