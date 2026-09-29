# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

QiTech Framework: a Rust framework for building EtherCAT and Modbus (RTU) machines. It is experimental, and breaking changes between commits are expected. The `jse-v2` branch is partway through a refactor (see `todo.txt`), so **`cargo check --workspace` does not currently compile**. `qitech_framework`, `qitech_framework_ctrl` and `qitech_framework_tui` all have errors from half-migrated session and runtime APIs. `qitech_framework_core` and `qitech_framework_macros` do build. Check with individual crates (`-p`) rather than assuming the whole workspace builds.

## Commands

The toolchain is pinned to Rust 1.90.0 in `rust-toolchain.toml`, with edition 2024. `shell.nix` provides the system deps (`pkg-config`, `systemdLibs` for libudev/serialport).

```sh
cargo check -p qitech_framework_core          # check a single crate
cargo clippy -p <crate>                       # the editor is configured to use clippy on save
cargo fmt                                     # rustfmt.toml: one import per `use`, grouped std/external/crate
cargo test -p <crate> <test_name>             # run a single test
cargo run -p beckhoff_el2004                  # run an example app (names = examples/apps/* package names)
```

Formatting convention: `imports_granularity = "Item"`, so every imported item gets its own `use` line. Don't merge them into `use foo::{a, b}`.

There are very few tests at the moment. `qitech_framework_core/tests/session_unix_tokio.rs_` is disabled on purpose (note the trailing `_`), and everything under `qitech_framework_ctrl/legacy/` (ClickHouse, the old REST API, and tests that use testcontainers) is outside the module tree and does not get compiled.

## Workspace layout

- **`qitech_framework_core`**: shared types with no hardware dependencies. It is used by both sides of the runtime↔controller split:
  - `schema/`: YAML machine-schema parser (`MachineSchema`, behind the `schema` feature).
  - `report/`: runtime → controller data (`RuntimeReport`, events, errors such as `BuildError` and `ActError`).
  - `request.rs`: controller → runtime requests.
  - `session/`: the `RuntimeTransport` (sync) and `ControllerTransport` (async) traits, plus `protocol` messages serialized with postcard. The unix, mpsc, runtime and controller session providers are commented out mid-refactor.
  - `build.rs` generates `vendors` (from `vendors.toml`) and the `with_uom!` macro and quantity types (from `quantities.toml`) into `OUT_DIR`.
- **`qitech_framework_macros`**: proc macros that read the machine's YAML schema **at compile time**.
  - `#[derive(Machine)]` finds `<crate>/<schema-dir>/<snake_case(StructName)>.yaml`. `schema-dir` defaults to `schemas` and can be overridden with `[package.metadata.qitech] schema-dir`. The derive embeds the schema and implements `MachineDescriptor` (vendor_id and machine_id come from the schema's `identification`).
  - `#[machine_build(MachineType)]` goes on `MachineBuild::build`. It rewrites and validates `ctx.config::<T>("name")` calls against the schema, so an unknown property name or a mismatched Rust type is a compile error.
  - `#[derive(EnumProperty)]` supports enum-valued properties.
- **`qitech_framework`**: the machine-author-facing crate. It re-exports core types, the macros, and `qitech_lib::units`.
  - `machine/`: the `Machine` trait (`act(dt)` is the cyclic update, plus `subscribe`/`unsubscribe`), the `MachineBuild` trait, and `BuildContext` together with the builders for config properties, state properties, measurements, commands, events and hardware lookup (e.g. `ctx.find_ethercat_device::<EL2004>(idx)`).
  - `resource/`: property registry, journals (property/event history), constraints, and the conversion between Rust types and `ScalarValue`.
  - `runtime/`: the current `Runtime`, a synchronous tick loop that drives EtherCAT, Modbus RTU and Xtrem buses, calls `act` on every machine instance, handles requests, and exports `RuntimeReport`s over a `RuntimeTransport`. Machines are registered as types in a `MachineRegistry` and then instantiated through their build fn.
  - `runtime_2/`: the replacement runtime and init flow that is being built now. Buses (such as the Modbus RTU bus config) are declared explicitly and machines are assigned to them.
  - Hardware drivers (EtherCAT HAL, Beckhoff and WAGO modules, etc.) come from the external `qitech_lib` git dependency, pinned by `rev` in the root `Cargo.toml`.
- **`qitech_framework_ctrl`**: the async (tokio) controller. It manages sessions with runtimes, keeps the schema and machine registries, dispatches requests and transactions, and runs pluggable `Actor`/`Listener` modules (`modules/`, for example rest_api and clickhouse).
- **`qitech_framework_tui`**: a ratatui/crossterm terminal UI that works as a controller client. It inspects machines and edits config, state, commands, measurements and logs.
- **`examples/apps/*`**: runnable machines, each a workspace member with its own `schemas/*.yaml` file. `examples/concepts/*.rs` are concept sketches that are not wired into the workspace.

## Writing a machine (the pattern examples follow)

1. Write `schemas/<snake_name>.yaml` containing `qms_version`, `revision`, `identification {name, vendor_id, machine_id}`, and property sections such as `config:` (e.g. `led1_on: !boolean`).
2. Add a `#[derive(Machine)] struct` whose fields are hardware handles (`Rc<RefCell<Device>>`) and property handles (`ConfigProperty<T>`, `StateProperty<T>`, `Measurement`, …).
3. `impl Machine` (override `act` for cyclic logic) and `impl MachineBuild` with `#[machine_build(Type)] fn build(ctx: &mut BuildContext)`. Wire callbacks such as `.on_external_changed(|m: &mut Self| ...)` in `build`.
4. Register the machine on the runtime configuration (`RuntimeConfiguration::new().ethercat(...).machine::<M>()`).

Machines are single-threaded (they use `Rc`/`RefCell` and are not `Send`). The runtime owns them in its synchronous loop, and anything async lives on the controller side.
