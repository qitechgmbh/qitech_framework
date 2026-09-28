# Creating a Machine

This guide walks through building a machine with the QiTech Framework, from an empty crate in your own project to a running machine you can control from the TUI. It assumes you know Rust and have read [resources.md](resources.md).

For reference, the framework repository contains complete, working examples in `examples/apps/`:

| Example | Hardware | Shows |
|---|---|---|
| `beckhoff_el2004` | EtherCAT digital output | the minimal machine: config properties with change callbacks |
| `beckhoff_el1002` / `beckhoff_el4008` / `wago_750_531` | EtherCAT digital/analog I/O | reading and writing EtherCAT devices |
| `qitech_laser` | Modbus RTU laser | units, state, measurements, events, hardware fault handling |
| `xtrem_scale` | Xtrem bus scales | commands, several instances of one machine type |

## How it fits together

A machine consists of four parts:

1. A **schema** (YAML) declares the machine's identity and its [resources](resources.md). Controllers use it to know what exists.
2. A **struct** holds the hardware handles, resource handles and any internal state.
3. **`MachineBuild::build`** runs once at startup. It finds the hardware and creates the resource handles.
4. **`Machine::act`** runs every cycle (every 100 µs by default). This is where the control logic goes.

You **register the machine type** with the Runtime. The Runtime creates **one instance for every machine identity it finds in the hardware**. Registering a type doesn't create an instance by itself: with no hardware assigned to it, the type simply never runs.

```
schema.yaml ──(compile time)──► #[derive(Machine)] / #[machine_build]
                                         │
RuntimeConfiguration::machine::<M>() ────┤  registers the type
                                         ▼
hardware discovery ── finds vendor:machine:serial ──► M::build(ctx) ──► act(dt) every cycle
```

## 1. Set up your project

Your machine lives in **your own crate or workspace**. The framework is a dependency, and you don't need to change anything in the framework repository.

```sh
cargo new --bin my_machine
cd my_machine
```

### Toolchain

The framework uses Rust **edition 2024** and requires **Rust 1.90 or newer**. Pin the toolchain so everyone builds with the same version:

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.90.0"
```

### Dependencies

The framework isn't published on crates.io yet, so add it as a git dependency. **Pin a `rev`**: the project is experimental, and `main` can break between commits.

```toml
# Cargo.toml
[package]
name = "my_machine"
version = "0.1.0"
edition = "2024"

[dependencies]
qitech_framework = { git = "https://github.com/qitechgmbh/qitech_framework", rev = "<framework commit>" }

# Hardware drivers (EtherCAT terminals, Modbus devices, Xtrem, ...) and units.
# MUST be the exact same rev that the framework uses. See below.
qitech_lib = { git = "https://github.com/qitechgmbh/qitech_lib.git", rev = "<same rev as the framework>" }

tokio = { version = "1", features = ["full"] }
```

**`qitech_lib` must use the same revision as the framework.** Device types such as `EL2004` and traits such as `EthercatDevice` come from `qitech_lib`. If your crate and the framework pull different revisions, Cargo compiles two copies of the library, and their types don't match. You then get confusing errors like "expected `EL2004`, found `EL2004`" or "trait `EthercatDevice` is not implemented". To find the right revision, look up the `qitech_lib` entry in the framework's root `Cargo.toml` at the commit you pinned.

Units such as `Length` and `millimeter` are also available as `qitech_framework::units`, so if you only need units you don't need `qitech_lib` directly.

### Project layout

```
my_machine/
├── Cargo.toml
├── rust-toolchain.toml
├── .cargo/
│   └── config.toml      # runner that grants hardware capabilities (step 7)
├── schemas/
│   └── my_machine.yaml  # one file per machine type (step 2)
└── src/
    └── main.rs
