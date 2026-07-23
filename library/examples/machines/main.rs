use qitech_framework::machine::{Machine, MachineBuild, build::{BuildContext, error::BuildResult, resource::{MeasurementOptions, StatePropertyOptions}}, error::ActResult, resource::{command::{CommandExecuteError, CommandHandle}, event::EventEmitter, measurement::Measurement, state_property::StateProperty}};
use serde::Serialize;


pub fn main() {

}


// #[machine("laser_v1")]
pub struct MyMachine {
    state: StateProperty<f64>,
    measurement: Measurement<f64>,
    event: EventEmitter<MyEvent>,

    // #[]
    command: CommandHandle,
}

// #[event("laser_v1", "just.some.event")]
#[derive(Serialize)]
pub struct MyEvent {
    apple: f64,
    tree: i64,
    awesome: bool,
}

impl MyMachine {
    pub fn start_winding(&mut self, _args: ()) -> Result<(), CommandExecuteError> {
        Ok(())
    }
}

impl Machine for MyMachine {
    fn act(&mut self) -> ActResult {
        _ = self.command.set_enabled(false)?;

        _ = self.event.emit(MyEvent {
            apple: 1.0,
            tree: 0,
            awesome: true,
        });

        Ok(())
    }
}

impl MachineBuild for MyMachine {
    #[machine_build(laser_v1)]
    fn build(mut ctx: BuildContext<'_>) -> BuildResult<Self> {
        
        // let command = command!("just.some.command", Self::start_winding);

        // let state = state_property!("just.some.state", initial_value = 0.0);

        let state = ctx.register_state_property::<f64>("just.some.state", StatePropertyOptions {
            initial_value: 0.0,
        })?;

        let measurement = ctx.register_measurement::<f64>("just.some.measurement", MeasurementOptions {
            initial_value: Some(0.0),
            .. Default::default()
        })?;

        let event = ctx.register_event("just.some.event")?;
        let command = ctx.register_command("just.some.command", Self::start_winding)?;

        Ok(Self { state, measurement, event, command })
    }
}
