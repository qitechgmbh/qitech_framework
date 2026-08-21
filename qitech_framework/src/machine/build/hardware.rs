use std::any::type_name;
use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework_core::report::error::BuildError;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::modbus::ModbusDevice;

use crate::machine::BuildContext;
use crate::machine::hardware::EtherCATDeviceIdentified;
use crate::machine::hardware::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;

// --- ethercat ---
impl BuildContext<'_> {
    pub fn get_ethercat_interface(&self) -> Result<EtherCATThreadChannel, BuildError> {
        self.ethercat_interface
            .clone()
            .ok_or(BuildError::ExpectedEtherCATInterface)
    }

    pub fn get_ethercat_device<T>(&self, index: usize) -> Result<Rc<RefCell<T>>, BuildError>
    where
        T: EthercatDevice,
    {
        let Hardware::Ethercat(EtherCATDeviceIdentified { device, .. }) =
            self.hardware_at(index)?
        else {
            return Err(BuildError::ExpectedEtherCATDeviceAtIndex { index });
        };

        downcast_ecat_dev(device.clone()).ok_or(cast_error_at::<T>(index))
    }

    pub fn find_ethercat_device_and_addr<T>(
        &self,
        role: u16,
    ) -> Result<(Rc<RefCell<T>>, u16), BuildError>
    where
        T: EthercatDevice,
    {
        let (
            index,
            EtherCATDeviceIdentified {
                device,
                info: ident,
            },
        ) = self.find_ethercat_by_role(role)?;
        let device = downcast_ecat_dev(device.clone()).ok_or(cast_error_at::<T>(index))?;
        Ok((device, ident.device_address))
    }

    pub fn find_ethercat_device_by_type<T>(&self) -> Result<Rc<RefCell<T>>, BuildError>
    where
        T: EthercatDevice,
    {
        let found = self.hardware.iter().find_map(|hw| if let Hardware::Ethercat(identified) = hw {
            downcast_ecat_dev(identified.device.clone())
        } else {
            None
        });

        found.ok_or(BuildError::MissingEtherCATDevice { type_name: std::any::type_name::<T>().to_string() })
    }

    pub fn find_ethercat_device<T>(&self, role: u16) -> Result<Rc<RefCell<T>>, BuildError>
    where
        T: EthercatDevice,
    {
        self.find_ethercat_device_and_addr::<T>(role)
            .map(|(device, _)| device)
    }

    pub fn find_ethercat_device_addr(&self, role: u16) -> Result<u16, BuildError> {
        self.find_ethercat_by_role(role)
            .map(|(_, hw)| hw.info.device_address)
    }
}

// --- modbus ---
impl BuildContext<'_> {
    pub fn get_modbus_rtu_device<T>(&self, index: usize) -> Result<Rc<RefCell<T>>, BuildError>
    where
        T: 'static,
    {
        let Some(hardware) = self.hardware.get(index) else {
            return Err(BuildError::ExpectedHardwareAtIndex { index });
        };

        let Hardware::ModbusRTU(ModbusRTUDeviceIdentified { device, .. }) = hardware else {
            return Err(BuildError::ExpectedSerialDeviceAtIndex { index });
        };

        downcast_modbus_dev(device.clone()).ok_or(cast_error_at::<T>(index))
    }
}

// --- helpers ---
impl BuildContext<'_> {
    fn hardware_at(&self, index: usize) -> Result<&Hardware, BuildError> {
        self.hardware
            .get(index)
            .ok_or(BuildError::ExpectedHardwareAtIndex { index })
    }

    fn find_ethercat_by_role(
        &self,
        role: u16,
    ) -> Result<(usize, &EtherCATDeviceIdentified), BuildError> {
        self.hardware
            .iter()
            .enumerate()
            .find_map(|(i, hw)| match hw {
                Hardware::Ethercat(identified) if identified.info.role == role => {
                    Some((i, identified))
                }
                _ => None,
            })
            .ok_or(BuildError::ExpectedEtherCATDeviceWithRole { role })
    }
}

// --- utils ---
fn downcast_ecat_dev<T: 'static>(
    device: Rc<RefCell<dyn EthercatDevice>>,
) -> Option<Rc<RefCell<T>>> {
    if !device.borrow().as_any().is::<T>() {
        return None;
    }

    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Some(Rc::from_raw(raw_concrete_ptr)) }
}

fn downcast_modbus_dev<T: 'static>(
    device: Rc<RefCell<dyn ModbusDevice>>,
) -> Option<Rc<RefCell<T>>> {
    if !device.borrow().as_any().is::<T>() {
        return None
    }

    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Some(Rc::from_raw(raw_concrete_ptr)) }
}

fn cast_error_at<T: 'static>(index: usize) -> BuildError {
    let expected = type_name::<T>().to_string();
    BuildError::DeviceTypeMismatch { index, expected }
}