```

Put schemas in `schemas/` next to the `Cargo.toml` of the crate that defines the machine. That is where the macros look by default.

### Several machines in one workspace

If you build several machine types, a workspace keeps the framework revision in one place:

```toml
# <workspace root>/Cargo.toml
[workspace]
members = ["machines/*", "app"]
resolver = "2"

[workspace.dependencies]
qitech_framework = { git = "https://github.com/qitechgmbh/qitech_framework", rev = "<framework commit>" }
qitech_lib = { git = "https://github.com/qitechgmbh/qitech_lib.git", rev = "<same rev as the framework>" }
tokio = { version = "1", features = ["full"] }
```

```toml
# machines/my_machine/Cargo.toml
[dependencies]
qitech_framework.workspace = true
qitech_lib.workspace = true
```

A common layout is one library crate per machine type, each with its own `schemas/` folder, plus one binary crate (`app`) that registers them all with the Runtime (step 6).

## 2. Write the schema

Create `schemas/<machine_name>.yaml`. The file name **must** be the struct name in `snake_case`: `MyMachine` becomes `schemas/my_machine.yaml`. The macros look the file up by that name at compile time.

```yaml
qms_version: 1.0     # schema format version
revision: 1          # increase when you change this machine's interface

identification:
  name: my_machine
  vendor_id: 1       # your vendor id (1 = QiTech GmbH), see below
  machine_id: 42     # unique for this vendor

config:
  speed:
    target: !meter_per_second
    limit: !meter_per_second
  mode: !enum [idle, running, cleaning]

state:
  running: !boolean
  error: !?string    # "?" = nullable

measurements:
  speed: !meter_per_second
  temperature: !?degree_celsius

commands:
  start: !command
  stop: !command

events:
  overheated: !event
```

- **`vendor_id` + `machine_id`** identify the machine *type* everywhere: on the wire, in the EtherCAT EEPROM and in the controller. Don't reuse a `machine_id` for a different machine.
- **Vendor ids** are listed in the framework's `qitech_framework_core/vendors.toml`, which currently only contains QiTech (`1`). If you build machines under your own name, ask for a vendor id to be added there, so your ids can't collide with anyone else's. Within your vendor id, you choose the `machine_id`s.
- **Resource paths** come from nesting: the config above defines `speed.target`, `speed.limit` and `mode`.
- **Types**: `!boolean`, `!integer`, `!string`, `!enum [...]` (or a map `{ name: value }`), `!float` (also `!fraction`, `!percentage`), or any unit from `qitech_framework_core/quantities.toml`, written in snake_case (`MeterPerSecond` → `!meter_per_second`).

See [resources.md](resources.md) for what each kind of resource is for.

## 3. Define the struct

```rust
use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::Machine;                          // the derive macro
use qitech_framework::machine::{
    ConfigProperty, StateProperty, Measurement, EventEmitter,
};
use qitech_lib::units::Velocity;

#[derive(Machine)]
pub struct MyMachine {
    // --- hardware ---
    drive: Rc<RefCell<SomeDriveDevice>>,

    // --- config ---
    speed_target: ConfigProperty<Velocity>,
    speed_limit: ConfigProperty<Velocity>,
    mode: ConfigProperty<Mode>,

    // --- state ---
    running: StateProperty<bool>,

    // --- measurements ---
    speed: Measurement<Velocity>,
    temperature: Measurement<Option<ThermodynamicTemperature>>,

    // --- events ---
    overheated: EventEmitter<()>,
}
```

`#[derive(Machine)]` reads the schema and implements `MachineDescriptor`, which provides:

- `MyMachine::IDENTIFICATION`: the `vendor_id`/`machine_id` from the schema, and
- `MyMachine::SCHEMA`: the schema text, embedded into the binary.

It does **not** implement the `Machine` trait itself. You do that in step 5.

### Enum properties

Config properties can use your own enums. Derive `EnumProperty`, and keep the variants unit-only, matching the schema's variant names in `snake_case`:

