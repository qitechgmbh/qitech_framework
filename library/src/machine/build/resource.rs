use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::build::BuildResult;
use crate::machine::error::CommandExecuteError;
use crate::machine::resource::CommandHandle;
use crate::machine::resource::CommandRegisterOptions;
use crate::machine::resource::ConfigProperty;
use crate::machine::resource::ConfigPropertyRegisterOptions;
use crate::machine::resource::EventEmitter;
use crate::machine::resource::Measurement;
use crate::machine::resource::MeasurementRegisterOptions;
use crate::machine::resource::StateProperty;
use crate::machine::resource::conversion::BoundedMeta;
use crate::machine::resource::conversion::Extract;
use crate::machine::resource::conversion::ScalarTypeWrapper;
use crate::machine::resource::conversion::TypeWrapper;

// --- config property ---
impl<'a> BuildContext<'a> {
    pub fn config<'b, T>(&'b mut self, path: &'static str) -> ConfigPropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: ScalarTypeWrapper + 'static,
        T::Type: Clone + DeserializeOwned + Default + BoundedMeta,
    {
        ConfigPropertyBuilder {
            root: self,
            path,
            options: Default::default(),
        }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T>
where
    T: ScalarTypeWrapper + 'static,
    T::Type: Clone + DeserializeOwned + BoundedMeta,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    options: ConfigPropertyRegisterOptions<T::Type>,
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: ScalarTypeWrapper + 'static,
    T::Type: Clone + DeserializeOwned + BoundedMeta,
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

    pub fn validate(mut self, value: fn(&T::Type) -> Result<(), String>) -> Self {
        self.options.validate = Some(value);
        self
    }

    pub fn register(self) -> BuildResult<ConfigProperty<T::Type>> {
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
        T: ScalarTypeWrapper,
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
    T: ScalarTypeWrapper,
    T::Type: Default,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    initial: T::Type,
}

impl<'a, 'b, T> StatePropertyBuilder<'a, 'b, T>
where
    T: ScalarTypeWrapper,
    T::Type: Default,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.initial = T::convert_input(value);
        self
    }

    pub fn register(self) -> BuildResult<StateProperty<T::Type>> {
        Ok(self.root.resources.state_properties.register::<T>(
            self.root.ident,
            self.path,
            self.initial,
        )?)
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
    pub fn command<'b, M, A>(&'b mut self, path: &'static str) -> CommandBuilder<'a, 'b, M, A>
    where
        'a: 'b,
        M: Machine + 'static,
        A: serde::de::DeserializeOwned + 'static,
    {
        CommandBuilder {
            root: self,
            path,
            options: Default::default(),
            _marker: PhantomData,
        }
    }
}

pub struct CommandBuilder<'a, 'b, M, A>
where
    M: Machine + 'static,
    A: serde::de::DeserializeOwned + 'static,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    options: CommandRegisterOptions<M, A>,
    _marker: PhantomData<(M, A)>,
}

impl<'a, 'b, M, A> CommandBuilder<'a, 'b, M, A>
where
    M: Machine + 'static,
    A: serde::de::DeserializeOwned + 'static,
{
    pub fn execute(mut self, execute: fn(&mut M, A) -> Result<(), CommandExecuteError>) -> Self {
        self.options.execute = Some(execute);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.options.disabled = true;
        self
    }

    pub fn register(self) -> BuildResult<CommandHandle> {
        Ok(self.root.resources.commands.register::<M, A>(
            self.root.ident,
            self.path,
            self.options,
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
