# Resources

A **resource** is anything a machine exposes to the outside world under a name. Together, a machine's resources form its **interface**: everything a controller, a user interface or another machine can see or do with it.

There are five kinds of resource:

| Kind | Direction | Has a value? | Who changes it |
|---|---|---|---|
| **Config property** | in / out | yes | the controller *and* the machine |
| **State property** | out | yes | only the machine |
| **Measurement** | out | yes (sampled) | only the machine |
| **Command** | in | no | triggered by the controller |
| **Event** | out | no | emitted by the machine |

In code, the kind is `ResourceKind` (`qitech_framework_core/src/report/mod.rs`).

## Value resources and signal resources

The five kinds fall into two groups:

- **Value resources** (config properties, state properties and measurements) always have a *current value*. The runtime stores that value in memory, reports it, and other machines can subscribe to it.
- **Signal resources** (commands and events) have no stored value. A command is an action you trigger, and an event is a notification that something happened. They exist only as records in the report stream.

Keep this split in mind when you read the runtime code: `ResourceRegistry` only holds value resources. Commands and events are handled separately.

## Addressing a resource

A resource is uniquely identified by two things:

1. the **machine instance** it belongs to (`MachineInstanceIdentification`, written `vendor:machine:serial`), and
2. its **resource path**, a dot-separated name derived from the schema.

In code this pair is `ResourceKey { ident, path }`.

The resource path comes from how keys are nested in the schema. Nested keys are joined with dots until a type tag (such as `!millimeter`) appears:

```yaml
config:
  diameter:
    target: !millimeter        # path: diameter.target
    tolerance:
      upper: !millimeter       # path: diameter.tolerance.upper
      lower: !millimeter       # path: diameter.tolerance.lower
```

Paths are unique *within a kind*, so a config property and a measurement may both be called `diameter`. To address a resource fully, you therefore need the machine instance, the kind and the path.

## Declaring resources

Every resource is declared twice:

1. in the machine's **YAML schema** (`schemas/<machine_name>.yaml`), which tells controllers what exists, and
2. in the machine's **`build` function**, which creates the Rust handle the machine uses at runtime.

The `#[machine_build(MyMachine)]` macro reads the schema at compile time. It rejects `ctx.config("…")` calls whose path isn't in the schema, whose `Option<T>` usage doesn't match the property's nullability, or whose unit type is wrong.

### Types

Value resources use type tags:

- `!boolean`, `!integer`, `!string`, `!enum`
- `!float`, or a physical unit such as `!millimeter`. The available units are listed in `qitech_framework_core/quantities.toml`. On the Rust side, a unit maps to a `uom` quantity such as `Length`.
- A `?` after the `!` makes the value **nullable** (`!?millimeter`). On the Rust side it becomes an `Option<T>`.

Measurements only support booleans, integers and floats/units. Strings and enums are not allowed.

## The five kinds in detail

### Config property

A **setting** that the controller and the machine can both change. Examples are a target diameter, a tolerance, or whether an LED is on.

```yaml
config:
  led1_on: !boolean
```
```rust
let led = ctx.config::<bool>("led1_on")
    .default(false)
    .on_external_changed(|m: &mut MyMachine| m.apply_led())
    .build()?;                                   // -> ConfigProperty<bool>
```

- **Constraints** such as `minimum`/`maximum` (numbers), `length_min`/`pattern` (strings) and the allowed variants (enums) are checked on every write, whoever makes it. The machine can change them at runtime with `set_min`, `set_max` and `set_allowed`.
- Each property has a **default**, and `reset()` restores it.
- The machine decides whether outside writes are allowed right now, using `allow_external_write()` and `forbid_external_write(reason)`. It can also forbid them from the start with `forbid_external_writes()` on the builder. The reason is shown to the controller.
- `on_external_changed` runs a callback after every controller write to the property. It does **not** run when the machine changes the value itself. (Currently it also runs when the write was rejected or left the value unchanged, so check the value in the callback if that matters.)
- Every write is recorded as a `ConfigPropertyEvent::Written` with its origin (machine or external) and outcome (accepted, changed or not, or rejected with a reason), so there is a full history of every change.

### State property

A value that **the machine owns** and that is visible but read-only from outside. Examples are "in tolerance", the current mode, or an error flag.

