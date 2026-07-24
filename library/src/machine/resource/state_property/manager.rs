use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineStateMutation;
use qitech_framework_common::ScalarValue;

use super::StateProperty;
use crate::machine::resource::Journal;
use crate::machine::resource::PropertyAccessor;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::PropertyRegistry;
use crate::machine::resource::PropertyResolver;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::kind;
use crate::machine::resource::property::ExtractFn;

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type Resolver<'a> =
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type Reader<'a> = PropertyAccessor<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ReaderHandle<T> = PropertyReadHandle<kind::StateProperty, T>;

pub struct Manager {
    registry: Registry,
    journal: Journal<MachineStateMutation>,
}

impl Manager {
    pub(crate) fn register<T: Default + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        initial_value: T,
        extract: ExtractFn<ScalarValue>,
    ) -> RegisterResult<StateProperty<T>> {
        let handle = self.registry.register::<T>(ident, path, "", extract, ())?;
        handle.write(initial_value);

        Ok(StateProperty {
            handle,
            journal: self.journal.init_handle(),
            ident,
            name: path,
        })
    }

    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }

    pub(crate) fn drain_journal_with(&mut self, f: impl FnMut(&MachineStateMutation)) {
        self.journal.drain_with(f);
    }
}
