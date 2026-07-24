use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::with_uom_units;

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
pub struct Measurement<T: TypeWrapper> {
    handle: PropertyHandle<T::Type>,
    stats: Statistics<T::Type>,
}

impl<T> Measurement<T>
where
    T: TypeWrapper,
    T::Type: Copy,
{
    pub fn get(&self) -> T::Type {
        *self.handle.read()
    }
}

impl<T> Measurement<T>
where
    T: TypeWrapper,
    T::Type: Copy + PartialOrd,
{
    pub fn set(&mut self, value: T::Type) {
        self.handle.write(value);
        self.stats.update(value);
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl Measurement<$unit> {
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
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl Measurement<Option<$unit>> {
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
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_units!(uom, impl_uom);

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
    pub(crate) fn new() -> Self {
        Self {
            registry: Default::default(),
        }
    }

    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
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
        options: RegisterOptions<T::Type>,
    ) -> RegisterResult<Measurement<T>>
    where
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + PartialOrd + Default,
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

        let mut sp: Measurement<Option<f64>> = r.register(
            "just.some.float.optional",
            RegisterOptions {
                initial_value: None,
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1.0));
        assert_eq!(sp.get(), Some(1.0));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: Measurement<i64> = r.register(
            "just.some.int",
            RegisterOptions {
                initial_value: Some(1),
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), 1);
        sp.set(2);
        assert_eq!(sp.get(), 2);

        let mut sp: Measurement<Option<i64>> = r.register(
            "just.some.optional.int",
            RegisterOptions {
                initial_value: None,
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1));
        assert_eq!(sp.get(), Some(1));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: Measurement<bool> = r.register(
            "just.some.bool",
            RegisterOptions {
                initial_value: Some(false),
                record_min: true,
                record_max: true,
            },
        )?;
        assert!(!sp.get());
        sp.set(true);
        assert!(sp.get());
        sp.set(false);
        assert!(!sp.get());

        let mut sp: Measurement<Option<bool>> = r.register(
            "just.some.optional.bool",
            RegisterOptions {
                initial_value: None,
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), None);
        sp.set(Some(true));
        assert_eq!(sp.get(), Some(true));
        sp.set(None);
        assert_eq!(sp.get(), None);

        // --- uom ---
        let mut sp: Measurement<millimeter> = r.register(
            "just.some.millimeter",
            RegisterOptions {
                initial_value: Some(Length::new::<millimeter>(1.0)),
                record_min: true,
                record_max: true,
            },
        )?;

        assert_eq!(sp.get_as::<millimeter>(), 1.0);

        sp.set(Length::new::<meter>(99.0));
        assert_eq!(sp.get_as::<meter>(), 99.0);

        sp.set(Length::ZERO);
        assert_eq!(sp.get_as::<millimeter>(), 0.0);

        sp.set_as::<millimeter>(1.0);
        assert_eq!(sp.get_as::<millimeter>(), 1.0);

        sp.set_as::<centimeter>(1.0);
        assert_eq!(sp.get_as::<centimeter>(), 1.0);

        sp.set_as::<meter>(1.0);
        assert_eq!(sp.get_as::<meter>(), 1.0);

        // --- uom optional ---
        let mut sp: Measurement<Option<millimeter>> = r.register(
            "just.some.optional.millimeter",
            RegisterOptions {
                initial_value: None,
                record_min: true,
                record_max: true,
            },
        )?;

        assert_eq!(sp.get(), None);

        sp.set(Some(Length::new::<centimeter>(99.0)));
        assert_eq!(sp.get_as::<centimeter>(), Some(99.0));

        sp.set(Some(Length::ZERO));
        assert_eq!(sp.get_as::<millimeter>(), Some(0.0));

        sp.set_as::<millimeter>(Some(1.0));
        assert_eq!(sp.get_as::<millimeter>(), Some(1.0));

        sp.set_as::<centimeter>(Some(1.0));
        assert_eq!(sp.get_as::<centimeter>(), Some(1.0));

        sp.set_as::<meter>(Some(1.0));
        assert_eq!(sp.get_as::<meter>(), Some(1.0));

        Ok(())
    }
}
