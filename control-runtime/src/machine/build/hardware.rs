use std::any::type_name;
use std::{cell::RefCell, rc::Rc};

use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::modbus::ModbusDevice;

use crate::MachineBuildError;
use crate::machine::{Hardware, IdentifiedEthercat, IdentifiedModbus};
use crate::machine::build::BuildResult;
use super::BuildContext;

// --- ethercat ---
impl BuildContext<'_> {
    pub fn get_ethercat_interface(&self) -> BuildResult<EtherCATThreadChannel> {
        self.ethercat_interface
            .clone()
            .ok_or(MachineBuildError::ExpectedEtherCATInterface)
    }

    pub fn get_ethercat_device<T>(&self, index: usize) -> BuildResult<Rc<RefCell<T>>>
    where 
        T: EthercatDevice
    {
        let Hardware::Ethercat(IdentifiedEthercat { device, .. }) = self.hardware_at(index)? else {
            return Err(MachineBuildError::ExpectedEtherCATDeviceAtIndex { index });
        };

        downcast_ecat_dev(index, device.clone())
    }

    pub fn find_ethercat_device_and_addr<T>(&self, role: u16) -> BuildResult<(Rc<RefCell<T>>, u16)>
    where 
        T: EthercatDevice
    {
        let (index, IdentifiedEthercat { device, ident }) = self.find_ethercat_by_role(role)?;
        let device = downcast_ecat_dev(index, device.clone())?;
        Ok((device, ident.device_address)) 
    }

    pub fn find_ethercat_device<T>(&self, role: u16) -> BuildResult<Rc<RefCell<T>>> 
    where 
        T: EthercatDevice
    {
        self.find_ethercat_device_and_addr::<T>(role).map(|(device, _)| device)
    }

    pub fn find_ethercat_device_addr(&self, role: u16) -> BuildResult<u16> {
        self.find_ethercat_by_role(role).map(|(_, hw)| hw.ident.device_address)
    }
}

// --- serial ---
impl BuildContext<'_> {
    pub fn get_serial_device<T>(&self, index: usize) -> BuildResult<Rc<RefCell<T>>> 
    where 
        T: 'static
    {
        let Some(hardware) = self.hardware.get(index) else {
            return Err(MachineBuildError::ExpectedHardwareAtIndex { index });
        };

        let Hardware::Modbus(IdentifiedModbus { device }) = hardware else {
            return Err(MachineBuildError::ExpectedSerialDeviceAtIndex { index });
        };

        downcast_modbus_dev(index, device.clone())
    }
}

// --- helpers ---
impl BuildContext<'_> {
    fn hardware_at(&self, index: usize) -> BuildResult<&Hardware> {
        self.hardware
            .get(index)
            .ok_or(MachineBuildError::ExpectedHardwareAtIndex { index })
    }

    fn find_ethercat_by_role(&self, role: u16) -> BuildResult<(usize, &IdentifiedEthercat)> {
        self.hardware
            .iter()
            .enumerate()
            .find_map(|(i, hw)| match hw {
                Hardware::Ethercat(identified) if identified.ident.role == role => {
                    Some((i, identified))
                }
                _ => None,
            })
            .ok_or(MachineBuildError::ExpectedEtherCATDeviceWithRole { role })
    }
}

fn downcast_ecat_dev<T: 'static>(
    index: usize, 
    device: Rc<RefCell<dyn EthercatDevice>>
) -> BuildResult<Rc<RefCell<T>>>{
    if!device.borrow().as_any().is::<T>(){
        let expected = type_name::<T>();
        return Err(MachineBuildError::DeviceTypeMismatch {
            index,expected
        });
    }
    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
}

fn downcast_modbus_dev<T: 'static>(
    index: usize, 
    device: Rc<RefCell<dyn ModbusDevice>>
) -> BuildResult<Rc<RefCell<T>>>{
    if!device.borrow().as_any().is::<T>(){
        let expected = type_name::<T>();
        return Err(MachineBuildError::DeviceTypeMismatch {
            index,expected
        });
    }
    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
}
