// --- export common qitech lib parts ---
pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::report::WriteCapability;
pub use qitech_framework_core::session;
pub use qitech_framework_macros::EnumProperty;
pub use qitech_lib::units;

pub mod machine;
pub mod resource;

mod runtime;
pub use runtime::Runtime;
pub use runtime::RuntimeConfiguration;

pub mod prelude {
    pub use crate::MachineIdentification;
    pub use crate::MachineIdentificationUnique;
    pub use crate::machine::BuildContext;
    pub use crate::machine::Machine;
    pub use crate::machine::MachineBuild;
    pub use crate::machine::MachineDescriptor;
    pub use crate::machine::RemoteProperty;
    pub use crate::machine::SubscribeContext;
    pub use crate::machine::SubscribeError;
    pub use crate::machine::SubscribeResult;
    pub use crate::machine::error::ActError;
    pub use crate::machine::error::ActErrorKind;
    pub use crate::machine::error::ActResult;
    pub use crate::machine::error::BuildError;
    pub use crate::machine::error::BuildResult;
    pub use crate::resource::ConfigProperty;
    pub use crate::resource::Measurement;
    pub use crate::resource::StateProperty;
    pub use crate::runtime::Runtime;
    pub use crate::runtime::RuntimeConfiguration;
    pub use crate::units::length::centimeter;
    pub use crate::units::length::meter;
    pub use crate::units::length::millimeter;
    // --- uom units
    pub use crate::units::*;
}

#[doc(hidden)]
pub mod __private {
    pub use qitech_framework_core::ScalarValue;
    pub use qitech_framework_core::ScalarValueTypeMismatchError;
    pub use qitech_framework_core::report::ConstraintViolationError;
    pub use qitech_framework_core::report::Constraints;
}
