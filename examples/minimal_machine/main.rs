use std::time::Duration;

use control_run::{self, Machine, MachineActResult, MachineBuild, MachineBuildError, MachineBuilder, MachineRegistry, Measurement};

pub fn main() {
    let config = control_run::Config { 
        stay_in_preop: false, 
        hotplug_duration: Duration::from_secs(1),
    };

    let mut registry = MachineRegistry::default();
    registry.register(include_str!("schema.yaml"), MinimalMachine::build);

    control_run::run(config, registry);
}

struct MinimalMachine {
    counter: Measurement<i64>,
}

impl MachineBuild for MinimalMachine {
    fn build(mut builder: MachineBuilder<'_>) -> Result<Self, MachineBuildError> {
        Ok(Self { 
            config: builder.config("config").on_changed().register() 
            counter: builder.measurement("counter").register() 
        })
    }
}

impl Machine for MinimalMachine {
    fn act(&mut self) -> MachineActResult {
        self.counter.set(self.counter.get() + 1);
        _ = self;
        Ok(())
    }

    fn react(&mut self, registry: &DataRegistry) -> MachineActResult {
        registry.
    }
}
