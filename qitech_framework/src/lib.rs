use std::thread;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::session;
pub use qitech_framework_core::vendors;
use qitech_framework_hub::HubConfiguration;
pub use qitech_framework_macros::*;
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

pub async fn run_with_hub(
    config_runtime: RuntimeConfiguration,
    config_hub: HubConfiguration,
) -> Result<(), i64> {
    let (session_runtime, session_controller) = session::tokio_mpsc(64);

    let runtime_thread = thread::spawn(move || {
        let runtime = Runtime::init(config_runtime, session_runtime).unwrap();
        runtime.run().unwrap();
    });

    qitech_framework_hub::run(config_hub, session_controller).await;

    panic!("oh no ");
    Ok(())
}

pub mod app {
    use std::thread;

    use qitech_framework_core::session;
    use qitech_framework_hub::HubConfiguration;

    use crate::runtime::Runtime;
    use crate::runtime::RuntimeConfiguration;

    pub fn run_hub(
        config_runtime: RuntimeConfiguration,
        config_hub: HubConfiguration,
    ) -> Result<(), i64> {
        let (session_runtime, session_controller) = session::tokio_mpsc(64);

        let runtime_thread = thread::spawn(move || {
            let runtime = Runtime::init(config_runtime, session_runtime).unwrap();
            runtime.run();
        });

        qitech_framework_hub::run(config_hub, session_controller);
        Ok(())
    }

    /*
    pub fn run_tui(
        runtime_config: RuntimeConfiguration,
        tui_config: TuiConfiguration,
    ) -> Result<(), Error> {
        let (runtime_session, tui_session) = session::crossbeam(64);

        let runtime_thread = thread::spawn(move || -> Result<(), Error> {
            let runtime = Runtime::init(runtime_config, runtime_session)?;
            runtime.run();
            Ok(())
        });

        let tui_result = Tui::create(tui_config)
            .and_then(|tui| tui.run(tui_session));

        // tui_session is dropped here, causing the runtime to notice
        // that its controller disappeared.
        let runtime_result = runtime_thread
            .join()
            .map_err(|_| Error::RuntimePanic)?;

        tui_result?;
        runtime_result?;

        Ok(())
    }
    */
}
