use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineStateMutation;
use super::{JournalHandle, PropertyHandle};

mod manager;
pub use manager::Manager;
pub use manager::Resolver;
pub use manager::Reader;
pub use manager::ReadHandle;

#[derive(Debug)]
pub struct StateProperty<T> {
    ident: MachineIdentificationUnique,
    name: &'static str,
    handle: PropertyHandle<T>,
    journal: JournalHandle<MachineStateMutation>,
}

impl<T> StateProperty<T> {
    pub fn get(&self) -> &T { self.handle.read() }

    // pub fn set(&mut self, value: T) {
    //     self.handle.write(value.clone());
    // 
    //     self.journal.append(MachineStateMutation { 
    //         source: self.ident, 
    //         resource_path: Cow::Borrowed(self.name), 
    //         value: T::into_scalar(value),
    //         timestamp: Utc::now(), 
    //     });
    // }
}

/*
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
*/