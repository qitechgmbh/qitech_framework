
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

#[allow(non_camel_case_types)]
pub trait kind_t: private::Kind {}

mod private {
    pub trait Kind: { const KIND: super::Kind; }
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
        impl private::Kind for $kind {
            const KIND: Kind = Kind::$kind;
        }
        
        impl kind_t for $kind {}
    };
}

impl_kind!(ConfigProperty);
impl_kind!(StateProperty);
impl_kind!(Measurement);
impl_kind!(Command);
impl_kind!(Event);