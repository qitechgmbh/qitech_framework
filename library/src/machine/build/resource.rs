use std::marker::PhantomData;
use std::rc::Rc;

use qitech_framework_core::report::MachineConfigPropertyConstraints;
use qitech_framework_core::report::MachineConfigWriteCapability;
use serde::Serialize;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::build::BuildError;
use crate::machine::build::BuildResult;
use crate::machine::resource::CanExecuteFn;
use crate::machine::resource::ConfigProperty;
use crate::machine::resource::ConfigPropertyCapabilities;
use crate::machine::resource::EventEmitter;
use crate::machine::resource::ExecuteFn;
use crate::machine::resource::GetCapabilitiesFn;
use crate::machine::resource::IntoCanExecuteFn;
use crate::machine::resource::IntoExecuteFn;
use crate::machine::resource::Measurement;
use crate::machine::resource::MeasurementRegisterOptions;
use crate::machine::resource::StateProperty;
use crate::machine::resource::conversion::Extract;
use crate::machine::resource::conversion::TypeWrapper;
use crate::machine::resource::error::RegisterError;

// --- config property ---
impl<'a> BuildContext<'a> {
    pub fn config<'b, T>(&'b mut self, path: &'static str) -> ConfigPropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: TypeWrapper + 'static,
        T::Type: Clone + Default,
    {
        ConfigPropertyBuilder {
            root: self,
            path,
            default: T::Type::default(),
            constraints: None,
            get_capabilities: None,
        }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T>
where
    T: TypeWrapper + 'static,
    T::Type: Clone,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,

    // --- configuration ---
    default: T::Type,
    constraints: Option<T::Constraints>,
    get_capabilities: Option<GetCapabilitiesFn>,
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: TypeWrapper + 'static,
    T::Type: Clone,
{
    pub fn default(mut self, value: T::Input) -> Self {
        self.options.default = T::convert_input(value);
        self
    }

    pub fn minimum(mut self, value: T::Input) -> Self {
        self.options.min = T::convert_input(value).as_bound();
        self
    }

    pub fn max(mut self, value: T::Input) -> Self {
        self.options.max = T::convert_input(value).as_bound();
        self
    }

    pub fn get_capabilities<M: Machine + 'static>(
        self,
        func: fn(&M) -> ConfigPropertyCapabilities,
    ) -> Self {
        self.get_capabilities = Some(func.into_get_capabilities_fn());
        Self
    }

    pub fn register(self) -> BuildResult<ConfigProperty<T::Type>> {
        let get_capabilities_fn = self.get_capabilities.unwrap_or_else(|| {
            Rc::new(move |machine: &dyn Machine| {
                _ = machine;

                Ok(ConfigPropertyCapabilities {
                    writable: MachineConfigWriteCapability::allowed(),
                    constraints: MachineConfigPropertyConstraints::None,
                })
            })
        });

        Ok(self.root.resources.config_properties.register::<T>(
            self.root.ident,
            self.path,
            self.options,
        )?)
    }
}

// --- state property ---
impl<'a> BuildContext<'a> {
    pub fn state<'b, T>(&'b mut self, path: &'static str) -> StatePropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: TypeWrapper,
        T::Type: Default,
    {
        StatePropertyBuilder {
            root: self,
            path,
            initial: Default::default(),
        }
    }
}

pub struct StatePropertyBuilder<'a, 'b, T>
where
    T: TypeWrapper,
    T::Type: Default,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    initial: T::Type,
}

impl<'a, 'b, T> StatePropertyBuilder<'a, 'b, T>
where
    T: TypeWrapper,
    T::Type: Default,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.initial = T::convert_input(value);
        self
    }

    pub fn register(self) -> BuildResult<StateProperty<T::Type>> {
        let res = self.root.resources.state_properties.register::<T>(
            self.root.ident,
            self.path,
            self.initial,
        );

        if let Err(e) = &res {
            println!("e: {e} | {}", self.path);
        }

        Ok(res?)
    }
}

// --- measurement ---
impl<'a> BuildContext<'a> {
    pub fn measurement<'b, T>(&'b mut self, path: &'static str) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + PartialOrd + Default,
    {
        MeasurementBuilder {
            root: self,
            path,
            options: Default::default(),
        }
    }
}

pub struct MeasurementBuilder<'a, 'b, T>
where
    T: TypeWrapper + Extract<Option<f64>> + 'static,
    T::Type: Copy + PartialOrd + Default,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    options: MeasurementRegisterOptions<T::Type>,
}

impl<'a, 'b, T> MeasurementBuilder<'a, 'b, T>
where
    T: TypeWrapper + Extract<Option<f64>> + 'static,
    T::Type: Copy + PartialOrd + Default,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.options.initial = T::convert_input(value);
        self
    }

    pub fn record_min(mut self) -> Self {
        self.options.record_min = true;
        self
    }

    pub fn record_max(mut self) -> Self {
        self.options.record_max = true;
        self
    }

    pub fn record_avg(mut self) -> Self {
        self.options.record_avg = true;
        self
    }

    pub fn record_stddev(mut self) -> Self {
        self.options.record_stddev = true;
        self
    }

    pub fn register(self) -> BuildResult<Measurement<T::Type>> {
        Ok(self.root.resources.measurements.register::<T>(
            self.root.ident,
            self.path,
            self.options,
        )?)
    }
}

// --- command ---
impl<'a> BuildContext<'a> {
    pub fn command<'b, M>(&'b mut self, path: &'static str) -> CommandBuilder<'a, 'b, M>
    where
        'a: 'b,
        M: Machine + 'static,
    {
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
    can_execute_fn: Option<CanExecuteFn>,
    execute_fn: Option<ExecuteFn>,
    _marker: PhantomData<M>,
}

impl<'a, 'b, M> CommandBuilder<'a, 'b, M>
where
    M: Machine + 'static,
{
    pub fn can_execute(mut self, value: fn(&M) -> bool) -> Self {
        self.can_execute_fn = Some(value.into_can_execute_fn());
        self
    }

    pub fn execute(mut self, value: fn(&mut M) -> Result<(), String>) -> Self {
        self.execute_fn = Some(value.into_execute_fn());
        self
    }

    pub fn register(self) -> BuildResult<()> {
        let Some(execute) = self.execute_fn else {
            let err = RegisterError::MissingRequiredField("execute");
            return Err(BuildError::RegisterError(err));
        };

        Ok(self.root.resources.commands.register(
            self.root.ident,
            self.path,
            self.disabled,
            execute,
        )?)
    }
}

// --- event ---
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
    T: Serialize + 'static,
{
    pub fn register(self) -> BuildResult<EventEmitter<T>> {
        Ok(self
            .root
            .resources
            .events
            .register::<T>(self.root.ident, self.path)?)
    }
}
