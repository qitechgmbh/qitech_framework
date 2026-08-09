use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::marker::PhantomData;

use chrono::Utc;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::CommandRecord;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::error::ActResult;
use crate::machine::error::BuildResult;
use crate::machine::instance::CommandCanExecuteFn;
use crate::machine::instance::CommandHandle;

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

    can_execute: Option<fn(&M) -> OperationCapability>,
    execute: Option<fn(&mut M) -> ActResult>,

    _marker: PhantomData<M>,
}

impl<'a, 'b, M> CommandBuilder<'a, 'b, M>
where
    M: Machine + 'static,
{
    pub fn can_execute(mut self, func: fn(&M) -> OperationCapability) -> Self {
        self.can_execute = Some(func);
        self
    }

    pub fn execute(mut self, func: fn(&mut M) -> ActResult) -> Self {
        self.execute = Some(func);
        self
    }

    pub fn build(self) -> BuildResult<()> {
        if self.root.type_id != TypeId::of::<M>() {
            return Err(BuildError::MachineTypeMismatch {
                expected: self.root.type_name.to_string(),
                received: type_name::<M>().to_string(),
            });
        }

        if self.root.commands_registered.contains_key(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let Some(execute) = self.execute else {
            return Err(BuildError::MissingRequiredField("execute".to_string()));
        };

        let can_execute_fn = self.can_execute.map(|func| {
            Box::new(move |machine: &dyn Machine| -> OperationCapability {
                let machine = (machine as &dyn Any)
                    .downcast_ref::<M>()
                    .expect("machine type mismatch");

                (func)(machine)
            }) as CommandCanExecuteFn
        });

        let execute_fn = Box::new(move |machine: &mut dyn Machine| -> ActResult {
            let machine = (machine as &mut dyn Any)
                .downcast_mut::<M>()
                .expect("machine type mismatch");

            (execute)(machine)
        });

        self.root.commands_registered.insert(
            self.path,
            CommandHandle {
                capability_prev: OperationCapability::Allowed,
                can_execute_fn,
                execute_fn,
            },
        );

        self.root
            .journals_temp
            .commands
            .new_handle()
            .append(CommandRecord {
                timestamp: Utc::now(),
                machine: self.root.ident,
                path: self.path.to_string(),
                event: CommandEvent::Registered,
            });

        Ok(())
    }
}
