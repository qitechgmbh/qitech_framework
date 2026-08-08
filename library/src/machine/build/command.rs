use std::any::Any;
use std::any::TypeId;
use std::marker::PhantomData;

use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::error::BuildResult;

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

        CommandBuilder {
            root: self,
            path,
            can_execute: None,
            execute: None,
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

    can_execute: Option<fn(&M) -> bool>,
    execute: Option<fn(&mut M) -> Result<(), String>>,

    _marker: PhantomData<M>,
}

impl<'a, 'b, M> CommandBuilder<'a, 'b, M>
where
    M: Machine + 'static,
{
    pub fn can_execute(mut self, func: fn(&M) -> bool) -> Self {
        self.can_execute = Some(func);
        self
    }

    pub fn execute(mut self, func: fn(&mut M) -> Result<(), String>) -> Self {
        self.execute = Some(func);
        self
    }

    pub fn register(self) -> BuildResult<()> {
        if self
            .root
            .commands
            .iter()
            .find(|x| x.path == self.path)
            .is_some()
        {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let Some(execute) = self.execute else {
            return Err(BuildError::MissingRequiredField("execute".to_string()));
        };

        self.root.commands.push(CommandItem {
            path: self.path,
            can_execute: self.can_execute,
            execute,
        });

        Ok(())
    }
}

pub struct CommandItem {
    path: &'static str,
    can_execute: Option<Box<dyn Fn(&dyn Machine) -> bool>>,
    execute: Box<dyn Fn(&mut dyn Machine) -> Result<(), String>>,
}
