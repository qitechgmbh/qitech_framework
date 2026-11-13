use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::Machine;
use qitech_framework::TuiConfiguration;
use qitech_framework::machine::*;
use qitech_framework::machine_build;
use qitech_framework::run_with_tui;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::ethercat_hal::io::analog_input::AnalogCurrentInputDevice;
use qitech_lib::ethercat_hal_devices::beckhoff_modules::el3021::EL3021;
use qitech_lib::units::ElectricCurrent;
use qitech_lib::units::electric_current::milliampere;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<BeckhoffEL3021Machine>();

    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap()
}

#[derive(Machine)]
pub struct BeckhoffEL3021Machine {
    el3021: Rc<RefCell<EL3021>>,
    current: Measurement<Option<ElectricCurrent>>,
}

impl Machine for BeckhoffEL3021Machine {
    fn act(&mut self, _dt: std::time::Duration) -> ActResult {
        let el3021 = self.el3021.borrow();

        let current = el3021.get_current(0).ok();
        self.current.set(current);

        Ok(())
    }
}

impl MachineBuild for BeckhoffEL3021Machine {
    #[machine_build(BeckhoffEL3021Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let el3021 = ctx.find_ethercat_device::<EL3021>(3)?;

        let current = ctx.measurement::<Option<milliampere>>("current").build()?;

        Ok(Self { el3021, current })
    }
}
