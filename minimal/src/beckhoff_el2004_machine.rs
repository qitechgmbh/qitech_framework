use std::{cell::RefCell, rc::Rc, time::Instant};

use qitech_framework::machine::{MachineIdentification, ActResult, BuildContext, BuildResult, ConfigProperty, Machine, MachineBuild, MachineDescriptor};
use qitech_lib::ethercat_hal::{devices::beckhoff_modules::el2004::EL2004, io::digital_output::DigitalOutputDevice};

pub struct EL2004Machine {
    leds: [ConfigProperty<bool>; 4],
    el2004: Rc<RefCell<EL2004>>,
}

impl EL2004Machine {

    fn update_led(&mut self, port: usize) -> ActResult {
        let value = self.leds[port].get();

        self
            .el2004
            .borrow_mut()
            .set_output(port, value);

        Ok(())
    }

    fn on_led1_changed(&mut self) -> ActResult {
        self.update_led(0)
    }

    fn on_led2_changed(&mut self) -> ActResult {
        self.update_led(1)
    }

    fn on_led3_changed(&mut self) -> ActResult {
        self.update_led(2)
    }

    fn on_led4_changed(&mut self) -> ActResult {
        self.update_led(3)
    }
}

impl Machine for EL2004Machine {

    fn act(&mut self, _now: Instant) -> ActResult {
        Ok(())
    }
}

impl MachineBuild for EL2004Machine {

    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let el2004 = ctx.find_ethercat_device::<EL2004>(1)?;

        let led1_on = ctx
            .config::<bool>("led1_on")
            .default(false)
            .on_external_changed(EL2004Machine::on_led1_changed)
            .build()?;

        let led2_on = ctx
            .config::<bool>("led2_on")
            .default(false)
            .on_external_changed(EL2004Machine::on_led2_changed)
            .build()?;

        let led3_on = ctx
            .config::<bool>("led3_on")
            .default(false)
            .on_external_changed(EL2004Machine::on_led3_changed)
            .build()?;

        let led4_on = ctx
            .config::<bool>("led4_on")
            .default(false)
            .on_external_changed(EL2004Machine::on_led4_changed)
            .build()?;

        Ok(Self {
            el2004,
            leds: [led1_on, led2_on, led3_on, led4_on],
        })
    }
}

impl MachineDescriptor for EL2004Machine {

    const SCHEMA: &'static str = include_str!("../schemas/beckhoff_el2004_machine.yaml");

    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        machine_id: 0x0a,
        vendor_id: 1,
    };
}