```rust
use qitech_framework::EnumProperty;

#[derive(Debug, Clone, Copy, PartialEq, EnumProperty)]
pub enum Mode { Idle, Running, Cleaning }
```

## 4. Implement `MachineBuild`

`build` runs once per machine instance during Runtime initialization. It gets a `BuildContext` and returns the finished machine.

```rust
use qitech_framework::machine::{BuildContext, BuildResult, MachineBuild};
use qitech_framework::machine_build;
use qitech_lib::units::velocity::meter_per_second;

impl MachineBuild for MyMachine {
    #[machine_build(MyMachine)]
    fn build(ctx: &mut BuildContext<'_>) -> BuildResult<Self> {
        // --- hardware ---
        let drive = ctx.find_ethercat_device::<SomeDriveDevice>(1)?;   // role 1

        // --- config ---
        let speed_target = ctx
            .config::<meter_per_second>("speed.target")
            .default(0.5)
            .minimum(0.0)
            .maximum(2.0)
            .on_external_changed(|m: &mut MyMachine| m.apply_speed())
            .build()?;

        let speed_limit = ctx
            .config::<meter_per_second>("speed.limit")
            .default(2.0)
            .forbid_external_writes()                // machine-controlled for now
            .build()?;

        let mode = ctx.config::<Mode>("mode").default(Mode::Idle).build()?;

        // --- commands ---
        ctx.command::<MyMachine>("start")
            .can_execute(|m| m.start_capability())
            .execute(|m| m.start())
            .build()?;

        ctx.command::<MyMachine>("stop")
            .execute(|m| m.stop())
            .build()?;

        Ok(Self {
            drive,
            speed_target,
            speed_limit,
            mode,
            running: ctx.state::<bool>("running").initial(false).build()?,
            speed: ctx.measurement::<meter_per_second>("speed").build()?,
            temperature: ctx
                .measurement::<Option<degree_celsius>>("temperature")
                .build()?,
            overheated: ctx.event("overheated").build()?,
        })
    }
}
```

### The type parameter on `config` / `state` / `measurement`

The type in the turbofish says **how values are given and converted**. It is not always the type of the handle you get back:

| Schema type | Turbofish | Resulting handle |
|---|---|---|
| `!boolean` | `::<bool>` | `ConfigProperty<bool>` |
| `!integer` | `::<i64>` (or another integer type) | `ConfigProperty<i64>` |
| `!float` | `::<f64>` | `ConfigProperty<f64>` |
| `!millimeter` | `::<millimeter>` (the **unit**) | `ConfigProperty<Length>` (the **quantity**) |
| `!?millimeter` | `::<Option<millimeter>>` | `Measurement<Option<Length>>` |
| `!string` | `::<heapless::String<N>>` | `ConfigProperty<heapless::String<N>>` |
| `!enum [...]` | `::<MyEnum>` | `ConfigProperty<MyEnum>` |

Values you pass to `.default(..)`, `.minimum(..)` and `.initial(..)` are in the turbofish type, so `.default(0.5)` in the example means 0.5 m/s. Strings use `heapless::String` because resource values have to live inline, with no heap allocation.

### What `#[machine_build]` checks at compile time

For every `ctx.config("…")` call, it checks that:

- the path exists in the schema's `config` section,
- the `Option<…>` wrapper matches the property's nullability, and
- the unit type matches the schema's quantity.

Enum and string configs **must** have an explicit turbofish.

Everything else, including state, measurements, commands, events and duplicate registrations, is checked at **build time** and returned as a `BuildError`.

### Hardware lookup

| Method | Finds |
|---|---|
| `find_ethercat_device::<T>(role)` | the EtherCAT device assigned to this machine with that role (see step 6) |
| `find_ethercat_device_and_addr::<T>(role)` / `find_ethercat_device_addr(role)` | the same, plus or only the device address |
| `get_ethercat_device::<T>(index)` | the n-th hardware item assigned to this machine, if it's an EtherCAT device |
| `get_modbus_rtu_device::<T>(index)` | the n-th hardware item, if it's a Modbus RTU device |
| `get_xtrem_device::<T>(index)` / `get_xtrem_probe(index)` | the n-th hardware item, if it's an Xtrem device, and its discovery data |
| `get_ethercat_interface()` | the EtherCAT thread channel, for mailbox/CoE access |
| `ctx.ident()` | this instance's `vendor:machine:serial` |

