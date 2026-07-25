#[macro_use]
extern crate uom as uom_crate;

pub use qitech_framework_common::MachineIdentification;
pub use qitech_framework_common::MachineIdentificationUnique;

mod hardware;
pub mod machine;
// pub mod runtime;
pub mod uom;

mod types;
pub use types::BoundsError;
