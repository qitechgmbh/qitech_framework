use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    rc::Rc,
};
use serde::Serialize;
use anyhow::anyhow;
use qitech_lib::{
    ethercat_hal::{
        EtherCATThreadChannel,
        devices::{EthercatDevice, downcast_rc_refcell},
    },
    modbus::ModbusDevice,
};
use control_core::{LogOrigin, MachineIdentificationUnique};

use crate::DataStore;
use crate::data::{LogRecorderHandle, MachineEventRecorderHandle};
use crate::machine::config::{Bounded, Bounds};
use crate::machine::{BoundedConfigProperty, ConfigProperty, Hardware, Measurement, MeasurementStatistics, StateProperty};

pub trait MachineBuild: Sized {
    fn build(builder: MachineBuilder<'_>) -> Result<Self, MachineBuildError>;
}

pub struct MachineBuilder<'a> {
    ident: MachineIdentificationUnique,
    hardware: Vec<Hardware>,
    ethercat_interface: Option<EtherCATThreadChannel>,

    // data
    data_store: &'a mut DataStore,
    // registered items (for detecting duplicates)
    // registered_config_properties: heapless::Vec<&'static str, 64>,
    // registered_events: heapless::Vec<&'static str, 64>
}

// base
impl<'a> MachineBuilder<'a> {
    pub fn new(
        ident: MachineIdentificationUnique,
        hardware: Vec<Hardware>,
        ethercat_interface: Option<EtherCATThreadChannel>,
        data_store: &'a mut DataStore,
    ) -> Self {
        Self {
            ident,
            hardware,
            ethercat_interface,
            data_store,
        }
    }

    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }
}

