use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use tokio_modbus::Address;
use tokio_modbus::ExceptionCode;
use tokio_modbus::Quantity;
use tokio_modbus::Request;
use tokio_modbus::Response;
use tokio_modbus::SlaveId;

mod rtu;
pub use rtu::ModbusRTUBusConfig;
pub use rtu::ModbusRtuBus;
pub use rtu::ModbusRtuPort;

mod dev;

struct LaserV1Loader {
    laser: ModbusSlot,
}

pub struct ModbusSlot {}

// --- device ---
pub struct ModbusDevice {
    pub(crate) slave_id: SlaveId,

    /// Cyclic reads, derived from the slots assigned to this device.
    pub(crate) bulk_reads: Vec<BulkRead>,

    /// One-off requests, sent in order ahead of the bulk reads.
    pub(crate) queued_requests: VecDeque<QueuedRequest>,
}

impl ModbusDevice {
    pub(crate) fn new(slave_id: SlaveId) -> Self {
        Self {
            slave_id,
            bulk_reads: Vec::new(),
            queued_requests: VecDeque::new(),
        }
    }

    pub(crate) fn add_bulk_read(
        &mut self,
        kind: BulkReadKind,
        address: Address,
        quantity: Quantity,
        interval: Duration,
    ) {
        self.bulk_reads.push(BulkRead {
            kind,
            address,
            quantity,
            interval,
            last_read: None,
        });
    }

    pub(crate) fn enqueue(&mut self, request: QueuedRequest) {
        self.queued_requests.push_back(request);
    }
}

// --- bulk reads ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BulkReadKind {
    Coils,
    DiscreteInputs,
    HoldingRegisters,
    InputRegisters,
}

#[derive(Debug, Clone)]
pub(crate) struct BulkRead {
    pub(crate) kind: BulkReadKind,
    pub(crate) address: Address,
    pub(crate) quantity: Quantity,
    pub(crate) interval: Duration,
    pub(crate) last_read: Option<Instant>,
}

impl BulkRead {
    pub(crate) fn is_due(&self, now: Instant) -> bool {
        self.last_read
            .is_none_or(|last| now.duration_since(last) >= self.interval)
    }

    pub(crate) fn request(&self) -> Request<'static> {
        match self.kind {
            BulkReadKind::Coils => Request::ReadCoils(self.address, self.quantity),
            BulkReadKind::DiscreteInputs => {
                Request::ReadDiscreteInputs(self.address, self.quantity)
            }
            BulkReadKind::HoldingRegisters => {
                Request::ReadHoldingRegisters(self.address, self.quantity)
            }
            BulkReadKind::InputRegisters => {
                Request::ReadInputRegisters(self.address, self.quantity)
            }
        }
    }
}

// --- queued requests ---
/// Identifies a queued request. Assigned by the runtime, which keeps the callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RequestId(u64);

impl RequestId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QueuedRequest {
    pub(crate) id: RequestId,
    pub(crate) request: Request<'static>,
}

// --- bus messages ---
/// Sent from the runtime to a bus.
#[derive(Debug, Clone)]
pub(crate) struct ModbusBusRequest {
    pub(crate) slave_id: SlaveId,
    pub(crate) request: QueuedRequest,
}

/// Sent from a bus to the runtime.
#[derive(Debug)]
pub(crate) enum ModbusBusEvent {
    Enabled,
    Disabled,
    Response(ModbusDeviceResponse),
}

#[derive(Debug)]
pub(crate) struct ModbusDeviceResponse {
    pub(crate) slave_id: SlaveId,
    pub(crate) source: ResponseSource,
    pub(crate) result: Result<Response, ModbusRequestError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseSource {
    /// Index into the device's `bulk_reads`.
    BulkRead(usize),
    Request(RequestId),
}

#[derive(Debug)]
pub(crate) enum ModbusRequestError {
    BusDisabled,
    UnknownDevice,
    Exception(ExceptionCode),
    Transport(tokio_modbus::Error),
}
