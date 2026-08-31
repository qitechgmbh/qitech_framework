use qitech_framework::Machine;
use qitech_framework::TuiConfiguration;
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
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_354::Wago750_354;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_531::Wago750_531;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use std::cell::RefCell;
use std::rc::Rc;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<Wago750_531Machine>();
    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap();
}

#[derive(Machine)]
pub struct Wago750_531Machine {
    coupler: Rc<RefCell<Wago750_354>>,
    leds: [ConfigProperty<bool>; 4],
}

impl Machine for Wago750_531Machine {}

impl Wago750_531Machine {
    /// Safe to unwrap because we validated the type in build().
    fn wago531(coupler: &mut Wago750_354) -> &mut Wago750_531 {
        coupler.slot_devices[0]
            .as_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Wago750_531>()
            .unwrap()
    }

    fn update_led(&mut self, port: usize) -> ActResult {
        let value = self.leds[port].get();
        let mut coupler = self.coupler.borrow_mut();
        Self::wago531(&mut coupler).set_output(port, value);
        Ok(())
    }
}

impl MachineBuild for Wago750_531Machine {
    #[machine_build(Wago750_531Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let (coupler, coupler_addr) = ctx.find_ethercat_device_and_addr::<Wago750_354>(0)?;
        let channel = ctx.get_ethercat_interface()?;

        {
            let mut c = coupler.borrow_mut();
            let modules = Wago750_354::initialize_modules(channel.clone(), coupler_addr)
                .expect("Failed to initialize coupler modules");
            for module in modules {
                c.set_module(module);
            }
            c.init_slot_modules(channel, coupler_addr);

            // Validate type once at build time
            let slot_dev = c.slot_devices[0]
                .as_ref()
                .expect("No device in slot 0: is the 750-531 registered in init_slot_modules?");
            assert!(
                slot_dev.as_any().is::<Wago750_531>(),
                "Slot 0 is not a Wago 750-531"
            );
        }

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
            coupler,
            leds: [led1, led2, led3, led4],
        })
    }
}

