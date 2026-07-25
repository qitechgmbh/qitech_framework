use core::fmt::Display;
use core::fmt::Formatter;
use std::any::type_name;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::modbus::ModbusDevice;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::machine::Machine;
use crate::machine::Resources;
use crate::machine::error::CommandExecuteError;
use crate::machine::hardware::EtherCATDeviceIdentified;
use crate::machine::hardware::Hardware;
use crate::machine::hardware::ModbusDeviceIdentified;
use crate::machine::resource::CommandHandle;
use crate::machine::resource::ConfigProperty;
use crate::machine::resource::ConfigPropertyRegisterOptions;
use crate::machine::resource::EventEmitter;
use crate::machine::resource::Measurement;
use crate::machine::resource::MeasurementRegisterOptions;
use crate::machine::resource::StateProperty;
use crate::machine::resource::conversion::BoundedMeta;
use crate::machine::resource::conversion::Extract;
use crate::machine::resource::conversion::ScalarTypeWrapper;
use crate::machine::resource::conversion::TypeWrapper;
use crate::machine::resource::error::RegisterError;

pub struct BuildContext<'a> {
    ident: MachineIdentificationUnique,
    interface: Option<EtherCATThreadChannel>,
    resources: &'a mut Resources,
    hardware: Vec<Hardware>,
}

// --- resources ---
impl<'a> BuildContext<'a> {
    pub fn register_config<T>(
        &mut self,
        path: &'static str,
        options: ConfigPropertyRegisterOptions<T::Type>,
    ) -> BuildResult<ConfigProperty<T::Type>>
    where
        T: ScalarTypeWrapper + 'static,
        T::Type: Clone + Serialize + DeserializeOwned + BoundedMeta,
    {
        Ok(self
            .resources
            .config_properties
            .register::<T>(self.ident, path, options)?)
    }

    pub fn register_state<T>(
        &mut self,
        path: &'static str,
        initial_value: T::Type,
    ) -> BuildResult<StateProperty<T::Type>>
    where
        T: ScalarTypeWrapper,
        T::Type: Default,
    {
        Ok(self
            .resources
            .state_properties
            .register::<T>(self.ident, path, initial_value)?)
    }

    pub fn register_measurement<T>(
        &mut self,
        path: &'static str,
        options: MeasurementRegisterOptions<T::Type>,
    ) -> BuildResult<Measurement<T>>
    where
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + PartialOrd + Default,
    {
        Ok(self
            .resources
            .measurements
            .register::<T>(self.ident, path, options)?)
    }

    pub fn register_command<M, A>(
        &mut self,
        path: &'static str,
        execute: fn(&mut M, A) -> Result<(), CommandExecuteError>,
    ) -> BuildResult<CommandHandle>
    where
        M: Machine + 'static,
        A: serde::de::DeserializeOwned + 'static,
    {
        Ok(self
            .resources
            .commands
            .register(self.ident, path, execute)?)
    }

    pub fn register_event<T>(&mut self, path: &'static str) -> BuildResult<EventEmitter<T>>
    where
        T: Serialize,
    {
        Ok(self.resources.events.register::<T>(self.ident, path)?)
    }
}

// --- ethercat ---
impl BuildContext<'_> {
    pub fn get_ethercat_interface(&self) -> BuildResult<EtherCATThreadChannel> {
        self.interface
            .clone()
            .ok_or(BuildError::ExpectedEtherCATInterface)
    }

    pub fn get_ethercat_device<T>(&self, index: usize) -> BuildResult<Rc<RefCell<T>>>
    where
        T: EthercatDevice,
    {
        let Hardware::Ethercat(EtherCATDeviceIdentified { device, .. }) =
            self.hardware_at(index)?
        else {
            return Err(BuildError::ExpectedEtherCATDeviceAtIndex { index });
        };

        downcast_ecat_dev(index, device.clone())
    }

    pub fn find_ethercat_device_and_addr<T>(&self, role: u16) -> BuildResult<(Rc<RefCell<T>>, u16)>
    where
        T: EthercatDevice,
    {
        let (index, EtherCATDeviceIdentified { device, ident }) =
            self.find_ethercat_by_role(role)?;
        let device = downcast_ecat_dev(index, device.clone())?;
        Ok((device, ident.device_address))
    }

