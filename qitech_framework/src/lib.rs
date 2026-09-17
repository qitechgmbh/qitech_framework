pub use qitech_framework_core::ScalarValue;
pub use qitech_framework_core::ident;
pub use qitech_framework_core::ident::DeviceHardwareIdentification;
pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineInstanceIdentification;
pub use qitech_framework_core::report::ConfigPropertyEvent;
pub use qitech_framework_core::report::ConfigPropertyWriteOutcome;
pub use qitech_framework_core::report::Constraints;
pub use qitech_framework_core::report::EtherCATDeviceMetadata;
pub use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::EventRecord;
pub use qitech_framework_core::report::MachinesReport;
pub use qitech_framework_core::report::RuntimeEvent;
pub use qitech_framework_core::report::RuntimeInitEvent;
pub use qitech_framework_core::report::RuntimeReport;
pub use qitech_framework_core::report::StatePropertyEvent;
pub use qitech_framework_core::request::RuntimeRequestKind;
pub use qitech_framework_core::schema::MachineSchema;
pub use qitech_framework_core::session;
pub use qitech_framework_core::vendors;
pub use qitech_framework_macros::*;
pub use qitech_lib::units;

mod controller;

pub mod machine;
mod resource;

pub mod runtime;

pub type ConfigPropertyEventRecord = EventRecord<ConfigPropertyEvent>;
pub type StatePropertyEventRecord = EventRecord<StatePropertyEvent>;

mod modbus;
mod service;

#[doc(hidden)]
/// exposed for proc macros
pub mod __private {
    pub use qitech_framework_core::ScalarValue;
    pub use qitech_framework_core::ScalarValueTypeMismatchError;
    pub use qitech_framework_core::report::ConstraintViolationError;
    pub use qitech_framework_core::report::Constraints;
    pub use qitech_framework_core::schema::MeasurementDefinition;
    pub use qitech_framework_core::schema::ScalarPropertyDefinition;

    pub use crate::machine::MachineDescriptor;
    pub use crate::resource::constraints::EnumConstraints;
    pub use crate::resource::conversion::PropertyAdapter;
    pub use crate::resource::conversion::PropertyType;
}