Devices are returned as `Rc<RefCell<T>>`. Keep that in your struct and `borrow_mut()` it in `act`.

### Rules for `build`

- **Return `Err`, don't panic**, when hardware is missing or doesn't fit. The build is rolled back, the controller gets `MachineBuildCompleted { result: Err(..) }`, and the other machines still start.
- Register every resource in the schema that the machine uses, **once**. Registering the same path twice is a `DuplicateResource` error.
- `on_external_changed` and `command(...)` take the machine type (`|m: &mut MyMachine|`, `::<MyMachine>`). Using a different type returns `IllegalMachineType`.

## 5. Implement `Machine`

```rust
use std::time::Duration;
use qitech_framework::machine::{
    ActError, ActErrorImpact, ActErrorKind, ActResult, Machine, OperationCapability,
};

impl Machine for MyMachine {
    fn act(&mut self, dt: Duration) -> ActResult {
        // --- read hardware ---
        let actual = self.drive.borrow().speed();
        self.speed.set(actual);

        // --- control logic ---
        let target = if *self.running.get_ref() {
            self.speed_target.get().min(self.speed_limit.get())
        } else {
            Velocity::new::<meter_per_second>(0.0)
        };
        self.drive.borrow_mut().set_speed(target);

        // --- faults ---
        if self.drive.borrow().is_overheated() {
            self.overheated.emit(&());
            return Err(ActError {
                kind: ActErrorKind::HardwareFault("drive overheated".into()),
                impact: ActErrorImpact::Degraded,
            });
        }

        Ok(())
    }
}

impl MyMachine {
    fn apply_speed(&mut self) -> ActResult { /* … */ Ok(()) }

    fn start_capability(&self) -> OperationCapability {
        if self.mode.get() == Mode::Running {
            OperationCapability::Allowed
        } else {
            OperationCapability::Forbidden { reason: "switch mode to running first".into() }
        }
    }

    fn start(&mut self) -> ActResult { self.running.set(true); Ok(()) }
    fn stop(&mut self) -> ActResult { self.running.set(false); Ok(()) }
}
```

### Rules for `act`

`act` runs inside the real-time loop. Everything in the Runtime shares one cycle budget, which is 100 µs by default.

- **Never block.** Don't sleep, don't do blocking I/O and don't wait on locks. Drivers in `qitech_lib` follow a *send request / handle response* pattern. Call those methods, don't wait.
- **Use `dt` for timing.** For slow devices, count down a timer and only send a request when it expires. Both `qitech_laser` (6 ms) and `xtrem_scale` (20 ms) do this.
- **Avoid allocating** on the hot path where you can.
- **Report errors through `ActResult`.** `ActErrorImpact` decides what happens:
  - `Ignore`: the error is discarded; the machine keeps running.
  - `Degraded`: the machine keeps running with reduced capability.
  - `Irrecoverable`: the machine is **removed** from the Runtime, and `RuntimeEvent::RemovedMachine` is reported. Use it for broken hardware, for example after a grace period with no responses, as `qitech_laser` does.

### Reading and writing resources

| Handle | Read | Write |
|---|---|---|
| `ConfigProperty<T>` | `get()`, `get_ref()`, `get_as::<unit>()` | `set(v)` → `Result<changed, ConstraintViolation>`, `set_as::<unit>(f64)`, `reset()`, `set_default`, `set_min/max[_clamped]`, `set_allowed`, `allow_external_write()` / `forbid_external_write(reason)` |
| `StateProperty<T>` | `get()`, `get_ref()`, `get_as::<unit>()` | `set(v)` → `bool` (true if the value changed) |
| `Measurement<T>` | `get()`, `get_ref()`, `get_as::<unit>()` | `set(v)`, `set_as::<unit>(f64)` |
| `EventEmitter<T>` | — | `emit(&payload)` (the payload is serialized to JSON) |

