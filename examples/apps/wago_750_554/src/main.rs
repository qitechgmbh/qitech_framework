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
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_554::Wago750_554;
use qitech_lib::ethercat_hal::io::analog_output::{AnalogOutputDevice, AnalogOutputOutput};
use std::cell::RefCell;
use std::rc::Rc;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<Wago750_554Machine>();
    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap();
}

#[derive(Machine)]
pub struct Wago750_554Machine {
    coupler: Rc<RefCell<Wago750_354>>,
    wago554: Box<Wago750_554>,
    outputs: [ConfigProperty<f32>; 2],
}

impl Machine for Wago750_554Machine {}

impl Wago750_554Machine {
    fn update_output(&mut self, port: usize) -> ActResult {
        let value = self.outputs[port].get();
        self.wago554
            .set_output(port, AnalogOutputOutput(value.clamp(0.0, 1.0)));
        Ok(())
    }
}

impl MachineBuild for Wago750_554Machine {
    #[machine_build(Wago750_554Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let (coupler, coupler_addr) = ctx.find_ethercat_device_and_addr::<Wago750_354>(0)?;
        let channel = ctx.get_ethercat_interface()?;

        let wago554 = {
            let mut c = coupler.borrow_mut();
            let modules = Wago750_354::initialize_modules(channel.clone(), coupler_addr)
                .expect("Failed to initialize coupler modules");
            for module in modules {
                c.set_module(module);
            }
            c.init_slot_modules(channel, coupler_addr);

            let slot = c
                .slot_devices
                .iter()
                .position(|s| {
                    s.as_ref().map_or(false, |d| {
                        d.as_any().downcast_ref::<Wago750_554>().is_some()
                    })
                })
                .expect("No Wago 750-554 found on the bus");

            let slot_dev = c.slot_devices[slot]
                .take()
                .expect("No device in slot");
            slot_dev
                .into_any()
                .downcast::<Wago750_554>()
                .expect("Slot is not a Wago 750-554")
        };

        let ao1 = ctx
            .config::<f32>("ao1")
            .on_external_changed(|m: &mut Wago750_554Machine| m.update_output(0))
            .build()?;
        let ao2 = ctx
            .config::<f32>("ao2")
            .on_external_changed(|m: &mut Wago750_554Machine| m.update_output(1))
            .build()?;

        Ok(Self {
            coupler,
            wago554,
            outputs: [ao1, ao2],
        })
    }
}
