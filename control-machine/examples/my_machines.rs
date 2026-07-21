use control_macros::machine_build;
use qitech_framework_machine::{Build, BuildContext, BuildResult, resource::{StateProperty, StatePropertyOptions}};
use qitech_lib::units::{length::millimeter};

pub struct MockMachine {
    state: StateProperty<millimeter>,
}

pub fn main() {}

// #[machine_subscribe("laser_v1")]
// 

#[machine_build(machine = "laser_v1")]
impl Build for MockMachine {
    fn build(mut ctx: BuildContext<'_>) -> BuildResult<Self> {

        let x = state_property!("diameter.target");

        Ok(Self { 
            state: ctx.register_state_property("diameter.target", StatePropertyOptions {
                ..Default::default()
            })?,

            // state: 
            // state: state_property!(diameter.target),
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