use qitech_framework::Machine;
use qitech_framework::TuiConfiguration;
use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActErrorKind;
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
use qitech_lib::ethercat_hal_devices::wago_modules::wago_750_402::Wago750_402;
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
    wago750_402: Rc<RefCell<Wago750_402>>,
    inputs: [StateProperty<bool>; 4],
}

impl Machine for Wago750_402Machine {
    fn act(&mut self, _dt: Duration) -> ActResult {

        let wago750_402 = self.wago750_402.borrow_mut();

        for port in 0..wago750_402.get_port_count() {
            let value = wago750_402.get_input(port).map_err(|_err| ActError {
                kind: ActErrorKind::HardwareFault("Error reading port value!".to_string()),
                impact: ActErrorImpact::Ignore,
            })?;

            self.inputs[port].set(value);
        }

        Ok(())
    }
}

impl MachineBuild for Wago750_402Machine {
    #[machine_build(Wago750_402Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let wago750_402 = ctx.find_ethercat_device::<Wago750_402>(1)?;

        let input1 = ctx.state::<bool>("input1").build()?;
        let input2 = ctx.state::<bool>("input2").build()?;
        let input3 = ctx.state::<bool>("input3").build()?;
        let input4 = ctx.state::<bool>("input4").build()?;

        Ok(Self {
            wago750_402,
            inputs: [input1, input2, input3, input4],
        })
    }
}
