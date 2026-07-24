use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConfigProperty => "config property",
            Self::StateProperty => "state property",
            Self::Measurement => "measurement",
            Self::Command => "command",
            Self::Event => "event",
        };
        f.write_str(s)
    }
}

#[allow(non_camel_case_types)]
pub trait KindVariant: private::Sealed {}

mod private {
    pub trait Sealed: { const KIND: super::Kind; }
}

#[derive(Debug)]
pub struct StateProperty;

#[derive(Debug)]
pub struct ConfigProperty;

#[derive(Debug)]
pub struct Measurement;

#[derive(Debug)]
pub struct Command;

#[derive(Debug)]
pub struct Event;

macro_rules! impl_kind {
    ($kind:tt) => {
        impl private::Sealed for $kind {
            const KIND: Kind = Kind::$kind;
        }
        
        impl KindVariant for $kind {}
    };
}

impl_kind!(ConfigProperty);
impl_kind!(StateProperty);
impl_kind!(Measurement);
impl_kind!(Command);
impl_kind!(Event);