use std::rc::Rc;
use std::cell::RefCell;
use qitech_lib::ethercat_hal::io::analog_input::physical::AnalogInputValue;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::angle::revolution;
use qitech_lib::units::electric_potential::volt;
use qitech_lib::units::f64::*;
use control_runtime::{Measurement, StateProperty};

const ANALOG_INPUT_PORT: usize = 0;

pub(super) struct TensionArm {
    // --- hardware ---
    pub analog_input: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- state ---
    pub zero: StateProperty<Angle, revolution>,
    
    /// was zeroed at least once
    pub zeroed: StateProperty<bool>,

    // --- measurements ---
    pub angle: Measurement<Angle, revolution>,
}

impl TensionArm {
    pub fn new(
        analog_input: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        zero: StateProperty<Angle, revolution>,
        zeroed: StateProperty<bool>,
        angle: Measurement<Angle, revolution>,
    ) -> Self {
        Self {
            analog_input,
            zero,
            zeroed,
            angle,
        }
    }

    fn volts_to_angle(&self, volts: f64) -> Angle {
        // 0V = 0deg 5V = 3600deg
        // always wrap into 0..1 revolution
        Angle::new::<revolution>(volts / 5.0) % Angle::new::<revolution>(1.0)
    }

    fn get_volts(&self) -> Result<f64, anyhow::Error> {
        // get the normalized value from the analog input
        let analog_input = &*self.analog_input.borrow();

        let range = match analog_input.analog_input_range() {
            Some(range) => range,
            None => return Err(anyhow::anyhow!("No input range supplied")),
        };

        let value = analog_input.get_analog_input(ANALOG_INPUT_PORT)?.get_physical(&range);
        match value {
            AnalogInputValue::Potential(v) => Ok(v.get::<volt>()),
            _ => panic!("Expected a potential value"),
        }
    }

    fn raw_angle(&self) -> anyhow::Result<Angle> {
        // get volts
        let volts = self.get_volts()?;

        // println!("volts: {volts}");

        // 0V = 0deg 5V = 3600deg
        Ok(self.volts_to_angle(volts))
    }

    pub fn get_angle(&self) -> anyhow::Result<Angle> {
        let raw = self.raw_angle()?;

        if raw < self.zero.get() {
            // We've wrapped around, so add a full revolution
            Ok(raw + Angle::new::<revolution>(1.0) - self.zero.get())
        } else {
            // Normal case
            Ok(raw - self.zero.get())
        }
    }

    pub fn zero(&mut self) {
        match self.raw_angle() {
            Ok(angle) => {
                self.zero.set(angle);
                self.zeroed.set(true);
            }
            Err(e) => {
                _ = e;
                // TODO: log error
                // tracing::error!("Failed to zero tension_arm angle {:?}", e)
            }
        }
    }
}
