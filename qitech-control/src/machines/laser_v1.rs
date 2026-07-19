use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};

use qitech_lib::units::{Length, length::millimeter};
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::{LaserDevice, LaserError};

use control_runtime::{
    Machine, MachineActError, MachineActResult, MachineBuild, 
    MachineBuildError, MachineBuilder,
};

use control_runtime::machine::{ConfigProperty, StateProperty, Measurement};

pub struct LaserV1 {
    // --- hardware ---
    device: Rc<RefCell<LaserDevice>>,

    // -- config ---
    diameter_target: ConfigProperty<millimeter>,
    diameter_tolerance_upper: ConfigProperty<millimeter>,
    diameter_tolerance_lower: ConfigProperty<millimeter>,

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

impl MachineBuild for LaserV1 {
    fn build(mut builder: MachineBuilder<'_>) -> Result<Self, MachineBuildError> {
        // --- hardware
        let device = builder.get_serial_device_by_index::<LaserDevice>(0)?;

        Ok(Self {
            device,
            diameter_target: builder
                .config("diameter.tolerance.lower", Length::new::<millimeter>(0.05))
                .register()?,

            diameter_tolerance_upper: builder
                .config("diameter.tolerance.lower", Length::new::<millimeter>(0.05))
                .register()?,

            diameter_tolerance_lower: builder
                .config("diameter.tolerance.lower", Length::new::<millimeter>(0.05))
                .register()?,
                
            in_tolerance: builder.state("in_tolerance").register()?,
            diameter: builder.measurement("diameter").register()?,
            diameter_x: builder.measurement("diameter_x").register()?,
            diameter_y: builder.measurement("diameter_y").register()?,
            roundness: builder.measurement("roundness").register()?,
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

    fn update_device(&mut self) -> MachineActResult {
        let mut laser = self.device.borrow_mut();

        if let Err(e) = laser.handle_response()
            && let Some(laser_error) = e.downcast_ref::<LaserError>()
            && let LaserError::IoErr() = laser_error {
            return Err(MachineActError {
                recoverable: false,
                message: "Physical hardware I/O broke.".to_string(),
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
