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
pub(crate) fn open_bus(config: &XtremConfig) -> Result<XtremBusHandle, anyhow::Error> {
    XtremBus::open(config.bus.clone())
}

/// Broadcast a serial-number read and collect every module that answers within `window`.
pub(crate) fn discover(
    handle: &XtremBusHandle,
    window: Duration,
) -> Result<Vec<XtremProbe>, anyhow::Error> {
    get_async_runtime().block_on(discovery::discover(handle, window))
}