// hardware
impl MachineBuilder<'_> {
    pub fn get_ethercat_interface(&self) -> anyhow::Result<EtherCATThreadChannel> {
        match &self.ethercat_interface {
            Some(v) => Ok(v.clone()),
            None => Err(anyhow!(
                "No Ethercat Interface was supplied, but is required to setup Machine"
            )),
        }
    }

    pub fn try_get_ethercat_device_by_index<T>(
        &self,
        index: usize,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error>
    where
        T: EthercatDevice,
    {
        let hw = self.hardware.get(index);
        let hw = match hw {
            Some(hw) => hw,
            None => return Err(anyhow::anyhow!("index {} not found in hardware", index)),
        };

        let identified_ethercat = match hw {
            Hardware::Ethercat(rc_ecat) => rc_ecat,
            _ => {
                return Err(anyhow::anyhow!(
                    "index {} not an ethercat device in hardware",
                    index
                ));
            }
        };

        downcast_rc_refcell::<T>(identified_ethercat.device.clone())
    }

    pub fn try_get_ethercat_meta_by_role(&self, role: u16) -> Result<u16, anyhow::Error> {
        for i in 0..self.hardware.len() {
            let hardware = self.hardware.get(i).expect("try_get_ethercat_device_by_role failed to get hardware even though i is in range of len??????");
            match hardware {
                Hardware::Ethercat(identified_ethercat) => {
                    if identified_ethercat.ident.role == role {
                        return Ok(identified_ethercat.ident.device_address);
                    }
                    continue;
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!(
            "index {} not an ethercat device in hardware",
            role
        ))
    }

    pub fn downcast_serial_rc_refcell<T: 'static>(
        dev: Rc<RefCell<dyn ModbusDevice>>,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error> {
        // Check if the inner type is actually T
        let is_t = dev.borrow().as_any().is::<T>();
        if !is_t {
            return Err(anyhow::anyhow!("Type mismatch in hardware downcast"));
        }
        // Since we verified the type above, we can use raw pointers.
        let raw_trait_ptr = Rc::into_raw(dev);
        // We cast the fat pointer to a thin pointer of the concrete RefCell<T>
        let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
        unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
    }

    pub fn get_serial_device_by_index<T: 'static>(
        &self,
        index: usize,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error> {
        let hw = self.hardware.get(index).unwrap().clone();
        match hw {
            Hardware::Modbus(identified_modbus) => {
                Self::downcast_serial_rc_refcell::<T>(identified_modbus.hw)
            }
            _ => Err(anyhow::anyhow!(
                "index {} not an modbus device in hardware",
                index
            )),
        }
    }

    pub fn get_ethercat_device_and_addr<T>(
        &self,
        role: u16,
    ) -> Result<(Rc<RefCell<T>>, u16), anyhow::Error>
    where
        T: EthercatDevice,
    {
        for i in 0..self.hardware.len() {
            let hardware = self.hardware.get(i).expect("try_get_ethercat_device_by_role failed to get hardware even though i is in range of len??????");
            match hardware {
                Hardware::Ethercat(identified_ethercat) => {
                    if identified_ethercat.ident.role == role {
                        let res = downcast_rc_refcell::<T>(identified_ethercat.device.clone())?;
                        return Ok((res, identified_ethercat.ident.device_address));
                    }
                    continue;
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!(
            "index {} not an ethercat device in hardware",
            role
        ))
    }

    pub fn get_ethercat_device<T>(
        &self,
        role: u16,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error>
    where
        T: EthercatDevice,
    {
        for i in 0..self.hardware.len() {
            let hardware = self.hardware.get(i).expect("try_get_ethercat_device_by_role failed to get hardware even though i is in range of len??????");
            match hardware {
                Hardware::Ethercat(identified_ethercat) => {
                    if identified_ethercat.ident.role == role {
                        return downcast_rc_refcell::<T>(identified_ethercat.device.clone());
                    }
                    continue;
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!(
            "index {} not an ethercat device in hardware",
            role
        ))
    }
}

// data
impl<'a> MachineBuilder<'a> {
    pub fn config<'b, T, U>(
        &'b mut self,
        name: &'static str,
        default_value: T,
    ) -> ConfigPropertyBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Copy + Default,
    {
        ConfigPropertyBuilder {
            root: self,
            name,
            default_value,
            initial_value: default_value,
            _marker: PhantomData,
        }
    }

    pub fn config_bounded<'b, T, U>(
        &'b mut self,
        name: &'static str,
        default_value: T,
    ) -> BoundedConfigPropertyBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Bounded + Copy + Default,
    {
        BoundedConfigPropertyBuilder {
            root: self,
            name,
            bounds: Default::default(),
            default_value,
            initial_value: default_value,
            _marker: PhantomData,
        }
    }

    pub fn state<'b, T, U>(
        &'b mut self, 
        name: &'static str
    ) -> StatePropertyBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Default,
    {
        StatePropertyBuilder { 
            root: self, 
            name, 
            initial_value: Default::default(), 
            _marker: PhantomData 
        }
    }

    pub fn measurement<'b, T, U>(
        &'b mut self,
        name: &'static str,
    ) -> MeasurementBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Copy + Default,
    {
        MeasurementBuilder {
            root: self,
            name,
            record_min: false,
            record_max: false,
            initial_value: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn event<'b, T>(
        &'b mut self,
        name: &'static str,
    ) -> EventBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Serialize,
    {
        EventBuilder { root: self, name, _marker: PhantomData }
    }

    pub fn log_handle(&mut self) -> LogRecorderHandle {
        let rec = &mut self.data_store.recorder;
        rec.create_log_handle(LogOrigin::Machine(self.ident))
    }
}

// sub builders
pub struct ConfigPropertyBuilder<'a, 'b, T, U = ()>
where
    T: Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    default_value: T,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Default, U> ConfigPropertyBuilder<'_, '_, T, U>
where
    T: Clone + Default,
{
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> ConfigProperty<T, U> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let data_handle = reg.register_config(ident, name).unwrap();

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_config_handle(ident, name);

        ConfigProperty::new(data_handle, rec_handle, self.default_value, self.initial_value)
    }
}

pub struct BoundedConfigPropertyBuilder<'a, 'b, T, U = ()>
where
    T: Bounded + Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    bounds: Bounds<T>,
    default_value: T,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Default, U> BoundedConfigPropertyBuilder<'_, '_, T, U>
where
    T: Bounded + Clone + Default,
{
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn min(&mut self, value: T) -> &mut Self {
        self.bounds.min = Some(value.as_bound());
        self
    }

    pub fn max(&mut self, value: T) -> &mut Self {
        self.bounds.max = Some(value.as_bound());
        self
    }

    pub fn register(self) -> BoundedConfigProperty<T, U> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let data_handle = reg.register_config(ident, name).unwrap();

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_config_handle(ident, name);

        BoundedConfigProperty::new(
            data_handle, 
            rec_handle, 
            self.bounds,
            self.default_value, 
            self.initial_value
        )
    }
}

pub struct StatePropertyBuilder<'a, 'b, T, U = ()>
where
    T: Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Default, U> StatePropertyBuilder<'_, '_, T, U>
where
    T: Default,
{
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> StateProperty<T, U> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let data_handle = reg.register_state(ident, name).unwrap();

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_state_handle(ident, name);

        StateProperty::new(data_handle, rec_handle, self.initial_value)
    }
}

pub struct MeasurementBuilder<'a, 'b, T, U>
where
    T: Copy + Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    record_min: bool,
    record_max: bool,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Copy + Default, N> MeasurementBuilder<'_, '_, T, N>
where
    T: Copy + Default,
{
    pub fn record_min(&mut self) -> &mut Self {
        self.record_min = true;
        self
    }

    pub fn record_max(&mut self) -> &mut Self {
        self.record_max = true;
        self
    }

    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> Measurement<T, N> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let handle = reg.register_measurement(ident, name, false).unwrap();

        let min = if self.record_min {
            let name = reg.register_name(format!("{name}.min"));
            let handle = reg.register_measurement(ident, name, true).unwrap();
            Some(handle)
        } else {
            None
        };

        let max = if self.record_max {
            let name = reg.register_name(format!("{name}.max"));
            let handle = reg.register_measurement(ident, name, true).unwrap();
            Some(handle)
        } else {
            None
        };

        let stats = MeasurementStatistics::new(min, max);
        Measurement::new(handle, stats, self.initial_value)
    }
}

pub struct EventBuilder<'a, 'b, T: Serialize> {
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T: Serialize> EventBuilder<'_, '_, T> {
    pub fn register(self) -> MachineEventRecorderHandle<T> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());

        let rec = &mut self.root.data_store.recorder;

        rec.create_event_handle(ident, name)
    }
}

// Error
#[derive(Debug)]
pub enum MachineBuildError {
    RequiredEtherCATInterface,
    AlreadyRegistered(&'static str, &'static str),
    SchemaViolation,
    Custom(anyhow::Error),
}

impl Display for MachineBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered(prefix, name) => {
                write!(f, "'{prefix}.{name}' already registered")
            }
            Self::SchemaViolation => {
                write!(f, "machine schema violation")
            }
            Self::RequiredEtherCATInterface => {
                write!(f, "machine required a valid ethercat interface")
            }
            Self::Custom(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for MachineBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Custom(err) => Some(err.root_cause()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for MachineBuildError {
    fn from(err: anyhow::Error) -> Self {
        Self::Custom(err)
    }
}
