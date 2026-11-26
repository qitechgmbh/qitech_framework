use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::Machine;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine_build;
use qitech_framework::run_with_tui;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::ethercat_hal::io::analog_output::AnalogCurrentOutputDevice;
use qitech_lib::ethercat_hal_devices::wago_modules::wago_750_554::Wago750_554;
use qitech_lib::units::ElectricCurrent;
use qitech_lib::units::electric_current::milliampere;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<Wago750_554Machine>();
    run_with_tui(config_rt, Default::default()).await.unwrap();
}

#[derive(Machine)]
pub struct Wago750_554Machine {
    wago750_554: Rc<RefCell<Wago750_554>>,
    outputs: [ConfigProperty<ElectricCurrent>; 2],
}

impl Machine for Wago750_554Machine {}

impl Wago750_554Machine {
    fn update_output(&mut self, port: usize) -> ActResult {
        let value = self.outputs[port].get();
        let mut wago750_554 = self.wago750_554.borrow_mut();

        wago750_554.set_current(port, value);
        Ok(())
    }
}

impl MachineBuild for Wago750_554Machine {
    #[machine_build(Wago750_554Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let wago750_554 = ctx.get_ethercat_device::<Wago750_554>(2)?;

        let output1 = ctx
            .config::<milliampere>("output1")
            .on_external_changed(|m: &mut Wago750_554Machine| m.update_output(0))
            .build()?;
        let output2 = ctx
            .config::<milliampere>("output2")
            .on_external_changed(|m: &mut Wago750_554Machine| m.update_output(1))
            .build()?;

        Ok(Self {
            wago750_554,
            outputs: [output1, output2],
        })
    }
}
