use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::machine::resource::Measurement;
use qitech_framework::machine::resource::StateProperty;
use qitech_lib::ethercat_hal::io::analog_input::physical::AnalogInputValue;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_framework::uom::Angle;
use qitech_framework::uom::ConstZero;
use qitech_framework::uom::angle::revolution;
use qitech_framework::uom::electric_potential::volt;

const ANALOG_INPUT_PORT: usize = 0;

pub struct TensionArm {
    // --- hardware ---
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- state ---
    zero: StateProperty<Option<Angle>>,

    // --- measurements ---
    angle: Measurement<Angle>,
}

impl TensionArm {
    pub fn new(
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        zero: StateProperty<Option<Angle>>,
        angle: Measurement<Angle>,
    ) -> Self {
        Self {
            device,
            zero,
            angle,
        }
    }

    pub fn angle(&self) -> anyhow::Result<Angle> {
        let raw = self.raw_angle()?;

        let zero = self.zero.get().unwrap_or(Angle::ZERO);

        if raw < zero {
            // We've wrapped around, so add a full revolution
            Ok(raw + Angle::new::<revolution>(1.0) - zero)
        } else {
            // Normal case
            Ok(raw - zero)
        }
    }

    pub fn zero(&mut self) {
        match self.raw_angle() {
            Ok(angle) => self.zero.set(Some(angle)),
            Err(_) => {
                // TODO: log error
                // tracing::error!("Failed to zero tension_arm angle {:?}", e)
            }
        }
    }
}

// --- helpers ---
impl TensionArm {
    fn raw_angle(&self) -> anyhow::Result<Angle> {
        let volts = self.get_volts()?;

        // 0V = 0deg 5V = 3600deg
        Ok(self.volts_to_angle(volts))
    }

    fn get_volts(&self) -> Result<f64, anyhow::Error> {
        // get the normalized value from the analog input
        let device = &*self.device.borrow();

        let Some(range) = device.analog_input_range() else {
            return Err(anyhow::anyhow!("No input range supplied"));
        };

        match device
            .get_analog_input(ANALOG_INPUT_PORT)?
            .get_physical(&range)
        {
            AnalogInputValue::Potential(v) => Ok(v.get::<volt>()),
            _ => panic!("Expected a potential value"),
        }
    }

    fn volts_to_angle(&self, volts: f64) -> Angle {
        // 0V = 0deg 5V = 3600deg
        // always wrap into 0..1 revolution
        Angle::new::<revolution>(volts / 5.0) % Angle::new::<revolution>(1.0)
    }
}
