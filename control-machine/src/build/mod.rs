use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use control_core::MachineIdentificationUnique;
use crate::Hardware;
use crate::resource::{ConfigPropertyManager, MeasurementManager, StatePropertyManager};

mod types;
pub use types::MachineBuildError;

// mod config;
mod state;
mod measurement;

trait StatePropertySpec {
    const NAME: &'static str;
    type Value;
}

struct StatePropertyOptions {
    name: &'static str,
}

mod schemas {
    pub mod laser_v1 {
        pub mod config {
            pub mod diameter {
                use crate::build::StatePropertySpec;

                pub enum Target {
                    Idle,
                    Run,
                    Calibrate,
                }

                #[allow(non_camel_case_types)]
                pub struct target;

                // subscribing works by providing a StatePropertySpec
                impl StatePropertySpec for target {
                    const NAME: &'static str = "diameter.target";
                    type Value = Target;
                }

                pub mod tolerance {
                    use crate::build::StatePropertyOptions;

                    pub const lower: StatePropertyOptions = StatePropertyOptions {
                        name: "diameter.tolerance.lower",
                    };

                    pub const upper: StatePropertyOptions = StatePropertyOptions {
                        name: "diameter.tolerance.upper",
                    };
                }
            }

            
        }
    }
}

fn useit() {
    use schemas::laser_v1::*;
    config::diameter::target;

    let ctx: MachineBuildContext;

    let target = ctx.register(config::diameter::target)?;

    let target = ctx
        .config("diameter.target", Length::new::<millimeter>(1.75))
        .with_lower_bound(Length::ZERO)
        .register()?;

    let lower = ctx
        .config("diameter.tolerance.lower", Length::new::<millimeter>(0.05))
        .with_lower_bound(Length::ZERO)
        .register()?;

    let upper = ctx
        .config("diameter.tolerance.upper", Length::new::<millimeter>(0.05))
        .with_lower_bound(Length::ZERO)
        .register()?;
}

// ctx.state.register(registration: StatePropertyRegistration)

pub trait MachineBuild: Sized {
    fn build(ctx: MachineBuildContext<'_>) -> Result<Self, MachineBuildError>;
}

pub struct MachineBuildContext<'a> {
    ident: MachineIdentificationUnique,
    ethercat_interface: Option<EtherCATThreadChannel>,
    hardware: Vec<Hardware>,
    config_properties: &'a mut ConfigPropertyManager,
    state_properties: &'a mut StatePropertyManager,
    measurements: &'a mut MeasurementManager,
}

impl<'a> MachineBuildContext<'a> {
    pub fn new(
        ident: MachineIdentificationUnique,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
        config_properties: &'a mut ConfigPropertyManager,
        state_properties: &'a mut StatePropertyManager,
        measurements: &'a mut MeasurementManager,
    ) -> Self {
        Self {
            ident,
            ethercat_interface,
            hardware,
            config_properties,
            state_properties,
            measurements,
        }
    }

    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
