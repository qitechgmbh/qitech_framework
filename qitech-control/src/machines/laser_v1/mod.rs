use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use qitech_lib::{
    modbus::{
        ModbusDevice,
        devices::qitech_laser::{LaserDevice, LaserError},
    },
    units::{Length, length::millimeter},
};
use control_runtime::{
    ConfigProperty, Machine, MachineActError, MachineActResult, MachineBuild, MachineBuildError,
    MachineBuilder, Measurement, StateProperty,
};

pub struct DiameterConfig {
    target: ConfigProperty<Length, millimeter>,
    tolerance_higher: ConfigProperty<Length, millimeter>,
    tolerance_lower: ConfigProperty<Length, millimeter>,
}

pub struct LaserV1 {
    // --- hardware
    device: Rc<RefCell<LaserDevice>>,

    // -- config ---
    config_diameter: DiameterConfig,

    // --- state ---
    in_tolerance: StateProperty<bool>,

    // --- measurements ---
    diameter: Measurement<Length, millimeter>,
    diameter_x: Measurement<Option<Length>, millimeter>,
    diameter_y: Measurement<Option<Length>, millimeter>,
    roundness: Measurement<Option<f64>>,

    // -- misc ---
    last_request: Instant,
}

impl MachineBuild for LaserV1 {
    fn build(mut builder: MachineBuilder<'_>) -> Result<Self, MachineBuildError> {
        // --- hardware
        let device = builder.get_serial_device_by_index::<LaserDevice>(0)?;

        // --- properties ---
        let config_diameter = DiameterConfig {
            target: builder
                .config("diameter.target", Length::new::<millimeter>(1.75))
                .register(),

            tolerance_higher: builder
                .config("diameter.tolerance.higher", Length::new::<millimeter>(0.05))
                .register(),

            tolerance_lower: builder
                .config("diameter.tolerance.lower", Length::new::<millimeter>(0.05))
                .register(),
        };

        return Ok(Self {
            device,
            config_diameter,
            in_tolerance: builder.state("in_tolerance").register(),
            diameter: builder.measurement("diameter").register(),
            diameter_x: builder.measurement("diameter_x").register(),
            diameter_y: builder.measurement("diameter_y").register(),
            roundness: builder.measurement("roundness").register(),
            last_request: Instant::now(),
        });
    }
}

impl Machine for LaserV1 {
    fn act(&mut self) -> MachineActResult {
        self.update_device();

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

        let target = self.config_diameter.target.get();
        let top = target + self.config_diameter.tolerance_higher.get();
        let bottom = target - self.config_diameter.tolerance_lower.get();

        self.diameter.get() > top || self.diameter.get() < bottom
    }

    fn update_device(&mut self) -> MachineActResult {
        let mut laser = self.device.borrow_mut();
        if let Err(e) = laser.handle_response() {
            if let Some(laser_error) = e.downcast_ref::<LaserError>() {
                if let LaserError::IoErr() = laser_error {
                    return Err(MachineActError {
                        recoverable: false,
                        message: "Physical hardware I/O broke.".to_string(),
                    });
                }
            }
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
