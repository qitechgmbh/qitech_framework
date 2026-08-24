use std::cell::RefCell;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework::Machine;
use qitech_framework::MachineIdentification;
use qitech_framework::TuiConfiguration;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine_build;
use qitech_framework::run_with_tui;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_framework::runtime::XtremConfig;
use qitech_framework::vendors;
use qitech_lib::units::Mass;
use qitech_lib::units::mass::kilogram;
use qitech_lib::xtrem::ScaleMode;
use qitech_lib::xtrem::XtremBusConfig;
use qitech_lib::xtrem::XtremDevice;
use qitech_lib::xtrem::XtremScale;

/// Device ids of the modules on the bus (register `0001h`) — the address the bus routes replies
/// on, and the reason each module needs a distinct one.
///
/// Run `cargo run -p xtrem --example discover` to read them off the hardware, or start this app
/// and read them out of the `XtremDiscoveryCompleted` init event — every module that answers the
/// sweep is listed there, claimed or not. Modules ship as `01`; assign distinct ids with
/// `cargo run -p xtrem --example assign_ids`.
const SCALE_1_DEVICE_ID: u8 = 0x03;
const SCALE_2_DEVICE_ID: u8 = 0x04;
const SCALE_3_DEVICE_ID: u8 = 0x05;

/// Directed broadcast for the machine subnet, plus the port the modules listen on (register
/// `0701h`).
///
/// This has to be the subnet's own broadcast address, not the `255.255.255.255` default: on a
/// host with more than one interface the all-ones address leaves via the default route, which is
/// the office network rather than the machine network, and no module ever hears the sweep.
const BROADCAST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 4, 255), 4444);

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(true)
        .init();

    let bus = XtremBusConfig {
        broadcast_addr: BROADCAST,
        ..Default::default()
    };

    let config_rt = RuntimeConfiguration::new()
        .xtrem(XtremConfig {
            bus,
            ..Default::default()
        })
        .xtrem_device::<XtremScale>(
            SCALE_1_DEVICE_ID,
            ScaleV1::IDENTIFICATION.unique(1),
            ScaleMode::Poll,
        )
        .xtrem_device::<XtremScale>(
            SCALE_2_DEVICE_ID,
            ScaleV1::IDENTIFICATION.unique(2),
            ScaleMode::Poll,
        )
        .xtrem_device::<XtremScale>(
            SCALE_3_DEVICE_ID,
            ScaleV1::IDENTIFICATION.unique(3),
            ScaleMode::Poll,
        )
        .machine::<ScaleV1>();

    run_with_tui(
        config_rt,
        TuiConfiguration::default().refresh_rate(Duration::from_millis(10)),
    )
    .await
    .unwrap()
}

#[derive(Machine)]
pub struct ScaleV1 {
    // --- hardware ---
    device: Rc<RefCell<XtremScale>>,

    // --- state ---
    stable: StateProperty<bool>,
    overload: StateProperty<bool>,

    // --- measurements ---
    net: Measurement<Mass>,
    gross: Measurement<Mass>,
    tare: Measurement<Mass>,

    // --- misc ---
    request_timer: Duration,
}

impl MachineBuild for ScaleV1 {
    #[machine_build(ScaleV1)]
    fn build(ctx: &mut BuildContext<'_>) -> BuildResult<Self> {
        let device = ctx.get_xtrem_device::<XtremScale>(0)?;

        // Reset the gross reference so an empty platform reads zero.
        ctx.command::<ScaleV1>("zero")
            .execute(|machine| {
                machine.device.borrow_mut().zero();
                Ok(())
            })
            .build()?;

        // Store the current load as the tare, which drives net to zero. The module waits for a
        // stable reading and refuses if it never settles.
        ctx.command::<ScaleV1>("tare")
            .execute(|machine| {
                machine.device.borrow_mut().tare();
                Ok(())
            })
            .build()?;

        // Drop the stored tare, so net reads gross again.
        ctx.command::<ScaleV1>("clear_tare")
            .execute(|machine| {
                machine.device.borrow_mut().clear_tare();
                Ok(())
            })
            .build()?;

        Ok(Self {
            device,
            stable: ctx.state::<bool>("stable").build()?,
            overload: ctx.state::<bool>("overload").build()?,
            net: ctx.measurement::<kilogram>("net").build()?,
            gross: ctx.measurement::<kilogram>("gross").build()?,
            tare: ctx.measurement::<kilogram>("tare").build()?,
            request_timer: Duration::ZERO,
        })
    }
}

impl Machine for ScaleV1 {
    fn act(&mut self, dt: Duration) -> ActResult {
        self.update_device(dt);

        let Some(reading) = self.device.borrow().reading else {
            return Ok(());
        };

        self.net.set(reading.net);
        self.gross.set(reading.gross.mass);
        self.tare.set(reading.tare.mass);

        self.stable.set(reading.status.stable());
        self.overload.set(reading.status.overload());

        Ok(())
    }
}

impl ScaleV1 {
    pub const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 11,
    };

    /// Drive one polling step.
    fn update_device(&mut self, dt: Duration) {
        let mut scale = self.device.borrow_mut();

        if let Err(error) = scale.handle_response() {
            tracing::warn!("xtrem handle_response failed: {error}");
        }

        if let Some(error) = scale.take_error() {
            tracing::warn!("xtrem device {:02X}h: {error}", scale.device_id());
        }

        // The runtime ticks every 100 µs, which is far faster than the module can answer.
        self.request_timer = self.request_timer.saturating_sub(dt);

        if self.request_timer.is_zero() {
            self.request_timer = Duration::from_millis(20);

            if let Err(error) = scale.send_next_request() {
                tracing::warn!("xtrem send_next_request failed: {error}");
            }
        }
    }
}
