pub use qitech_framework_core::session;
pub use qitech_framework_macros::*;
pub use qitech_lib::units;

pub mod machine;
mod resource;

mod runtime;
pub use runtime::Runtime;
pub use runtime::RuntimeConfiguration;

#[doc(hidden)]
/// exposed for proc macros
pub mod __private {
    pub use qitech_framework_core::ScalarValue;
    pub use qitech_framework_core::ScalarValueTypeMismatchError;
    pub use qitech_framework_core::report::ConstraintViolationError;
    pub use qitech_framework_core::report::Constraints;
    pub use qitech_framework_core::schema::ScalarPropertyDefinition;
    pub use qitech_framework_core::schema::MeasurementDefinition;

    pub use crate::machine::MachineDescriptor;
    pub use crate::resource::constraints::EnumConstraints;
    pub use crate::resource::conversion::PropertyAdapter;
    pub use crate::resource::conversion::PropertyType;
}

pub fn x() {

}