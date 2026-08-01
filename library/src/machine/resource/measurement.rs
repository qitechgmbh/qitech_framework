use std::borrow::Cow;

use qitech_framework_core::MachineIdentificationUnique;
use qitech_framework_core::MachineMeasurement;
use qitech_framework_core::with_uom_quantities;

use super::PropertyHandle;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::conversion::Extract;
use crate::machine::resource::conversion::TypeWrapper;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::property_kind;
use crate::machine::resource::subscription::SubscribeError;
use crate::machine::resource::subscription::SubscribedProperty;

#[derive(Debug)]
pub struct Measurement<T> {
    handle: PropertyHandle<T>,
    stats: Statistics<T>,
}

impl<T> Measurement<T>
where
    T: Copy,
{
    pub fn get(&self) -> T {
        *self.handle.read()
    }
}

impl<T> Measurement<T>
where
    T: Copy + PartialOrd,
{
    pub fn set(&mut self, value: T) {
        self.handle.write(value);
        self.stats.update(value);
    }
}

// --- uom impl ---
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
                self.set(<$quantity>::new::<N>(value))
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
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_quantities!(impl_uom);

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
    ) -> RegisterResult<Measurement<T::Type>>
    where
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + PartialOrd + Default,
    {
        let mut init_handle =
            |postfix: Option<&'static str>| -> RegisterResult<PropertyHandle<T::Type>> {
                let path: Cow<'static, str> = match postfix {
                    Some(postfix) => Cow::Owned(format!("{path}.{postfix}")),
                    None => Cow::Borrowed(path),
                };

                self.inner.register::<T::Type>(
                    ident,
                    path.to_string(),
                    Metadata {
                        extract: T::extract,
                        is_stat: postfix.is_some(),
                    },
                    T::Type::default(),
                )
            };

        let handle = (init_handle)(None)?;
        handle.write(options.initial);

        let stat_min_handle = if options.record_min {
            Some(init_handle(Some("min"))?)
        } else {
            None
        };

        let stat_max_handle = if options.record_max {
            Some(init_handle(Some("max"))?)
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

    // --- subscriptions ---
    pub fn create_subscriber<T: 'static>(
        &mut self,
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
        resource: &'static str,
    ) -> Result<SubscribedProperty<T>, SubscribeError> {
        self.inner.create_subscriber(provider, subscriber, resource)
    }

    pub fn remove_subscription(
        &mut self,
        provider: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    ) {
        self.inner.remove_subscription(provider, consumer);
    }

    pub fn sync_cache(&mut self) {
        self.inner.sync_cache();
    }

    // --- export data ---
    // TODO: reset measurements somehow
    pub fn iter(&mut self, mut f: impl FnMut(MachineMeasurement)) {
        for (info, bytes) in self.inner.iter_mut() {
            let info = unsafe { info.as_ref() };
            // we don't know what T is but how to extract it
            let value = unsafe { (info.metadata.extract)(bytes) };

            let entry = MachineMeasurement {
                machine: info.machine,
                path: info.path.clone(),
                value,
            };

            (f)(entry);
        }
    }
}

#[derive(Debug, Default)]
pub struct RegisterOptions<T: Default> {
    pub initial: T,
    pub record_min: bool,
    pub record_max: bool,
}

// --- testing ---
#[cfg(test)]
mod test {
    use qitech_framework_core::MachineIdentification;
    use qitech_lib::units::ConstZero;
    use qitech_lib::units::Length;
    use qitech_lib::units::length::centimeter;
    use qitech_lib::units::length::meter;
    use qitech_lib::units::length::millimeter;

    use super::*;

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

        let mut sp: Measurement<Option<f64>> = r.register::<Option<f64>>(
            ident,
            "just.some.float.optional",
            RegisterOptions {
                initial: None,
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1.0));
        assert_eq!(sp.get(), Some(1.0));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: Measurement<i64> = r.register::<i64>(
            ident,
            "just.some.int",
            RegisterOptions {
                initial: 1,
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), 1);
        sp.set(2);
        assert_eq!(sp.get(), 2);

        let mut sp: Measurement<Option<i64>> = r.register::<Option<i64>>(
            ident,
            "just.some.optional.int",
            RegisterOptions {
                initial: None,
                record_min: true,
                record_max: true,
            },
        )?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1));
        assert_eq!(sp.get(), Some(1));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: Measurement<bool> = r.register::<bool>(
            ident,
            "just.some.bool",
            RegisterOptions {
                initial: false,
                record_min: true,
                record_max: true,
            },
        )?;
        assert!(!sp.get());
        sp.set(true);
        assert!(sp.get());
        sp.set(false);
        assert!(!sp.get());

        let mut sp: Measurement<Option<bool>> = r.register::<Option<bool>>(
            ident,
            "just.some.optional.bool",
            RegisterOptions {
                initial: None,
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
        let mut sp: Measurement<Length> = r.register::<millimeter>(
            ident,
            "just.some.millimeter",
            RegisterOptions {
                initial: Length::new::<millimeter>(1.0),
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
        let mut sp: Measurement<Option<Length>> = r.register::<Option<millimeter>>(
            ident,
            "just.some.optional.millimeter",
            RegisterOptions {
                initial: None,
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
