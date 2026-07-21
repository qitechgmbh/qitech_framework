use control_macros::machine_build;
use qitech_lib::units::{ConstZero, Length, length::millimeter};
use qitech_framework_machine::{build::{BuildContext, BuildError, MachineBuild}, resource::{ConfigProperty, Specification, StateProperty, StatePropertySpecification, kind}};

pub struct MockMachine {
    state: StateProperty<millimeter>,
}

pub fn main() {}

// #[machine_subscribe("laser_v1")]
// 

#[machine_build(machine = "laser_v1")]
impl MachineBuild for MockMachine {
    fn build(mut ctx: BuildContext<'_>) -> Result<Self, BuildError> {

        let x = state_property!("diameter.target");

        Ok(Self { 
            state: ctx.state::<TargetDiameter>().register(Length::ZERO)?,
        })

        // ctx.state::<millimeter>("diameter.target").with_initial_value(Length::new::<millimeter>(1.75)).register()?

        /*
        Ok(Self {
            my_config,

            config: config_property!(
                resource = "diameter.target",
                validate = |x| x != 2.0
            )
            state: state_property!("diameter.target"),
            measurement: measurement!("diameter"),
            event: event!("out_of_tolerance", MyEvent),
            command: command!(
                resource = "start_winding", 
                predicate = can_wind,
                handler = start_winding,
            ),
        })
        */
    }
}

struct TargetDiameter;
impl StatePropertySpecification for TargetDiameter {}
impl Specification for TargetDiameter {
    const NAME: &'static str = "diameter.target";
    type Kind = kind::StateProperty;
    type Type = millimeter;
}

