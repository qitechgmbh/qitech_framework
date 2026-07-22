use serde::de::DeserializeOwned;

use crate::MachineBuildContext;

impl<'a> MachineBuildContext<'a> {
    pub fn command<'b, M: 'static, A>(
        &'b mut self,
        name: &'static str,
        execute: fn(&mut M, A),
    ) -> CommandBuilder<'a, 'b, M, A>
    where
        A: DeserializeOwned + 'static,
    {
        CommandBuilder {
            root: self,
            name,
            pred: None,
            execute,
            initial_state: C,
        }
    }
}

pub struct CommandBuilder<'a, 'b, M, A = ()> {
    root: &'b mut MachineBuildContext<'a>,
    name: &'static str,
    pred: Option<fn(&M, &A)>,
    execute: fn(&mut M, A),
    initial_state: CommandState,
}

impl<M, A> CommandBuilder<'_, '_, M, A> {
    pub fn with_disable_by_default(mut self) -> Self {
        self.initial_state = CommandState::Disabled { reason: "disabled by default" };
        self
    }

    pub fn with_predicate(mut self, pred: fn(&M, &A)) -> Self {
        self.pred = Some(pred);
        self
    }

    pub fn register(self) -> CommandHandle<A> {
        let ident = self.root.identification();

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_machine_name(self.name.to_string());
        let data_handle = reg.register_machine_config_property(ident, name).unwrap();

        let rec = &mut self.root.data_store.journals;
        let rec_handle = rec.create_config_handle(ident, name);

        CommandHandle { entry: (), _marker: () }
    }
}
