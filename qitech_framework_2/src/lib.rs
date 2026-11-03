pub use qitech_framework_core::ScalarValue;
pub use qitech_framework_core::ident;
pub use qitech_framework_core::ident::MachineId;
pub use qitech_framework_core::link;
pub use qitech_framework_core::report::ConfigPropertyEvent;
pub use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::EventRecord;
pub use qitech_framework_core::report::MachinesReport;
pub use qitech_framework_core::report::RuntimeEvent;
pub use qitech_framework_core::report::RuntimeReport;
pub use qitech_framework_core::report::StatePropertyEvent;
pub use qitech_framework_core::request::RuntimeRequestKind;
pub use qitech_framework_core::schema::MachineSchema;
pub use qitech_lib::units;

mod descriptor;
pub use descriptor::MachineDescriptor;

pub mod system;
pub use system::System;

// mod ethercat;

pub mod machine_2;
pub mod modbus;
// pub mod machine;
// mod resource;

// pub mod runtime;
mod runtime_2;

// pub type ConfigPropertyEventRecord = EventRecord<ConfigPropertyEvent>;
// pub type StatePropertyEventRecord = EventRecord<StatePropertyEvent>;

// mod modbus;
