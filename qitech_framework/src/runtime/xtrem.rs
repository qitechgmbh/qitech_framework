use std::time::Duration;

use qitech_lib::common::get_async_runtime;
use qitech_lib::xtrem::ScaleMode;
use qitech_lib::xtrem::XtremBus;
use qitech_lib::xtrem::XtremBusHandle;
use qitech_lib::xtrem::XtremDevice;
use qitech_lib::xtrem::XtremProbe;
use qitech_lib::xtrem::XtremScale;
use qitech_lib::xtrem::discovery;

use crate::runtime::config::XtremConfig;

/// How a device type is constructed once the bus is up and discovery has found its module.
///
/// `ModbusDevice::new` is part of that trait, which is what lets `modbus_rtu_device::<D>()` stay
/// generic. `XtremScale` has no equivalent — it needs a `&XtremBusHandle` that must exist first
/// and a `&XtremProbe` only discovery can produce — so the contract lives here instead of in
/// `qitech_lib`, and `xtrem_device::<D>()` stays generic all the same.
pub trait XtremDeviceBuild: XtremDevice + Sized {
    fn build(
        bus: &XtremBusHandle,
        probe: &XtremProbe,
        mode: ScaleMode,
    ) -> Result<Self, anyhow::Error>;
}

impl XtremDeviceBuild for XtremScale {
    fn build(
        bus: &XtremBusHandle,
        probe: &XtremProbe,
        mode: ScaleMode,
    ) -> Result<Self, anyhow::Error> {
        Ok(XtremScale::from_probe(bus, probe, mode))
    }
}

/// Bind the shared socket and start the receive task.
///
/// One bus serves every module on the subnet. A socket per device would not work: the modules
/// reply to the port in register `0700h` rather than to the source port of the request, so all
/// traffic converges on one local port and is demultiplexed by `ID_O`.
pub(crate) fn open_bus(config: &XtremConfig) -> Result<XtremBusHandle, anyhow::Error> {
    XtremBus::open(config.bus.clone())
}

/// Broadcast a serial-number read and collect every module that answers within `window`.
///
/// `discovery::discover` is async, but `Runtime::init` is not. Blocking here is sound because
/// init always runs on a plain `std::thread` spawned by `run_with_tui` / `run_with_hub`, never
/// inside a Tokio context.
pub(crate) fn discover(
    handle: &XtremBusHandle,
    window: Duration,
) -> Result<Vec<XtremProbe>, anyhow::Error> {
    get_async_runtime().block_on(discovery::discover(handle, window))
}