```yaml
state:
  in_tolerance: !boolean
```
```rust
let mut in_tol = ctx.state::<bool>("in_tolerance").initial(false).build()?;
in_tol.set(true);                                // recorded only when it actually changes
```

State properties are **event-sourced**: the report contains every `ValueChanged`, not just the latest value. Use them for values that change occasionally and whose history matters.

### Measurement

A value that the machine updates **continuously**. Examples are a measured diameter, a temperature or a weight.

```yaml
measurements:
  diameter: !millimeter
  roundness: !?float
```
```rust
let mut diameter = ctx.measurement::<millimeter>("diameter").build()?;   // -> Measurement<Length>
diameter.set_as::<millimeter>(1.75);
```

Measurements are **sampled**, not recorded as events. On each report export (every 1/32 s by default), the runtime takes a `MeasurementSnapshot` of the current value. The machine may update a measurement every cycle, but a controller only sees the sampled values.

A schema can ask for per-sampling-window **statistics** on numeric measurements: `min`, `max`, `avg` and `stddev`. All of them are off by default.

Use a measurement for high-rate numeric signals. Use a state property when you need every change.

### Command

An **action** the controller can trigger. It takes no arguments and returns success or an error. Examples are "zero the scale", "tare" and "start homing".

```yaml
commands:
  tare: !command
```
```rust
ctx.command::<MyMachine>("tare")
    .can_execute(|m| m.tare_capability())         // optional: Allowed / Forbidden { reason }
    .execute(|m| m.tare())
    .build()?;
```

- `can_execute` tells the controller *in advance* whether the command is currently available, and why not if it isn't. The runtime checks it on every export and reports `CommandEvent::CapabilityChanged` when the answer changes, so user interfaces can grey out buttons.
- Execution is recorded as a `CommandEvent`.

If an action needs a parameter, model the parameter as a config property and have the command read it.

### Event

A **notification** that something happened at a point in time, possibly with structured data attached. Examples are "went out of tolerance" and "cycle finished".

```yaml
events:
  out_of_tolerance: !event
```
```rust
let mut out_of_tol = ctx.event::<OutOfTolerance>("out_of_tolerance").build()?;
out_of_tol.emit(&OutOfTolerance { /* … */ });
```

Events have no current value. They only exist as timestamped records in the report stream. Event fields can be nested objects, lists, enums, strings, booleans, integers or floats/units (see `EventFieldKind`).

Use an event when the *occurrence* is what matters. If you'd ask "what is it right now?", use a state property instead.

## Choosing the right kind

```
Can the operator/controller change it?
├── yes, it's a value           → Config property
├── yes, it's an action         → Command
└── no, the machine produces it
    ├── it has a current value
    │   ├── changes rarely, every change matters → State property
    │   └── changes continuously, sampling is fine → Measurement
    └── it's a one-off occurrence → Event
```

## How the runtime handles resources

This section is for people working on the framework itself.

- **Storage.** Value resources live in a `PropertyRegistry` per kind (config, state and measurement) inside `ResourceRegistry` (`qitech_framework/src/resource/`). Each registry has two fixed-size bump allocators: **value**, written by the owning machine, and **cache**, a snapshot. Handles such as `ConfigProperty<T>` point directly into the value memory, so reading one costs a single pointer dereference.
- **Snapshots.** At the end of every cycle, `sync_cache()` copies value into cache. Other machines only ever read the cache, so within a cycle every machine sees the same consistent state of every other machine, whatever order the machines run in.
- **Registration is transactional.** Each machine build registers its resources through a `PropertyRegistrar`. If `build` fails, the registrar rolls back every allocation and registration it made.
- **History.** Config changes, state changes, command events and emitted events are written to **journals** and drained into the `RuntimeReport` on each export. Measurements are sampled at the same moment.
- **Subscriptions.** A machine can subscribe to another machine (`Machine::subscribe`) and obtain `RemoteProperty<T>` handles to its value resources through `SubscribeContext::{config, state, measurement}`. A lifetime token guards each handle. Reading one after the subscription has ended panics instead of reading stale memory.
- **Access errors.** Looking up a resource that doesn't exist, or with the wrong Rust type, yields a `ResourceAccessError` (`MachineNotFound`, `ResourceNotFound { kind, path }` or `TypeMismatch`).
