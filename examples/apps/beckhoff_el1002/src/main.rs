use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::Machine;
use qitech_framework::TuiConfiguration;
use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact::Ignore;
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
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el1002::EL1002;
use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<BeckhoffEL1002Machine>();

    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap()
}

#[derive(Machine)]
pub struct BeckhoffEL1002Machine {
    el1002: Rc<RefCell<EL1002>>,
    inputs: [StateProperty<bool>; 2],
}

impl Machine for BeckhoffEL1002Machine {
    fn act(&mut self, _dt: std::time::Duration) -> ActResult {
        let el1002 = self.el1002.borrow();

        for port in 0..el1002.get_port_count() {
            let input = el1002.get_input(port).map_err(|_err| ActError {
                kind: ActErrorKind::Custom("This should actually always work!".to_string()),
                impact: Ignore,
            })?;

            self.inputs[port].set(input);
        }

        Ok(())
    }
}

impl MachineBuild for BeckhoffEL1002Machine {
    #[machine_build(BeckhoffEL1002Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let el1002 = ctx.find_ethercat_device::<EL1002>(4)?;

        let input1 = ctx.state::<bool>("input1").build()?;

        let input2 = ctx.state::<bool>("input2").build()?;

        Ok(Self {
            el1002,
            inputs: [input1, input2],
        })
    }
}
