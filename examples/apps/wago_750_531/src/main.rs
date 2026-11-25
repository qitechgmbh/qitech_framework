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
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal_devices::wago_modules::wago_750_531::Wago750_531;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<Wago750_531Machine>();
    run_with_tui(config_rt, Default::default()).await.unwrap();
}

#[derive(Machine)]
pub struct Wago750_531Machine {
    wago750_531: Rc<RefCell<Wago750_531>>,
    leds: [ConfigProperty<bool>; 4],
}

impl Machine for Wago750_531Machine {}

impl Wago750_531Machine {
    fn update_led(&mut self, port: usize) -> ActResult {
        let value = self.leds[port].get();
        let mut wago750_531 = self.wago750_531.borrow_mut();

        wago750_531.set_output(port, value);
        Ok(())
    }
}

impl MachineBuild for Wago750_531Machine {
    #[machine_build(Wago750_531Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let wago750_531 = ctx.find_ethercat_device::<Wago750_531>(1)?;

        let led1 = ctx
            .config::<bool>("led1_on")
            .on_external_changed(|m: &mut Wago750_531Machine| m.update_led(0))
            .build()?;
        let led2 = ctx
            .config::<bool>("led2_on")
            .on_external_changed(|m: &mut Wago750_531Machine| m.update_led(1))
            .build()?;
        let led3 = ctx
            .config::<bool>("led3_on")
            .on_external_changed(|m: &mut Wago750_531Machine| m.update_led(2))
            .build()?;
        let led4 = ctx
            .config::<bool>("led4_on")
            .on_external_changed(|m: &mut Wago750_531Machine| m.update_led(3))
            .build()?;

        Ok(Self {
            wago750_531,
            leds: [led1, led2, led3, led4],
        })
    }
}
