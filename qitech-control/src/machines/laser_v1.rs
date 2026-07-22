use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use anyhow::anyhow;

use qitech_lib::units::ConstZero;
use qitech_lib::units::{Length, length::millimeter};
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::{LaserDevice, LaserError};

use control_runtime::{
    Machine, MachineActError, MachineActResult, MachineBuild, MachineBuildError, MachineBuildContext, MachineIdentification,
};

use control_runtime::machine::{ConstrainedConfigProperty, Measurement, StateProperty};

#[machine("laser_v1")]
pub struct LaserV1 {
    // --- hardware ---
    device: Rc<RefCell<LaserDevice>>,

    // -- config ---
    diameter_target: ConstrainedConfigProperty<millimeter>,
    diameter_tolerance_upper: ConstrainedConfigProperty<millimeter>,
    diameter_tolerance_lower: ConstrainedConfigProperty<millimeter>,

    // --- state ---
    in_tolerance: StateProperty<bool>,

    // --- measurements ---
    diameter:   Measurement<millimeter>,
    diameter_x: Measurement<Option<millimeter>>,
    diameter_y: Measurement<Option<millimeter>>,
    roundness:  Measurement<Option<f64>>,

    // -- misc ---
    last_request: Instant,
}

// laser_v1.config

impl MachineBuild for LaserV1 {
    fn build(mut ctx: MachineBuildContext<'_>) -> Result<Self, MachineBuildError> {
        // --- hardware
        let device = ctx.get_serial_device::<LaserDevice>(0)?;

        // TODO: define schema parse into resolve config
        // let diameter_target = ctx.config(schemas::laser_v1::config::diameter.target);
        // let DiameterCondif = ConfigPropertyCOnfig {
        //     name: "diameter.target",
        //     default: Length::new::<millimeter>(1.74),
        //     lower_bound: Length::ZERO,
        // }; schemas::laser_v1::config.diameter.target

        let diameter_target = ctx
            .config("diameter.target", Length::new::<millimeter>(1.75))
            .with_lower_bound(Length::ZERO)
            .register()?;

        let diameter_tolerance_upper = ctx
            .config("diameter.tolerance.lower", Length::new::<millimeter>(0.05))
            .with_lower_bound(Length::ZERO)
            .register()?;

        let diameter_tolerance_lower = ctx
            .config("diameter.tolerance.upper", Length::new::<millimeter>(0.05))
            .with_lower_bound(Length::ZERO)
            .register()?;

        Ok(Self {
            device,
            diameter_target,
            diameter_tolerance_upper,
            diameter_tolerance_lower,
            in_tolerance: ctx.state("in_tolerance").register()?,
            diameter: ctx.measurement("diameter").register()?,
            diameter_x: ctx.measurement("diameter_x").register()?,
            diameter_y: ctx.measurement("diameter_y").register()?,
            roundness: ctx.measurement("roundness").register()?,
            last_request: Instant::now(),
        })
    }
}

impl Machine for LaserV1 {
    fn act(&mut self) -> MachineActResult {
        self.update_device()?;

        if let Some(m) = self.device.borrow().measurement.clone() {
            fn convert(value: u16) -> Length {
                Length::new::<millimeter>(value as f64 / 1000.0)
            }

            self.diameter.set(convert(m.diameter));
            self.diameter_x.set(Some(convert(m.x_axis)));
            self.diameter_y.set(Some(convert(m.y_axis)));
        }

        let roundness = self.compute_roundness();
        self.roundness.set(roundness);

        let in_tolerance = self.compute_in_tolerance();
        self.in_tolerance.set(in_tolerance);

        Ok(())
    }
}

impl LaserV1 {
    pub const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: 1,
        machine: 6,
    };

    fn update_device(&mut self) -> MachineActResult {
        let mut laser = self.device.borrow_mut();

        if let Err(e) = laser.handle_response()
            && let Some(laser_error) = e.downcast_ref::<LaserError>()
            && let LaserError::IoErr() = laser_error {
            return Err(MachineActError {
                recoverable: false,
                error: anyhow!("Physical hardware I/O broke."),
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

    /// Roundness = min(x, y) / max(x, y)
    fn compute_roundness(&mut self) -> Option<f64> {
        let (Some(x), Some(y)) = (
            self.diameter_x.get_as::<millimeter>(),
            self.diameter_y.get_as::<millimeter>(),
        ) else {
            return None;
        };

        if x > 0.0 && y > 0.0 {
            let roundness = f64::min(x, y) / f64::max(x, y);
            Some(roundness)
        } else if x == 0.0 && y == 0.0 {
            Some(0.0)
        } else {
            None
        }
    }

    /// Calculates if the current diameter is inside of the tolerance
    fn compute_in_tolerance(&mut self) -> bool {
        const DIAMETER_EPSILON: f64 = 0.0001; // in mm

        // early return true if the diameter is 0 to prevent warning happening before start
        if self.diameter.get_as::<millimeter>() < DIAMETER_EPSILON {
            return true;
        }

        let target = self.diameter_target.get();
        let top = target + self.diameter_tolerance_upper.get();
        let bottom = target - self.diameter_tolerance_lower.get();

        self.diameter.get() > top || self.diameter.get() < bottom
    }
}
