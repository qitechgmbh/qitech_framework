#[cfg(feature = "schema")]
pub mod schema;

#[cfg(feature = "schema")]
pub use schema::MachineSchema;

// // exposes with_uom!() for macro calls that operate on uom units
// // generates the macro from the units.toml file using build.rs
// #[cfg(feature = "uom")]
// include!(concat!(env!("OUT_DIR"), "/with_uom.rs"));

mod ident;
pub use ident::MachineIdentification;
pub use ident::MachineIdentificationUnique;
pub use ident::DeviceIdentificationIdentified;
pub use ident::DeviceIdentification;
pub use ident::DeviceMachineIdentification;
pub use ident::DeviceHardwareIdentification;
pub use ident::DeviceHardwareIdentificationEthercat;
pub use ident::DeviceHardwareIdentificationSerial;

mod machine;
pub use machine::MachinesReport;
pub use machine::MachineConfigMutation;
pub use machine::MachineStateMutation;
pub use machine::MachineMeasurementVec;
pub use machine::MachineMeasurement;
pub use machine::MachineCommandCall;
pub use machine::MachineEvent;

mod runtime;
pub use runtime::RuntimeRequest;
pub use runtime::RuntimeRequestKind;
pub use runtime::RuntimeReport;
pub use runtime::RuntimeReportData;
pub use runtime::RuntimeEvent;
pub use runtime::RuntimeEventKind;
pub use runtime::RuntimeState;

mod logs;
pub use logs::LogRecord;
pub use logs::LogLevel;
pub use logs::LogOrigin;

mod types;
pub use types::OperationOrigin;
pub use types::OperationResult;
pub use types::ScalarValue;
pub use types::ScalarValueKind;

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
