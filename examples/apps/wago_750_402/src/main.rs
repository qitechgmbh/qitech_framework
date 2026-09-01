use qitech_framework::Machine;
use qitech_framework::TuiConfiguration;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine_build;
use qitech_framework::run_with_tui;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_354::Wago750_354;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_402::Wago750_402;
use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<Wago750_402Machine>();
    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap();
}

#[derive(Machine)]
pub struct Wago750_402Machine {
    coupler: Rc<RefCell<Wago750_354>>,
    inputs: [StateProperty<bool>; 4],
}

impl Machine for Wago750_402Machine {
    fn act(&mut self, _dt: Duration) -> ActResult {
        let coupler = self.coupler.borrow();
        let wago402 = Self::wago402(&coupler);
        for (port, state) in self.inputs.iter_mut().enumerate() {
            state.set(wago402.get_input(port).unwrap_or(false));
        }
        Ok(())
    }
}

impl Wago750_402Machine {
    fn wago402(coupler: &Wago750_354) -> &Wago750_402 {
        coupler.slot_devices[0]
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<Wago750_402>()
            .unwrap()
    }
}

impl MachineBuild for Wago750_402Machine {
    #[machine_build(Wago750_402Machine)]
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
            let slot_dev = c.slot_devices[0].as_ref().expect("No device in slot 0");
            assert!(
                slot_dev.as_any().is::<Wago750_402>(),
                "Slot 0 is not a Wago 750-402"
            );
        }
        let in1 = ctx.state::<bool>("in1").build()?;
        let in2 = ctx.state::<bool>("in2").build()?;
        let in3 = ctx.state::<bool>("in3").build()?;
        let in4 = ctx.state::<bool>("in4").build()?;
        Ok(Self {
            coupler,
            inputs: [in1, in2, in3, in4],
        })
    }
}
