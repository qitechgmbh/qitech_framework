use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::with_uom_quantities;

use super::PropertyHandle;
use crate::machine::resource::PropertyAccessor;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::PropertyRegistry;
use crate::machine::resource::PropertyResolver;
use crate::machine::resource::conversion::Extract;
use crate::machine::resource::conversion::TypeWrapper;
use crate::machine::resource::error::RegisterResult;
use crate::uom;

#[derive(Debug)]
pub struct Measurement<T> {
    handle: PropertyHandle<T>,
    stats: Statistics<T>,
}

impl<T: Copy> Measurement<T> {
    pub fn get(&self) -> T {
        *self.handle.read()
    }
}

impl<T: Copy + PartialOrd> Measurement<T> {
    pub fn set(&mut self, value: T) {
        self.handle.write(value);
        self.stats.update(value);
    }
}

macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl Measurement<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64)
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value));
            }
        }

        impl Measurement<Option<$quantity>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>)
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>));
            }
        }
    };
}

// with_uom_quantities!(uom, impl_uom);

// --- statistics ---
#[derive(Debug)]
struct Statistics<T> {
    min: Option<PropertyHandle<T>>,
    max: Option<PropertyHandle<T>>,
}

impl<T: Copy + PartialOrd> Statistics<T> {
    pub fn update(&mut self, value: T) {
        if let Some(min) = &mut self.min
            && *min.read() > value
        {
            min.write(value);
        }

        if let Some(max) = &mut self.max
            && *max.read() > value
        {
            max.write(value);
        }
    }
}

// --- resource managment ---
const SLOT_SIZE: usize = size_of::<f64>();
const MAX_ITEMS: usize = 512;
type Kind = super::kind::Measurement;

pub type Registry = PropertyRegistry<SLOT_SIZE, MAX_ITEMS, Kind, Option<f64>>;
pub type Resolver<'a> = PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, Kind, Option<f64>>;
pub type Reader<'a> = PropertyAccessor<'a, SLOT_SIZE, MAX_ITEMS, Kind, Option<f64>>;
pub type ReaderHandle<T> = PropertyReadHandle<Kind, T>;

pub(crate) struct Manager {
    registry: Registry,
}

impl Manager {
    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}

/// --- registering ---
pub struct Registrar<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

impl Registrar<'_> {
    pub fn register<T>(
        &mut self,
        path: &'static str,
        options: RegisterOptions<T::Type>,
    ) -> RegisterResult<Measurement<T::Type>>
    where
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + Default,
    {
        let extract = T::extract;
        let reg = &mut self.manager.registry;

        let handle = reg.register::<T::Type>(self.machine, path, "", T::extract, ())?;
        handle.write(options.initial_value.unwrap_or_default());

        let stat_min_handle = if options.record_min {
            Some(reg.register::<T::Type>(self.machine, path, "min", extract, ())?)
        } else {
            None
        };

        let stat_max_handle = if options.record_max {
            Some(reg.register::<T::Type>(self.machine, path, "max", extract, ())?)
        } else {
            None
        };

        let stats = Statistics {
            min: stat_min_handle,
            max: stat_max_handle,
        };

        Ok(Measurement { handle, stats })
    }
}

#[derive(Debug, Default)]
pub struct RegisterOptions<T> {
    pub initial_value: Option<T>,
    pub record_min: bool,
    pub record_max: bool,
}