Every write is recorded and sent to the controller automatically, so you don't have to do anything extra.

`StateProperty::set` returning `bool` makes edge detection easy. For example, `if self.in_tolerance.set(ok) && !ok { self.out_of_tolerance.emit(&()) }` emits the event only on the change to "out of tolerance".

### Optional: reading other machines

A machine can read another machine's resources by implementing `subscribe`. The controller sets up a subscription with a `SubscribeMachine` request:

```rust
fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
    self.upstream_speed = Some(ctx.measurement::<Velocity>("speed")?);   // RemoteProperty<Velocity>
    Ok(())
}

fn unsubscribe(&mut self, _provider: MachineInstanceIdentification) {
    self.upstream_speed = None;       // required: reading after unsubscribe panics
}
```

The value you read is the other machine's value **at the end of the previous cycle**. The default `subscribe` implementation rejects every subscription with `UnsupportedMachine`.

> **Warning:** the Runtime currently doesn't call `unsubscribe` when a subscription ends, so reading a handle after an `UnsubscribeMachine` request panics and takes the Runtime down. Until that's fixed, avoid unsubscribing machines that are running. See [concepts/subscriptions.md](concepts/subscriptions.md) for how subscriptions and handle validity work.

## 6. Register the machine and assign hardware

In `main`, configure the Runtime: enable the buses you need, register your machine type, and tell the Runtime which hardware belongs to which machine **instance**.

```rust
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::runtime::{EtherCATConfig, RuntimeConfiguration};

let config = RuntimeConfiguration::new()
    .ethercat(EtherCATConfig::default())
    .machine::<MyMachine>();
```

An instance is identified by `vendor:machine:serial`. How hardware gets its serial depends on the bus.

### EtherCAT: the identity is stored on the device

Each EtherCAT device that belongs to a machine stores a `MachineDeviceInfo` in its EEPROM: machine vendor, machine id, machine **serial** and **role**. During startup the Runtime reads these and groups the devices by machine instance. `find_ethercat_device::<T>(role)` then picks the device with that role.

New devices have no identity yet. Assign one with a `WriteMachineDeviceInfo { machine_ident, role, subdevice_index }` request from the controller, then restart the Runtime.

Roles are your own convention per machine type. For example, `1` could be "main I/O terminal" and `2` "drive". Document them next to your `build` function.

### Modbus RTU: the identity is set in code

```rust
.modbus_rtu_device::<LaserDevice>(
    "/dev/serial/by-path/pci-…-usb-0:1:1.0-port0",   // stable serial path
    MyMachine::IDENTIFICATION.unique(1),              // → vendor:machine:1
    1,                                                // Modbus slave id
    None,                                             // Option<ModbusSettings>
)
```

Use `/dev/serial/by-path/…` so the same physical port always maps to the same machine. In `build`, get the device with `get_modbus_rtu_device::<T>(0)`.

### Xtrem: the identity is set in code

```rust
.xtrem(XtremConfig { bus, ..Default::default() })
.xtrem_device::<XtremScale>(0x03, ScaleV1::IDENTIFICATION.unique(1), ScaleMode::Poll)
.xtrem_device::<XtremScale>(0x04, ScaleV1::IDENTIFICATION.unique(2), ScaleMode::Poll)
```

Each bus device id maps to one machine instance. This example creates **two instances** of the same machine type. In `build`, use `get_xtrem_device::<T>(0)`.

### Runtime tuning (optional)

