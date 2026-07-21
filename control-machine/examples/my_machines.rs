use control_macros::machine_build;
use qitech_framework_machine::{Build, BuildContext, BuildResult, resource::{StateProperty, StatePropertyOptions}};
use qitech_lib::units::{Length, length::millimeter};

pub struct MockMachine {
    in_tolerance: StateProperty<Option<millimeter>>,
}

pub fn main() {}

// #[machine_subscribe("laser_v1")]
// state_property!("diameter.target")

#[machine_build(mock_machine)]
impl Build for MockMachine {
    fn build(mut ctx: BuildContext<'_>) -> BuildResult<Self> {
        Ok(Self { 
            in_tolerance: state_property!("enabled", initial_value = false),

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