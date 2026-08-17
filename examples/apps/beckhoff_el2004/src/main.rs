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
use qitech_framework::TuiConfiguration;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2004::EL2004;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<MyMachine>();

    run_with_tui(config_rt, TuiConfiguration::default())
        .await.unwrap()
}

#[derive(Machine)]
pub struct MyMachine {
    el2004: Rc<RefCell<EL2004>>,
    leds: [ConfigProperty<bool>; 4],
}

impl Machine for MyMachine {}

impl MyMachine {
    fn update_led(&mut self, port: usize) -> ActResult {
        let value = self.leds[port].get();
        self.el2004.borrow_mut().set_output(port, value);
        Ok(())
    }
}

impl MachineBuild for MyMachine {
    #[machine_build(MyMachine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let el2004 = ctx.find_ethercat_device::<EL2004>(1)?;

        let led1_on = ctx
            .config::<bool>("led1_on")
            .on_external_changed(|m: &mut MyMachine| m.update_led(1))
            .build()?;

        let led2_on = ctx
            .config::<bool>("led2_on")
            .on_external_changed(|m: &mut MyMachine| m.update_led(1))
            .build()?;

        let led3_on = ctx
            .config::<bool>("led3_on")
            .on_external_changed(|m: &mut MyMachine| m.update_led(2))
            .build()?;

        let led4_on = ctx
            .config::<bool>("led4_on")
            .on_external_changed(|m: &mut MyMachine| m.update_led(3))
            .build()?;

        Ok(Self {
            el2004,
            leds: [led1_on, led2_on, led3_on, led4_on],
        })
    }
}
