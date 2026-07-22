use control_macros::machine_build;
use qitech_framework_machine::{Build, BuildContext, BuildResult, resource::{StateProperty, StatePropertyOptions}};
use qitech_lib::units::{Length, length::millimeter};

// #[machine(mock_machine)]
pub struct MockMachine {
    in_tolerance: StateProperty<bool>,
}

//     // #[machine_subscribe(mock_machine)]

pub fn main() {
    ConfigPropertyOptions {
        
    };
}

// #[machine_subscribe("laser_v1")]
// state_property!("diameter.target")

impl Build for MockMachine {
    #[machine_build(mock_machine)]
    fn build(mut ctx: BuildContext<'_>) -> BuildResult<Self> {
        Ok(Self { 
            in_tolerance: state_property!("enabled", initial_value = false),
            // my_command: command!("start_winding", handle = |machine, args| {}),
        })

        // ctx.state::<millimeter>("diameter.target").with_initials_value(Length::new::<millimeter>(1.75)).register()?

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