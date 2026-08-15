use crate::Lockable;
use crate::MachineRegistry;
use crate::RuntimeReportReceiver;
use crate::RuntimeRequestSender;
use crate::SchemaRegistry;
use crate::Swappable;

// mod clickhouse;

#[derive(Debug, Clone)]
pub enum ModuleError {}

pub struct ModuleContext {
    pub schemas: Swappable<SchemaRegistry>,
    pub machines: Lockable<MachineRegistry>,
    pub report_rx: RuntimeReportReceiver,
    pub request_tx: RuntimeRequestSender,
}

pub trait Module {
    fn run(self, ctx: ModuleContext) -> impl Future<Output = ()> + Send;
}

// pub trait Storage: Send + Sync {
//     async fn load(&self, request: LoadRequest) -> Result<Data, StorageError>;
//     async fn save(&self, request: SaveRequest) -> Result<(), StorageError>;
// }
