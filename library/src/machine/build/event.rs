use std::marker::PhantomData;

use qitech_framework_core::report::error::BuildError;
use serde::Serialize;

use crate::machine::BuildContext;
use crate::machine::EventEmitter;
use crate::resource::ResourceKey;

impl<'a> BuildContext<'a> {
    pub fn event<'b, T>(&'b mut self, path: &'static str) -> EventBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Serialize,
    {
        EventBuilder {
            root: self,
            path,
            _marker: PhantomData,
        }
    }
}

pub struct EventBuilder<'a, 'b, T>
where
    T: Serialize,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    _marker: PhantomData<T>,
}

impl<'a, 'b, T> EventBuilder<'a, 'b, T>
where
    T: Serialize,
{
    pub fn build(&mut self) -> Result<EventEmitter<T>, BuildError> {
        if self.root.events_registered.contains(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        self.root.events_registered.insert(self.path);

        let key = ResourceKey {
            ident: self.root.ident,
            path: self.path,
        };

        let journal = self.root.journals.events.new_handle();

        Ok(EventEmitter {
            key,
            journal,
            _marker: PhantomData,
        })
    }
}
