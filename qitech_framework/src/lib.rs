use std::thread;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::session;
use qitech_framework_core::session::debug::DebugRuntimeSessionProvider;
pub use qitech_framework_core::vendors;
pub use qitech_framework_hub::HubConfiguration;
pub use qitech_framework_macros::*;
use qitech_framework_tui::Tui;
pub use qitech_framework_tui::TuiConfiguration;
pub use qitech_lib::units;

use crate::runtime::Runtime;
use crate::runtime::RuntimeConfiguration;

pub mod machine;
mod resource;

pub mod runtime;

#[doc(hidden)]
/// exposed for proc macros
pub mod __private {
    pub use qitech_framework_core::ScalarValue;
    pub use qitech_framework_core::ScalarValueTypeMismatchError;
    pub use qitech_framework_core::report::ConstraintViolationError;
    pub use qitech_framework_core::report::Constraints;
    pub use qitech_framework_core::schema::MeasurementDefinition;
    pub use qitech_framework_core::schema::ScalarPropertyDefinition;

    pub use crate::machine::MachineDescriptor;
    pub use crate::resource::constraints::EnumConstraints;
    pub use crate::resource::conversion::PropertyAdapter;
    pub use crate::resource::conversion::PropertyType;
}

pub fn run_debug(config: RuntimeConfiguration) {
    let runtime = Runtime::init(config, DebugRuntimeSessionProvider).unwrap();
    runtime.run().unwrap();
}

pub async fn run_with_hub(
    config_runtime: RuntimeConfiguration,
    config_hub: HubConfiguration,
) -> anyhow::Result<()> {
    let (provider_runtime, provider_controller) = session::mpsc(64);

    let runtime_thread = thread::spawn(move || {
        let runtime = Runtime::init(config_runtime, provider_runtime).unwrap();
        runtime.run().unwrap();
    });

    _ = runtime_thread;

    qitech_framework_hub::run(config_hub, provider_controller).await;
    Ok(())
}

pub async fn run_with_tui(
    runtime_config: RuntimeConfiguration,
    tui_config: TuiConfiguration,
) -> anyhow::Result<()> {
    let (runtime_provider, tui_provider) = session::mpsc(64);

    let runtime_thread = thread::spawn(move || -> anyhow::Result<()> {
        let runtime = Runtime::init(runtime_config, runtime_provider)?;
        runtime.run();
        Ok(())
    });

    let tui_result = match Tui::create(tui_config) {
        Ok(tui) => tui.run(tui_provider).await,
        Err(err) => Err(err),
    };

    let runtime_result = runtime_thread.join().unwrap();

    // tui_result?;
    runtime_result?;

    Ok(())
}
