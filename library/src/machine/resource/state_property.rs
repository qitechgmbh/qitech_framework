use core::fmt;
use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineStateMutation;
use qitech_framework_common::ScalarValue;
use qitech_framework_common::with_uom_units;

use super::JournalHandle;
use super::PropertyHandle;
use crate::machine::resource::Journal;
use crate::machine::resource::PropertyAccessor;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::PropertyRegistry;
use crate::machine::resource::PropertyResolver;
use crate::machine::resource::conversion::ScalarTypeWrapper;
use crate::machine::resource::error::RegisterResult;
use crate::uom;

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
    T::Type: Copy,
{
    pub fn get(&self) -> T::Type {
        *self.handle.read()
    }
}

impl StateProperty<String> {
    pub fn get_ref(&self) -> &String {
        self.handle.read()
    }
}

impl StateProperty<Option<String>> {
    pub fn get_ref(&self) -> &Option<String> {
        self.handle.read()
    }
}

impl<T> StateProperty<T>
where
    T: ScalarTypeWrapper,
{
    pub fn set(&mut self, value: T::Type) -> Result<(), WriteError> {
        self.journal
            .append(MachineStateMutation {
                source: self.ident,
                resource_path: Cow::Borrowed(self.path),
                value: T::into_scalar(&value),
                timestamp: Utc::now(),
            })
            .map_err(|_| WriteError::JournalFull)?;

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

            pub fn set_as<N>(&mut self, value: f64) -> Result<(), WriteError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl StateProperty<Option<$unit>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>) -> Result<(), WriteError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_units!(uom, impl_uom);

// --- manager ---
const SLOT_SIZE: usize = size_of::<String>();
const MAX_ITEMS: usize = 512;
type Kind = super::kind::Measurement;
type Format = ScalarValue;

pub type Registry = PropertyRegistry<SLOT_SIZE, MAX_ITEMS, Kind, Format>;
pub type Resolver<'a> = PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, Kind, Format>;
pub type Reader<'a> = PropertyAccessor<'a, SLOT_SIZE, MAX_ITEMS, Kind, Format>;
pub type ReaderHandle<T> = PropertyReadHandle<Kind, T>;

pub(crate) struct Manager {
    registry: Registry,
    journal: Journal<MachineStateMutation>,
}

impl Manager {
    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }

    pub(crate) fn new() -> Self {
        Self {
            registry: Default::default(),
            journal: Journal::new(),
        }
    }
}

/// --- registering ---
pub struct Registrar<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

impl<'a> Registrar<'a> {
    pub(crate) fn new(manager: &'a mut Manager, machine: MachineIdentificationUnique) -> Self {
        Self { manager, machine }
    }

    pub fn register<T>(
        &mut self,
        path: &'static str,
        initial_value: T::Type,
    ) -> RegisterResult<StateProperty<T>>
    where
        T: ScalarTypeWrapper,
        T::Type: Default,
    {
        let handle =
            self.manager
                .registry
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
#[derive(Debug)]
pub enum WriteError {
    JournalFull,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::JournalFull => write!(f, "journal is full"),
        }
    }
}

impl std::error::Error for WriteError {}

// --- testing ---
#[cfg(test)]
mod test {
    use qitech_framework_common::MachineIdentification;
    use uom_crate::ConstZero;

    use super::*;
    use crate::uom::Length;
    use crate::uom::length::centimeter;
    use crate::uom::length::meter;
    use crate::uom::length::millimeter;

    #[test]
    pub fn register_and_use() -> anyhow::Result<()> {
        let ident = MachineIdentificationUnique {
            identification: MachineIdentification {
                vendor_id: 0,
                machine_id: 0,
            },
            serial: 0,
        };

        let mut mgr = Manager::new();
        let mut r = Registrar::new(&mut mgr, ident);

        let mut sp: StateProperty<f64> = r.register("just.some.float", 1.0)?;
        assert_eq!(sp.get(), 1.0);
        sp.set(2.0)?;
        assert_eq!(sp.get(), 2.0);

        let mut sp: StateProperty<Option<f64>> = r.register("just.some.float.optional", None)?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1.0))?;
        assert_eq!(sp.get(), Some(1.0));
        sp.set(None)?;
        assert_eq!(sp.get(), None);

        let mut sp: StateProperty<i64> = r.register("just.some.int", 1)?;
        assert_eq!(sp.get(), 1);
        sp.set(2)?;
        assert_eq!(sp.get(), 2);

        let mut sp: StateProperty<Option<i64>> = r.register("just.some.optional.int", None)?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1))?;
        assert_eq!(sp.get(), Some(1));
        sp.set(None)?;
        assert_eq!(sp.get(), None);

        let mut sp: StateProperty<bool> = r.register("just.some.bool", false)?;
        assert!(!sp.get());
        sp.set(true)?;
        assert!(sp.get());
        sp.set(false)?;
        assert!(!sp.get());

        let mut sp: StateProperty<Option<bool>> = r.register("just.some.optional.bool", None)?;
        assert_eq!(sp.get(), None);
        sp.set(Some(true))?;
        assert_eq!(sp.get(), Some(true));
        sp.set(None)?;
        assert_eq!(sp.get(), None);

        let mut sp: StateProperty<String> =
            r.register("just.some.string", String::from("hello"))?;
        assert_eq!(sp.get_ref(), "hello");
        sp.set(String::from("world"))?;
        assert_eq!(sp.get_ref(), "world");
        sp.set(String::from("rust"))?;
        assert_eq!(sp.get_ref(), "rust");

        let mut sp: StateProperty<Option<String>> =
            r.register("just.some.optional.string", None)?;
        assert_eq!(*sp.get_ref(), None);
        sp.set(Some(String::from("hello")))?;
        assert_eq!(*sp.get_ref(), Some(String::from("hello")));
        sp.set(None)?;
        assert_eq!(*sp.get_ref(), None);

        // --- uom ---
        let mut sp: StateProperty<millimeter> =
            r.register("just.some.millimeter", Length::new::<millimeter>(1.0))?;

        assert_eq!(sp.get_as::<millimeter>(), 1.0);

        sp.set(Length::new::<meter>(99.0))?;
        assert_eq!(sp.get_as::<meter>(), 99.0);

        sp.set(Length::ZERO)?;
        assert_eq!(sp.get_as::<millimeter>(), 0.0);

        sp.set_as::<millimeter>(1.0)?;
        assert_eq!(sp.get_as::<millimeter>(), 1.0);

        sp.set_as::<centimeter>(1.0)?;
        assert_eq!(sp.get_as::<centimeter>(), 1.0);

        sp.set_as::<meter>(1.0)?;
        assert_eq!(sp.get_as::<meter>(), 1.0);

        // --- uom optional ---
        let mut sp: StateProperty<Option<millimeter>> =
            r.register("just.some.optional.millimeter", None)?;

        assert_eq!(sp.get(), None);

        sp.set(Some(Length::new::<centimeter>(99.0)))?;
        assert_eq!(sp.get_as::<centimeter>(), Some(99.0));

        sp.set(Some(Length::ZERO))?;
        assert_eq!(sp.get_as::<millimeter>(), Some(0.0));

        sp.set_as::<millimeter>(Some(1.0))?;
        assert_eq!(sp.get_as::<millimeter>(), Some(1.0));

        sp.set_as::<centimeter>(Some(1.0))?;
        assert_eq!(sp.get_as::<centimeter>(), Some(1.0));

        sp.set_as::<meter>(Some(1.0))?;
        assert_eq!(sp.get_as::<meter>(), Some(1.0));

        Ok(())
    }
}
