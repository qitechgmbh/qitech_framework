use std::cell::RefCell;
use std::rc::Rc;

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
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el4008::EL4008;
use qitech_lib::ethercat_hal::io::analog_output::AnalogVoltageOutputDevice;
use qitech_lib::units::ElectricPotential;
use qitech_lib::units::electric_potential::volt;

#[tokio::main]
pub async fn main() {
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<BeckhoffEL4008Machine>();

    run_with_tui(config_rt, TuiConfiguration::default())
        .await
        .unwrap()
}

#[derive(Machine)]
pub struct BeckhoffEL4008Machine {
    el4008: Rc<RefCell<EL4008>>,
    voltages: [ConfigProperty<ElectricPotential>; 8],
}

impl Machine for BeckhoffEL4008Machine {}

impl BeckhoffEL4008Machine {
    fn update_port(&mut self, port: usize) -> ActResult {
        let voltage = self.voltages[port].get();
        self.el4008.borrow_mut().set_output(port, voltage);
        Ok(())
    }
}

impl MachineBuild for BeckhoffEL4008Machine {
    #[machine_build(BeckhoffEL4008Machine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let el4008 = ctx.find_ethercat_device::<EL4008>(1)?;

        let voltage1 = ctx
            .config::<volt>("voltage1")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(0))
            .build()?;

        let voltage2 = ctx
            .config::<volt>("voltage2")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(1))
            .build()?;

        let voltage3 = ctx
            .config::<volt>("voltage3")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(2))
            .build()?;

        let voltage4 = ctx
            .config::<volt>("voltage4")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(3))
            .build()?;

        let voltage5 = ctx
            .config::<volt>("voltage5")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(4))
            .build()?;

        let voltage6 = ctx
            .config::<volt>("voltage6")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(5))
            .build()?;

        let voltage7 = ctx
            .config::<volt>("voltage7")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(6))
            .build()?;

        let voltage8 = ctx
            .config::<volt>("voltage8")
            .on_external_changed(|m: &mut BeckhoffEL4008Machine| m.update_port(7))
            .build()?;

        Ok(Self {
            el4008,
            voltages: [
                voltage1, voltage2, voltage3, voltage4, voltage5, voltage6, voltage7, voltage8,
            ],
        })
    }
}
