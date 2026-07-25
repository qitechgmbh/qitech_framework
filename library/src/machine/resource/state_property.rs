use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineStateMutation;
use qitech_framework_common::with_uom_quantities;

use super::PropertyHandle;
use crate::machine::resource::Journal;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::conversion::ScalarTypeWrapper;
use crate::machine::resource::error::RegisterResult;
use crate::uom;

pub struct StateProperty<T> {
    handle: PropertyHandle<T>,
    record: RecordFn<T>,
}

impl<T: Clone> StateProperty<T> {
    pub fn set(&mut self, value: T) {
        (self.record)(&value);
        self.handle.write(value);
    }

    pub fn get_ref(&self) -> &T {
        self.handle.read()
    }
}

impl<T: Copy> StateProperty<T> {
    pub fn get(&self) -> T {
        *self.handle.read()
    }
}

macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl StateProperty<$quantity> {
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

        impl StateProperty<Option<$quantity>> {
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

with_uom_quantities!(uom, impl_uom);

// --- manager ---
const SLOT_SIZE: usize = size_of::<String>();
const MAX_ITEMS: usize = 512;
type Kind = super::property_kind::StateProperty;

pub type ReaderHandle<T> = PropertyReadHandle<Kind, T>;

#[derive(Default)]
pub struct Manager {
    registry: PropertyManager<SLOT_SIZE, MAX_ITEMS, Kind>,
    journal: Journal<MachineStateMutation>,
}

impl Manager {
    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        initial_value: T::Type,
    ) -> RegisterResult<StateProperty<T::Type>>
    where
        T: ScalarTypeWrapper,
        T::Type: Default,
    {
        // create boxed function to type erase the wrapper type
        // to reduce the amount of expanded generic code inside the property
        let journal = self.journal.new_handle();
        let record = Box::new(move |value: &T::Type| {
            let entry = MachineStateMutation {
                source: ident,
                resource_path: Cow::Borrowed(path),
                value: T::into_scalar(value),
                timestamp: Utc::now(),
            };

            journal.append(entry);
        });

        let handle = self
            .registry
            .register::<T::Type>(ident, path, "", (), initial_value)?;
        Ok(StateProperty { handle, record })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.registry.unregister_machine(ident)
    }

    pub fn drain_journal(&mut self, f: impl FnMut(MachineStateMutation)) {
        self.journal.drain_with(f);
    }
}

// --- types ---
pub type RecordFn<T> = Box<dyn Fn(&T)>;

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

        let mut r = Manager::default();

        let mut sp: StateProperty<f64> = r.register::<f64>(ident, "just.some.float", 1.0)?;
        assert_eq!(sp.get(), 1.0);
        sp.set(2.0);
        assert_eq!(sp.get(), 2.0);

        let mut sp: StateProperty<Option<f64>> =
            r.register::<Option<f64>>(ident, "just.some.float.optional", None)?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1.0));
        assert_eq!(sp.get(), Some(1.0));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: StateProperty<i64> = r.register::<i64>(ident, "just.some.int", 1)?;
        assert_eq!(sp.get(), 1);
        sp.set(2);
        assert_eq!(sp.get(), 2);

        let mut sp: StateProperty<Option<i64>> =
            r.register::<Option<i64>>(ident, "just.some.optional.int", None)?;
        assert_eq!(sp.get(), None);
        sp.set(Some(1));
        assert_eq!(sp.get(), Some(1));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: StateProperty<bool> = r.register::<bool>(ident, "just.some.bool", false)?;
        assert!(!sp.get());
        sp.set(true);
        assert!(sp.get());
        sp.set(false);
        assert!(!sp.get());

        let mut sp: StateProperty<Option<bool>> =
            r.register::<Option<bool>>(ident, "just.some.optional.bool", None)?;
        assert_eq!(sp.get(), None);
        sp.set(Some(true));
        assert_eq!(sp.get(), Some(true));
        sp.set(None);
        assert_eq!(sp.get(), None);

        let mut sp: StateProperty<String> =
            r.register::<String>(ident, "just.some.string", String::from("hello"))?;
        assert_eq!(sp.get_ref(), "hello");
        sp.set(String::from("world"));
        assert_eq!(sp.get_ref(), "world");
        sp.set(String::from("rust"));
        assert_eq!(sp.get_ref(), "rust");

        let mut sp: StateProperty<Option<String>> =
            r.register::<Option<String>>(ident, "just.some.optional.string", None)?;
        assert_eq!(*sp.get_ref(), None);
        sp.set(Some(String::from("hello")));
        assert_eq!(*sp.get_ref(), Some(String::from("hello")));
        sp.set(None);
        assert_eq!(*sp.get_ref(), None);

        // --- uom ---
        let mut sp: StateProperty<Length> =
            r.register::<millimeter>(ident, "just.some.length", Length::new::<millimeter>(1.0))?;

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
        let mut sp: StateProperty<Option<Length>> =
            r.register::<Option<centimeter>>(ident, "just.some.optional.millimeter", None)?;

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
