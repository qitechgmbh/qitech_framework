use tokio::sync::oneshot;
use serde::{Deserialize, Serialize};
use control_core::{MachineIdentificationUnique, ScalarValue};

/// Request targetted at the runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeRequest {
    MutateConfig {
        identification: MachineIdentificationUnique,
        name: String,
        value: ScalarValue,
    },
    InvokeCommand {
        identification: MachineIdentificationUnique,
        name: String,
        data: String,  
    },
    Restart,
}

#[derive(Debug)]
pub struct RuntimeTransaction {
    pub uuid: TransactionId,
    pub request: RuntimeRequest,
    pub response: oneshot::Sender<Result<(), String>>,
}

pub type TransactionId = u64;