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
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_455::Wago750_455;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_554::Wago750_554;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::analog_output::{AnalogOutputDevice, AnalogOutputOutput};
use std::cell::RefCell;
use std::rc::Rc;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<AnalogLoopbackMachine>();
    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap();
}

#[derive(Machine)]
pub struct AnalogLoopbackMachine {
    coupler: Rc<RefCell<Wago750_354>>,
    wago554: Box<Wago750_554>,
    wago455: Box<Wago750_455>,
    ao: [ConfigProperty<f32>; 2],
    ai: [ConfigProperty<f32>; 4],
}

impl Machine for AnalogLoopbackMachine {
    fn act(&mut self) -> ActResult {
        for port in 0..4 {
            if let Ok(input) = self.wago455.get_input(port) {
                let ma = 4.0 + input.normalized * 16.0;
                self.ai[port].set(ma);
            }
        }
        Ok(())
    }
}

impl AnalogLoopbackMachine {
    fn update_output(&mut self, port: usize) -> ActResult {
        let value = self.ao[port].get();
        self.wago554
            .set_output(port, AnalogOutputOutput(value.clamp(0.0, 1.0)));
        Ok(())
    }
}

impl MachineBuild for AnalogLoopbackMachine {
    #[machine_build(AnalogLoopbackMachine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let (coupler, coupler_addr) = ctx.find_ethercat_device_and_addr::<Wago750_354>(0)?;
        let channel = ctx.get_ethercat_interface()?;

        let (wago554, wago455) = {
            let mut c = coupler.borrow_mut();
            let modules = Wago750_354::initialize_modules(channel.clone(), coupler_addr)
                .expect("Failed to initialize coupler modules");
            for module in modules {
                c.set_module(module);
            }
            c.init_slot_modules(channel, coupler_addr);

            let ao_slot = c
                .slot_devices
                .iter()
                .position(|s| {
                    s.as_ref().map_or(false, |d| {
                        d.as_any().downcast_ref::<Wago750_554>().is_some()
                    })
                })
                .expect("No Wago 750-554 found on the bus");

            let ai_slot = c
                .slot_devices
                .iter()
                .position(|s| {
                    s.as_ref().map_or(false, |d| {
                        d.as_any().downcast_ref::<Wago750_455>().is_some()
                    })
                })
                .expect("No Wago 750-455 found on the bus");

            let dev554 = c.slot_devices[ao_slot]
                .take()
                .expect("No device in AO slot")
                .into_any()
                .downcast::<Wago750_554>()
                .expect("AO slot is not a Wago 750-554");

            let dev455 = c.slot_devices[ai_slot]
                .take()
                .expect("No device in AI slot")
                .into_any()
                .downcast::<Wago750_455>()
                .expect("AI slot is not a Wago 750-455");

            (dev554, dev455)
        };

        let ao1 = ctx
            .config::<f32>("ao1")
            .on_external_changed(|m: &mut AnalogLoopbackMachine| m.update_output(0))
            .build()?;
        let ao2 = ctx
            .config::<f32>("ao2")
            .on_external_changed(|m: &mut AnalogLoopbackMachine| m.update_output(1))
            .build()?;

        let ai1 = ctx.config::<f32>("ai1_mA").build()?;
        let ai2 = ctx.config::<f32>("ai2_mA").build()?;
        let ai3 = ctx.config::<f32>("ai3_mA").build()?;
        let ai4 = ctx.config::<f32>("ai4_mA").build()?;

        Ok(Self {
            coupler,
            wago554,
            wago455,
            ao: [ao1, ao2],
            ai: [ai1, ai2, ai3, ai4],
        })
    }
}
