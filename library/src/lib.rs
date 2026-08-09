pub use qitech_framework_core::session;
pub use qitech_framework_macros::*;
pub use qitech_lib::units;

pub mod machine;
mod conversion;
mod resource;

mod runtime;
pub use runtime::Runtime;
pub use runtime::RuntimeConfiguration;

#[doc(hidden)]
pub mod __private {
    // --- used by proc macros ---
    pub use qitech_framework_core::ScalarValue;
    pub use qitech_framework_core::ScalarValueTypeMismatchError;
    pub use qitech_framework_core::report::ConstraintViolationError;
    pub use qitech_framework_core::report::Constraints;

    pub use crate::machine::MachineDescriptor;
    pub use crate::resource::constraints::EnumConstraints;
    pub use crate::conversion::PropertyAdapter;
    pub use crate::conversion::PropertyType;
}