    pub fn find_ethercat_device<T>(&self, role: u16) -> BuildResult<Rc<RefCell<T>>>
    where
        T: EthercatDevice,
    {
        self.find_ethercat_device_and_addr::<T>(role)
            .map(|(device, _)| device)
    }

    pub fn find_ethercat_device_addr(&self, role: u16) -> BuildResult<u16> {
        self.find_ethercat_by_role(role)
            .map(|(_, hw)| hw.ident.device_address)
    }
}

// --- modbus ---
impl BuildContext<'_> {
    pub fn get_modbus_rtu_device<T>(&self, index: usize) -> BuildResult<Rc<RefCell<T>>>
    where
        T: 'static,
    {
        let Some(hardware) = self.hardware.get(index) else {
            return Err(BuildError::ExpectedHardwareAtIndex { index });
        };

        let Hardware::Modbus(ModbusDeviceIdentified { device }) = hardware else {
            return Err(BuildError::ExpectedSerialDeviceAtIndex { index });
        };

        downcast_modbus_dev(index, device.clone())
    }
}

// --- helpers ---
impl BuildContext<'_> {
    fn hardware_at(&self, index: usize) -> BuildResult<&Hardware> {
        self.hardware
            .get(index)
            .ok_or(BuildError::ExpectedHardwareAtIndex { index })
    }

    fn find_ethercat_by_role(&self, role: u16) -> BuildResult<(usize, &EtherCATDeviceIdentified)> {
        self.hardware
            .iter()
            .enumerate()
            .find_map(|(i, hw)| match hw {
                Hardware::Ethercat(identified) if identified.ident.role == role => {
                    Some((i, identified))
                }
                _ => None,
            })
            .ok_or(BuildError::ExpectedEtherCATDeviceWithRole { role })
    }
}

// --- utils ---
fn downcast_ecat_dev<T: 'static>(
    index: usize,
    device: Rc<RefCell<dyn EthercatDevice>>,
) -> BuildResult<Rc<RefCell<T>>> {
    if !device.borrow().as_any().is::<T>() {
        let expected = type_name::<T>();
        return Err(BuildError::DeviceTypeMismatch { index, expected });
    }
    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
}

fn downcast_modbus_dev<T: 'static>(
    index: usize,
    device: Rc<RefCell<dyn ModbusDevice>>,
) -> BuildResult<Rc<RefCell<T>>> {
    if !device.borrow().as_any().is::<T>() {
        let expected = type_name::<T>();
        return Err(BuildError::DeviceTypeMismatch { index, expected });
    }
    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
}

// --- errors ---
pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Debug)]
pub enum BuildError {
    // --- hardware errors ---
    ExpectedEtherCATInterface,
    ExpectedHardwareAtIndex {
        index: usize,
    },
    ExpectedEtherCATDeviceWithRole {
        role: u16,
    },
    ExpectedEtherCATDeviceAtIndex {
        index: usize,
    },
    ExpectedSerialDeviceAtIndex {
        index: usize,
    },
    DeviceTypeMismatch {
        index: usize,
        expected: &'static str,
    },
    // --- resource errors ---
    RegisterError(RegisterError),
}

impl From<RegisterError> for BuildError {
    fn from(value: RegisterError) -> Self {
        BuildError::RegisterError(value)
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedEtherCATInterface => {
                write!(f, "machine required a valid ethercat interface")
            }
            Self::ExpectedHardwareAtIndex { index } => {
                write!(f, "expected hardware at index {index}")
            }
            Self::ExpectedEtherCATDeviceWithRole { role } => {
                write!(f, "expected an ethercat device with role {role}")
            }
            Self::ExpectedEtherCATDeviceAtIndex { index } => {
                write!(f, "expected an ethercat device at index {index}")
            }
            Self::ExpectedSerialDeviceAtIndex { index } => {
                write!(f, "expected a serial device at index {index}")
            }
            Self::DeviceTypeMismatch { index, expected } => {
                write!(
                    f,
                    "device type mismatch at index {index}. Expected: {expected}"
                )
            }
            _ => todo!(),
        }
    }
}
