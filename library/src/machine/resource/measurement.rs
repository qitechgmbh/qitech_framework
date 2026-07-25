use std::borrow::Cow;

use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineMeasurement;
use qitech_framework_common::with_uom_units;

use super::PropertyHandle;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::conversion::Extract;
use crate::machine::resource::conversion::TypeWrapper;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::property_kind;
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
            && *max.read() < value
        {
            max.write(value);
        }
    }
}

// --- manager ---
const SLOT_SIZE: usize = size_of::<f64>();
const MAX_ITEMS: usize = 512;
type Kind = property_kind::Measurement;

#[derive(Clone, Copy)]
struct Metadata {
    extract: unsafe fn(*const u8) -> Option<f64>,

    #[allow(unused)]
    is_stat: bool,
}

pub type ReadHandle<T> = PropertyReadHandle<Kind, T>;

#[derive(Default)]
pub struct Manager {
    inner: PropertyManager<SLOT_SIZE, MAX_ITEMS, Kind, Metadata>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        options: RegisterOptions<T::Type>,
    ) -> RegisterResult<Measurement<T>>
    where
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + PartialOrd + Default,
    {
        let mut init_handle = |postfix: &'static str| -> RegisterResult<PropertyHandle<T::Type>> {
            self.inner.register::<T::Type>(
                ident,
                path,
                postfix,
                Metadata {
                    extract: T::extract,
                    is_stat: !postfix.is_empty(),
                },
                T::Type::default()
            )
        };

        let handle = (init_handle)("")?;
        handle.write(options.initial_value.unwrap_or_default());

        let stat_min_handle = if options.record_min {
            Some(init_handle("min")?)
        } else {
            None
        };

        let stat_max_handle = if options.record_max {
            Some(init_handle("min")?)
        } else {
            None
        };

        let stats = Statistics {
            min: stat_min_handle,
            max: stat_max_handle,
        };

        Ok(Measurement { handle, stats })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.inner.unregister_machine(ident)
    }

    pub fn create_read_handle(&mut self) {}

    // TODO: reset measurements somehow
    pub fn drain_measurements(&mut self, mut f: impl FnMut(MachineMeasurement)) {
        for (info, bytes) in self.inner.iter_mut() {
            // we don't know what T is but how to extract it
            let value = unsafe { (info.metadata.extract)(bytes) };

            let entry = MachineMeasurement {
                source: info.ident,
                resource_path: Cow::Borrowed(info.path),
                value,
            };

            (f)(entry);
        }
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

        let mut r = Manager::new();

        let mut sp: Measurement<Option<f64>> = r.register(
            ident,
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
            ident,
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
            ident,
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
            ident,
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
            ident,
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
            ident,
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
            ident,
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
