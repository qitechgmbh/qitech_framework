use serde::{Serialize, Serializer};

mod registry;
pub use registry::DataRegistry;

mod recorder;
pub use recorder::DataRecorder;
pub use recorder::ConfigRecorderHandle;
pub use recorder::StateRecorderHandle;
pub use recorder::LogRecorderHandle;

mod property;
pub use property::*;

pub mod measurement;

pub type MachineMeasurementRegistry<'a> = measurement::Registry<0, 512>;
pub type MachineMeasurementReader<'a> = measurement::Reader<'a, 0, 512>;
pub type MachineMeasurementWriteHandle = measurement::WriteHandle;

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

