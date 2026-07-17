use serde::{Serialize, Serializer};

mod registry;
pub(crate) use registry::DataRegistry;
pub(crate) use registry::PropertyHandle;
pub(crate) use registry::MeasurementHandle;

mod recorder;
pub(crate) use recorder::DataRecorder;
pub(crate) use recorder::ConfigRecorderHandle;
pub(crate) use recorder::StateRecorderHandle;
pub(crate) use recorder::MachineEventRecorderHandle;
pub(crate) use recorder::LogRecorderHandle;

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
