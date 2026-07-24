use qitech_framework::machine::BuildContext;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine::error::ActResult;
use qitech_framework::machine::error::BuildResult;
use qitech_framework::machine::error::CommandExecuteError;
use qitech_framework::uom::Length;
use qitech_framework::uom::length::centimeter;
use qitech_framework::uom::length::meter;
use qitech_framework::uom::length::millimeter;
use serde::Serialize;
use uom::ConstZero;

pub struct TestMachine;

// #[event("laser_v1", "just.some.event")]
#[derive(Serialize)]
pub struct MyEvent {
    apple: f64,
    tree: i64,
    awesome: bool,
}

impl TestMachine {
    pub fn start_winding(&mut self, _args: ()) -> Result<(), CommandExecuteError> {
        Ok(())
    }
}

impl Machine for TestMachine {
    fn act(&mut self) -> ActResult {
        Ok(())
    }
}

impl MachineBuild for TestMachine {
    // #[machine_build(laser_v1)]
    fn build(mut ctx: BuildContext<'_>) -> BuildResult<Self> {
        // --- state property tests ---
        let sp: StateProperty<f64> = ctx.state.register("just.some.float.state", 1.0)?;
        sp.set(1.0).unwrap();
        sp.set(2.0).unwrap();
        sp.get();

        let sp: StateProperty<Option<f64>> =
            ctx.state.register("just.some.float.optional.state", None)?;
        sp.set(Some(1.0)).unwrap();
        sp.set(None).unwrap();
        sp.get();

        let sp: StateProperty<i64> = ctx.state.register("just.some.int.state", 1)?;
        sp.set(1).unwrap();
        sp.set(2).unwrap();
        sp.get();

        let sp: StateProperty<Option<i64>> =
            ctx.state.register("just.some.optional.int.state", None)?;
        sp.set(Some(1)).unwrap();
        sp.set(None).unwrap();
        sp.get();

        let sp: StateProperty<bool> = ctx.state.register("just.some.bool.state", false)?;
        sp.set(true).unwrap();
        sp.set(false).unwrap();
        sp.get();

        let sp: StateProperty<Option<bool>> =
            ctx.state.register("just.some.optional.bool.state", None)?;
        sp.set(Some(true)).unwrap();
        sp.set(None).unwrap();
        sp.get();

        let sp: StateProperty<String> = ctx
            .state
            .register("just.some.string.state", String::from("hello"))?;
        sp.set(String::from("world")).unwrap();
        sp.set(String::from("rust")).unwrap();
        sp.get();

        let sp: StateProperty<Option<String>> = ctx
            .state
            .register("just.some.optional.string.state", None)?;
        sp.set(Some(String::from("hello"))).unwrap();
        sp.set(None).unwrap();
        sp.get();

        // --- uom ---
        let sp: StateProperty<millimeter> = ctx
            .state
            .register("just.some.optional.string.state", Length::new(1.0))?;

        sp.set(Length::new(99.0)).unwrap();
        sp.set(Length::ZERO).unwrap();
        sp.get();

        sp.set_as::<millimeter>(1.0).unwrap();
        sp.set_as::<centimeter>(1.0).unwrap();
        sp.set_as::<meter>(1.0);

        sp.get_as::<millimeter>().unwrap();
        sp.get_as::<centimeter>().unwrap();
        sp.get_as::<meter>();

        // --- uom optional ---
        let sp: StateProperty<millimeter> = ctx
            .state
            .register("just.some.optional.string.state", Length::new(1.0))?;

        sp.set(Length::new(99.0)).unwrap();
        sp.set(Length::ZERO).unwrap();
        sp.get();

        sp.set_as::<millimeter>(1.0).unwrap();
        sp.set_as::<centimeter>(1.0).unwrap();
        sp.set_as::<meter>(1.0);

        sp.get_as::<millimeter>().unwrap();
        sp.get_as::<centimeter>().unwrap();
        sp.get_as::<meter>();

        Ok(Self)
    }
}
