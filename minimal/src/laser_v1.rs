use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework::MachineIdentification;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::MachineInterface;
use qitech_framework::machine::error::ActError;
use qitech_framework::machine::error::ActErrorKind;
use qitech_framework::machine::error::ActResult;
use qitech_framework::machine::error::BuildError;
use qitech_framework::resource::ConfigProperty;
use qitech_framework::vendors;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserError;
use qitech_lib::units::Length;
use qitech_lib::units::length::millimeter;

pub struct LaserV1 {
    // --- hardware ---
    device: Rc<RefCell<LaserDevice>>,

    // -- config ---
    diameter_target: ConfigProperty<Length>,
    diameter_target_default: ConfigProperty<Length>,
    diameter_target_enabled: ConfigProperty<bool>,
    diameter_target_min: ConfigProperty<Option<Length>>,
    diameter_target_max: ConfigProperty<Option<Length>>,

    // -- misc ---
    last_request: Instant,
}

impl MachineInterface for LaserV1 {
    const SCHEMA: &'static str = include_str!("../schemas/laser_v1.yaml");
}

impl MachineBuild for LaserV1 {
    fn build(ctx: &mut BuildContext) -> Result<Self, BuildError> {
        let device = ctx.get_modbus_rtu_device::<LaserDevice>(0)?;

        let diameter_target = ctx
            .config::<millimeter>("diameter.target")
            .default(1.75)
            .register()?;

        let diameter_target_default = ctx
            .config::<millimeter>("diameter.target_default")
            .default(1.75)
            .on_external_write(|m: &mut Self| {
                let default = *m.diameter_target_default.get_ref();
                m.diameter_target.set_default(default);
            })
            .register()?;

        let diameter_target_enabled = ctx
            .config::<bool>("diameter.target_enabled")
            .default(true)
            .on_external_write(|m: &mut Self| match *m.diameter_target_enabled.get_ref() {
                true => m.diameter_target.allow_external_write(),
                false => m.diameter_target.forbid_external_write("single use only"),
            })
            .register()?;

        let diameter_target_min = ctx
            .config::<Option<millimeter>>("diameter.target_min")
            .default(None)
            .on_external_write(|m: &mut Self| {
                let mut c = m.diameter_target.constraints().clone();
                c.set_min(*m.diameter_target_min.get_ref());
                m.diameter_target.set_constraints(c);
            })
            .register()?;

        let diameter_target_max = ctx
            .config::<Option<millimeter>>("diameter.target_max")
            .default(None)
            .on_external_write(|m: &mut Self| {
                let mut c = m.diameter_target.constraints().clone();
                c.set_max(*m.diameter_target_max.get_ref());
                m.diameter_target.set_constraints(c);
            })
            .register()?;

        Ok(Self {
            device,
            diameter_target,
            diameter_target_default,
            diameter_target_enabled,
            diameter_target_min,
            diameter_target_max,
            last_request: Instant::now(),
        })
    }
}

impl Machine for LaserV1 {
    fn act(&mut self) -> ActResult {
        self.update_device()?;
        Ok(())
    }
}

impl LaserV1 {
    pub const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 6,
    };

    fn update_device(&mut self) -> ActResult {
        let mut laser = self.device.borrow_mut();

        if let Err(e) = laser.handle_response()
            && let Some(laser_error) = e.downcast_ref::<LaserError>()
            && let LaserError::IoErr() = laser_error
        {
            return Err(ActError {
                recoverable: false,
                kind: ActErrorKind::HardwareFault("Physical hardware I/O broke.".into()),
            });
        }

        let now = Instant::now();
        if now.duration_since(self.last_request) > Duration::from_millis(6) {
            self.last_request = now;
            let res = laser.send_next_request();

            if res.is_err() {
                println!("send_next_request {:?}", res);
            }
        }

        Ok(())
    }
}
