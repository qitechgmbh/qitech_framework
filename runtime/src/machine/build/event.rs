use serde::Serialize;

impl<'a> MachineBuilder<'a> {
    pub fn event<'b, T>(
        &'b mut self,
        name: &'static str,
    ) -> EventBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Serialize,
    {
        EventBuilder { root: self, name, _marker: PhantomData }
    }
}

pub struct EventBuilder<'a, 'b, T: Serialize> {
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T: Serialize> EventBuilder<'_, '_, T> {
    pub fn register(self) -> MachineEventRecorderHandle<T> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());

        let rec = &mut self.root.data_store.recorder;

        rec.create_event_handle(ident, name)
    }
}