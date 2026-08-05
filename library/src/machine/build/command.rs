use std::any::TypeId;
use std::marker::PhantomData;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::build::BuildError;
use crate::machine::build::BuildResult;
use crate::resource::CommandDefinition;

impl<'a> BuildContext<'a> {
    pub fn command<'b, M>(&'b mut self, path: &'static str) -> CommandBuilder<'a, 'b, M>
    where
        'a: 'b,
        M: Machine + 'static,
    {
        assert_eq!(
            TypeId::of::<M>(),
            self.type_id,
            "Attempted to register a command for the wrong machine type."
        );

        if self.commands.iter().find(|x| x.path == path).is_some() {
            todo!("Return Duplicate Entry")
        }

        CommandBuilder {
            root: self,
            path,
            can_execute_fn: None,
            execute_fn: None,
            _marker: PhantomData,
        }
    }
}

pub struct CommandBuilder<'a, 'b, M>
where
    M: Machine + 'static,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,

    can_execute_fn: Option<*const ()>,
    execute_fn: Option<*const ()>,

    _marker: PhantomData<M>,
}

impl<'a, 'b, M> CommandBuilder<'a, 'b, M>
where
    M: Machine + 'static,
{
    pub fn can_execute(mut self, value: fn(&M) -> bool) -> Self {
        self.can_execute_fn = Some(value as *const ());
        self
    }

    pub fn execute(mut self, value: fn(&mut M) -> Result<(), String>) -> Self {
        self.execute_fn = Some(value as *const ());
        self
    }

    pub fn register(self) -> BuildResult<()> {
        let Some(execute) = self.execute_fn else {
            return Err(BuildError::MissingRequiredField("execute"));
        };

        let can_execute = self
            .can_execute_fn
            .unwrap_or(always_can_execute::<M> as *const ());

        self.root.commands.push(CommandDefinition {
            path: self.path,
            can_execute,
            execute,
        });

        Ok(())
    }
}

fn always_can_execute<M: Machine>(_: &M) -> bool {
    true
}
