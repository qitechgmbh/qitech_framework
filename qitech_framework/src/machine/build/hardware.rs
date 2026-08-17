use std::any::Any;
use std::any::type_name;
use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework_core::report::error::BuildError;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::xtrem::XtremDevice;
use qitech_lib::xtrem::XtremProbe;

use crate::machine::BuildContext;
use crate::machine::hardware::EtherCATDeviceIdentified;
use crate::machine::hardware::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::machine::hardware::XtremDeviceIdentified;

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

        downcast_dev(index, device.clone(), EthercatDevice::as_any)
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
        let device = downcast_dev(index, device.clone(), EthercatDevice::as_any)?;
        Ok((device, ident.device_address))
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

        downcast_dev(index, device.clone(), ModbusDevice::as_any)
    }
}

// --- xtrem ---
impl BuildContext<'_> {
    pub fn get_xtrem_device<T>(&self, index: usize) -> Result<Rc<RefCell<T>>, BuildError>
    where
        T: 'static,
    {
        let Hardware::Xtrem(XtremDeviceIdentified { device, .. }) = self.hardware_at(index)? else {
            return Err(BuildError::ExpectedXtremDeviceAtIndex { index });
        };

        downcast_dev(index, device.clone(), XtremDevice::as_any)
    }

    /// What the discovery sweep learned about the module at `index` — serial, device id, and
    /// unicast address.
    pub fn get_xtrem_probe(&self, index: usize) -> Result<&XtremProbe, BuildError> {
        let Hardware::Xtrem(XtremDeviceIdentified { probe, .. }) = self.hardware_at(index)? else {
            return Err(BuildError::ExpectedXtremDeviceAtIndex { index });
        };

        Ok(probe)
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
/// Narrow an `Rc<RefCell<dyn Trait>>` down to its concrete type.
///
/// Every device trait exposes its own inherent `as_any`, so the type check is handed in as
/// `as_any` rather than expressed as a bound — that keeps one implementation covering EtherCAT,
/// Modbus, and XTREM instead of a near-identical copy per trait.
fn downcast_dev<D, T>(
    index: usize,
    device: Rc<RefCell<D>>,
    as_any: impl FnOnce(&D) -> &dyn Any,
) -> Result<Rc<RefCell<T>>, BuildError>
where
    D: ?Sized,
    T: 'static,
{
    if !as_any(&device.borrow()).is::<T>() {
        let expected = type_name::<T>().to_string();
        return Err(BuildError::DeviceTypeMismatch { index, expected });
    }
    let raw_trait_ptr = Rc::into_raw(device);
    let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
    unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
}
