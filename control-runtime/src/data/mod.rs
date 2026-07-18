use serde::{Serialize, Serializer};

mod registry;
pub use registry::DataRegistry;
pub use registry::PropertyHandle;
pub use registry::MeasurementHandle;

mod recorder;
pub use recorder::DataRecorder;
pub use recorder::ConfigRecorderHandle;
pub use recorder::StateRecorderHandle;
pub use recorder::MachineEventRecorderHandle;
pub use recorder::LogRecorderHandle;

#[derive(Debug)]
pub struct DataStore {
    pub registry: DataRegistry,
    pub recorder: DataRecorder,
}

impl DataStore {
    pub(crate) fn new() -> Self {
        Self { 
            registry: DataRegistry::new(), 
            recorder: DataRecorder::new() 
        }
    }
}

impl Serialize for DataStore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        _ = serializer;
        todo!("TODO");
        // let mut state = serializer.serialize_struct("DataStore", 2)?;
        // state.serialize_field("registry", &self.registry)?;
        // state.serialize_field("recorder", &self.recorder)?;
        // state.end()
    }
}
