use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineStateMutation;
use qitech_framework_common::ScalarValue;
use qitech_framework_common::with_uom_units;

use crate::machine::resource::Journal;
use crate::machine::resource::PropertyAccessor;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::PropertyRegistry;
use crate::machine::resource::PropertyResolver;
use crate::machine::resource::conversion::ScalarTypeWrapper;
use crate::machine::resource::error::RegisterResult;
use crate::uom;

use super::JournalHandle;
use super::PropertyHandle;

#[derive(Debug)]
pub struct StateProperty<T: ScalarTypeWrapper> {
    ident: MachineIdentificationUnique,
    path: &'static str,
    handle: PropertyHandle<T::Type>,
    journal: JournalHandle<MachineStateMutation>,
}

impl<T> StateProperty<T>
where
    T: ScalarTypeWrapper,
{
    pub fn get(&self) -> &T::Type {
        self.handle.read()
    }

    pub fn set(&mut self, value: T::Type) -> Result<(), WriteError> {
        self.journal.append(MachineStateMutation {
            source: self.ident,
            resource_path: Cow::Borrowed(self.path),
            value: T::into_scalar(&value),
            timestamp: Utc::now(),
        }).map_err(|_| WriteError::JournalFull);

        self.handle.write(value.clone());
        Ok(())
    }
}

macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl StateProperty<$unit> {
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

        impl StateProperty<Option<$unit>> {
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

with_uom_units!(uom, impl_uom);

// --- manager ---
const SLOT_SIZE: usize = size_of::<f64>();
const MAX_ITEMS: usize = 512;
type Kind = super::kind::Measurement;
type Format = ScalarValue;

pub type Registry = PropertyRegistry<SLOT_SIZE, MAX_ITEMS, Kind, Format>;
pub type Resolver<'a> = PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, Kind, Format>;
pub type Reader<'a> = PropertyAccessor<'a, SLOT_SIZE, MAX_ITEMS, Kind, Format>;
pub type ReaderHandle<T> = PropertyReadHandle<Kind, T>;

#[derive(Debug)]
pub(crate) struct Manager {
    registry: Registry,
    journal: Journal<MachineStateMutation>,
}

impl Manager {
    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}

/// --- registering ---
#[derive(Debug)]
pub struct Registrar<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

impl Registrar<'_> {
    pub(crate) fn register<T>(
        &mut self,
        path: &'static str,
        initial_value: T::Type,
    ) -> RegisterResult<StateProperty<T>> 
    where
        T: ScalarTypeWrapper,
        T::Type: Default,
    {
        let handle = self.manager.registry
            .register::<T::Type>(self.machine, path, "", T::extract, ())?;

        handle.write(initial_value);

        Ok(StateProperty {
            handle,
            journal: self.manager.journal.init_handle(),
            ident: self.machine,
            path,
        })
    }
}

// --- errors ---
pub enum WriteError {
    JournalFull,
}
