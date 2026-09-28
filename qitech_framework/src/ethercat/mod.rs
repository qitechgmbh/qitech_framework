use qitech_lib::units::ElectricCurrent;
use qitech_lib::ethercat_hal::io::analog_input::AnalogCurrentInputDevice;

pub struct AnalogCurrentInput<D> {
    device: D,
    port: usize,
}

impl<D> AnalogCurrentInput<D>
where
    D: AnalogCurrentInputDevice,
{
    pub fn get_current(&self) -> Option<ElectricCurrent> {
        self.device.get_current(self.port)
    }

    pub fn get_current_relative(&self) -> Option<f64> {
        self.device.get_current_relative(self.port)
    }

    pub fn minimum_current(&self) -> ElectricCurrent {
        self.device.get_minimum_current()
    }

    pub fn maximum_current(&self) -> ElectricCurrent {
        self.device.get_maximum_current()
    }
}