```rust
RuntimeConfiguration::new()
    .cycle_period(Duration::from_micros(100))       // default 100 µs
    .export_interval(Duration::from_secs_f64(1.0 / 32.0))  // report rate, default 32 Hz
    .requests_per_cycle_max(10)                      // default 10
```

## 7. Run it

Pick a controller:

```rust
#[tokio::main]
async fn main() {
    let config = /* … */;

    // interactive terminal UI
    qitech_framework::run_with_tui(config, TuiConfiguration::default()).await.unwrap();

    // or: the Hub with your own listeners/actors
    // qitech_framework::run_with_hub(config, HubConfiguration::new()).await.unwrap();

    // or: no controller, just print what the Runtime sends
    // qitech_framework::run_debug(config);
}
```

### Hardware permissions

EtherCAT needs raw network access, and the real-time loop benefits from real-time scheduling and locked memory. Binaries you build don't have these permissions, so grant them to the binary before it starts. The easiest way is a Cargo **runner** in your project. Cargo calls it with the built binary as its first argument, on every `cargo run`.

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
runner = "scripts/run-linux"
```

```bash
#!/usr/bin/env bash
# scripts/run-linux   (chmod +x)
set -e

BINARY="$1"
shift

# raw sockets (EtherCAT), real-time priority, locked memory, device access
sudo setcap 'cap_dac_override,cap_net_raw,cap_sys_nice,cap_ipc_lock=eip' "$BINARY"

# serial adapters for Modbus RTU
if compgen -G "/dev/ttyUSB*" > /dev/null; then
    sudo chown "root:$(id --group)" /dev/ttyUSB*
fi

exec "$BINARY" "$@"
```

This is the same script as `bin/run-linux` in the framework repository. It prompts for `sudo` because `setcap` has to be run again after every rebuild. On a deployed machine, set the capabilities once at install time (for example in your package or systemd unit) instead.

### Start it

```sh
cargo run            # or: cargo run -p app   in a workspace
```

EtherCAT needs a free network interface with the devices connected. Modbus RTU needs the serial adapter plugged in at the path you configured.

In the TUI you should see:

- the init events,
- `MachineBuildCompleted` for your instance, and
- your resources, where you can edit config properties and run commands.

## Troubleshooting

| Symptom | Cause |
|---|---|
| Compile error `Could not find schema for 'MyMachine'` | The schema file isn't at `schemas/my_machine.yaml` next to the `Cargo.toml` of the crate that defines the machine. Check the name (struct name in snake_case). |
| Errors like "expected `EL2004`, found `EL2004`" or "`EthercatDevice` is not implemented" | Your `qitech_lib` revision differs from the framework's. Use exactly the same `rev` (step 1). |
| `Operation not permitted` when EtherCAT starts | The binary is missing its capabilities. Set up the runner, or run `setcap` (step 7). |
| Compile error `unknown config property` | A path in `ctx.config("…")` doesn't match the schema. Remember nesting: `speed.target`. |
| Compile error `cannot infer type for config property` | Enum or string config without a turbofish. |
| Compile error `expected quantity type …` / `nullable property requires Option<T>` | The turbofish doesn't match the schema's unit or nullability. |
| Machine never appears | No hardware is assigned to that `vendor:machine:serial`. EtherCAT: write the device identity. Modbus/Xtrem: check the path or device id and look for `…NotFound` init events. |
| `MachineBuildCompleted` with `ExpectedEtherCATDeviceWithRole` | No device with that role is assigned to this instance. |
| `…DeviceTypeMismatch` | The device at that index or role is a different driver type than you requested. |
| `IllegalResourcePath` / `IllegalResourceType` at build time | A state, measurement, command or event path or type doesn't match the schema. |
| `MachineTypeNotRegistered` | Hardware claims a machine type that you didn't register with `.machine::<M>()`. |
| Runtime panics with `bump allocator exhausted` | All machines together use more resource storage than the 4 KB pool per resource kind. |
| Cycle overruns in timings | `act` is too slow: something blocks, or a device is being polled on every cycle. |
