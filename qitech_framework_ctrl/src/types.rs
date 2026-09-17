use std::collections::BTreeMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::EventRecord;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::schema::FloatSemantic;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::schema::ScalarPropertyKind;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub type Swappable<T> = Arc<ArcSwap<T>>;

pub type SchemaRegistry = BTreeMap<MachineIdentification, MachineSchema>;
pub type MachineRegistry = BTreeMap<MachineInstanceIdentification, MachineEntry>;

pub type RuntimeReportSender = broadcast::Sender<Arc<RuntimeReport>>;
pub type RuntimeReportReceiver = broadcast::Receiver<Arc<RuntimeReport>>;

pub type RuntimeRequestSender = mpsc::Sender<(
    RuntimeRequestKind,
    oneshot::Sender<Result<(), RuntimeRequestError>>,
)>;

pub type RuntimeRequestReceiver = mpsc::Receiver<(
    RuntimeRequestKind,
    oneshot::Sender<Result<(), RuntimeRequestError>>,
)>;

pub type RuntimeRequestResponder = oneshot::Sender<Result<(), RuntimeRequestError>>;

#[derive(Debug)]
pub struct MachineEntry {
    pub updated_at: DateTime<Utc>,
    pub config_props: IndexMap<String, ConfigPropertyEntry>,
    pub state_props: IndexMap<String, StatePropertyEntry>,
    pub measurements: IndexMap<String, MeasurementEntry>,
}

#[derive(Debug)]
pub struct ConfigPropertyEntry {
    pub kind: ScalarPropertyKind,
    pub records: Vec<EventRecord<ConfigPropertyEvent>>,

    pub value: ScalarValue,
    pub default: ScalarValue,
    pub capability: OperationCapability,
    pub constraints: Constraints,
}

#[derive(Debug)]
pub struct StatePropertyEntry {
    pub kind: ScalarPropertyKind,
    pub records: Vec<EventRecord<ConfigPropertyEvent>>,
    pub value: ScalarValue,
}

#[derive(Debug)]
pub struct MeasurementEntry {
    pub label: String,
    pub value: Option<f64>,
    pub repr: FloatSemantic,
}
