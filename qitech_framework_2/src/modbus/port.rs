use std::time::Duration;

use tokio_serial::DataBits;
use tokio_serial::Parity;
use tokio_serial::StopBits;

#[derive(Debug, Clone)]
pub struct ModbusRtuLineConfig {
    pub(crate) path: String,
    pub(crate) baud_rate: u32,
    pub(crate) data_bits: DataBits,
    pub(crate) parity: Parity,
    pub(crate) stop_bits: StopBits,
    pub(crate) response_timeout: Duration,
}

/*
pub(crate) fn serial(&self, path: &Path) -> SerialPortBuilder {
    tokio_serial::new(path.to_string_lossy(), self.baud_rate)
        .data_bits(self.data_bits)
        .parity(self.parity)
        .stop_bits(self.stop_bits)
}
*/
