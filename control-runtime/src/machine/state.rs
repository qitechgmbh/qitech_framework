use crate::with_uom;
use crate::resource::{MachineStatePropertyHandle};
use crate::conversion::{Wrapped, WrappedIntoScalar};

#[derive(Debug)]
pub struct StateProperty<T: Wrapped> {
    reg_handle: MachineStatePropertyHandle<T::Inner>,
    rec_handle: StateRecorderHandle,
}

impl<T: Wrapped> StateProperty<T> {
    pub(crate) fn new(
        reg_handle: property::Handle<T::Inner>,
        rec_handle: StateRecorderHandle,
    ) -> Self {
        Self { reg_handle, rec_handle }
    }
}

impl<T> StateProperty<T>
where
    T: WrappedIntoScalar,
    T::Inner: Clone,
{
    pub fn get(&self) -> &T::Inner { self.reg_handle.read() }

    pub fn set(&mut self, value: T::Inner) {
        self.reg_handle.write(value.clone());
        self.rec_handle.record(T::into_scalar(value));
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

with_uom!(impl_uom);