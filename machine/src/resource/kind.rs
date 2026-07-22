use control_core::MachineResourceKind;

pub trait Kind: private::Kind {}

mod private {
    pub trait Kind: { const KIND: control_core::MachineResourceKind; }
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
            const KIND: MachineResourceKind = MachineResourceKind::$kind;
        }
        
        impl Kind for $kind {}
    };
}

impl_kind!(ConfigProperty);
impl_kind!(StateProperty);
impl_kind!(Measurement);
impl_kind!(Command);
impl_kind!(Event);