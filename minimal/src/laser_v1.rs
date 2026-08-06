use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework::MachineIdentification;
use qitech_framework::WriteCapability;
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
    diameter_tolerance_upper: ConfigProperty<Length>,
    diameter_tolerance_lower: ConfigProperty<Length>,

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
            .on_external_write(|m: &mut Self| {
                m.diameter_target.forbid_external_write("single use only");
            })
            // .minimum(0.0)
            .register()?;

        let diameter_tolerance_lower = ctx
            .config::<millimeter>("diameter.tolerance.lower")
            .default(0.05)
            // .minimum(0.0)
            .register()?;

        let diameter_tolerance_upper = ctx
            .config::<millimeter>("diameter.tolerance.upper")
            .default(0.05)
            // .minimum(0.0)
            .register()?;

        Ok(Self {
            device,
            diameter_target,
            diameter_tolerance_upper,
            diameter_tolerance_lower,
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
