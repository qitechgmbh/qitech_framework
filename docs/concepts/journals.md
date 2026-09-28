# Journals

A **journal** is the Runtime's append-only log of what happened to a machine's [resources](../resources.md) since the last report. Every change a controller needs to know about, such as a config write, a state change, a command becoming unavailable or an event being emitted, is written to a journal first. On each report export, the journals are drained into the `RuntimeReport`.

Journals are the reason a controller can rebuild the complete, ordered history of every machine without the Runtime keeping any history itself. That's what the protocol means by "reports are deltas" (see [protocol.md](../protocol.md)).

Code: `qitech_framework/src/resource/journal.rs`. Record types: `qitech_framework_core/src/report/`.

## What gets journaled

There is one journal per kind of change, grouped in `Journals`:

| Journal | Record type | Written when |
|---|---|---|
| `config_property` | `ConfigPropertyEvent` | a config property is registered, written (by the machine or the controller), or its default, write permission or constraints change |
| `state_property` | `StatePropertyEvent` | a state property is registered, or its value **changes** |
| `command` | `CommandEvent` | a command is registered, executed, or its `can_execute` result changes |
| `event` | `String` (the payload as JSON) | a machine calls `EventEmitter::emit` |

**Measurements are not journaled.** They change too often to record every value, so the Runtime *samples* them once per export instead (`MeasurementSnapshot`). This is the key difference between a measurement and a state property. See [resources.md](../resources.md#choosing-the-right-kind).

### The record

Every entry is wrapped in an `EventRecord<T>`:

```rust
struct EventRecord<T> {
    timestamp: DateTime<Utc>,                // when it happened, not when it was exported
    machine:   MachineInstanceIdentification,
    path:      String,                       // resource path
    event:     T,                            // what happened
}
```

The timestamp is taken **when the record is written**, so a controller sees the real time of each change within the ~31 ms export window.

### Record types in detail

```rust
enum ConfigPropertyEvent {
    Registered { default, capability, constraints }, // initial snapshot
    Written { value, origin, outcome },              // every write attempt
    DefaultChanged(value),
    CapabilityChanged(OperationCapability),          // allow/forbid external writes
    ConstraintsChanged(Constraints),
}

enum StatePropertyEvent {
    Registered { value },
    ValueChanged { value },
}

enum CommandEvent {
    Registered,
    CapabilityChanged(OperationCapability),
    Executed(Result<(), CommandExecuteError>),
}
```

A `Written` record captures the complete attempt, not just successful changes:

- `origin` is `Machine` (the machine called `set`) or `Request { request_id }` (a controller request), which links the record to the request that caused it.
- `outcome` is `Accepted { changed }`, where `changed: false` means the value was already equal, or `Rejected(reason)`, for example `NotWritable` or `ConstraintViolation`.

A rejected write is journaled *with* the value that was attempted, so the history also shows what someone *tried* to do.

## Lifecycle

```
            build                          every cycle                    every export (1/32 s)
  ┌──────────────────────┐      ┌───────────────────────────┐      ┌──────────────────────────────┐
  │ Registered records   │      │ handles call record()     │      │ drain_with(): records move   │
  │ → temporary journals │─────►│ requests call record()    │─────►│ into RuntimeReport, journal  │
  │ imported on success  │      │ (append to Vec)           │      │ is empty again               │
  └──────────────────────┘      └───────────────────────────┘      └──────────────────────────────┘
```

### 1. Build: transactional registration

While a machine builds, its `Registered` records go into **temporary journals** (`BuildContext::journals_temp`), not the real ones. The Runtime then either:

- **imports** them into the real journals (`Journal::import`) if `build` succeeds, or
- **drops** them if `build` fails, along with the resource allocations, so the controller never sees resources of a machine that doesn't exist.

This mirrors the rollback of the resource storage (`PropertyRegistrar`).

### 2. Running: recording

Resource handles don't talk to `Journals` directly. Each one holds a **`JournalHandle<T>`**, a cheap clone of the journal's shared buffer plus its own `ResourceKey` (machine and path). So `ConfigProperty::set` can call `self.journal.record(event)` without passing the machine or path around:

```rust
pub(crate) struct Journal<T>       { buffer: Rc<RefCell<Vec<EventRecord<T>>>> }
pub(crate) struct JournalHandle<T> { buffer: Rc<RefCell<Vec<EventRecord<T>>>>, key: ResourceKey }
```

The Runtime itself writes to the journals directly (`Journal::record(machine, path, event)`) in two cases:

- **controller requests**: external config writes and command executions, in `runtime/request.rs`;
- **capability scans**: on each export, the Runtime evaluates every command's `can_execute` and records `CapabilityChanged` when the result differs from last time.

Because everything runs on the one Runtime thread, `Rc<RefCell<…>>` is enough and no locking is needed.

### 3. Export: draining

In `Runtime::export_report_if_due`, each journal is drained **in full** into the matching `MachinesReport` list (`config_property_records`, `state_property_records`, `command_records`, `event_records`), and the report is sent. Nothing stays behind, and nothing is ever dropped. That's what makes the report stream complete.

## Ordering guarantees

- **Within one journal**, records are in the order they were written. Timestamps are non-decreasing in practice. They come from the wall clock (`Utc::now()`), so a clock adjustment can break that.
- **Across journals**, there is no shared order. A report contains four separate lists. To interleave them, for example "the config was written, then the state changed", sort by `timestamp`.
- **Across reports**, all records in report *n* happened before the records in report *n + 1*.
- **`Registered` comes first.** A resource's `Registered` record is in the first report, before any other record for that resource.

## For controller authors

- To rebuild a resource's current value: start from `Registered`, then apply every `Written` with `outcome: Accepted { changed: true }` (config) or every `ValueChanged` (state) in order.
- Show `Rejected` writes to the user. They explain why a setting "didn't stick".
- Match `origin: Request { request_id }` against your own pending requests to show who changed what.
- Never skip a report. A missing report means missing records, and your rebuilt state diverges silently.

## Known issues

These are the places where the current implementation doesn't fully deliver the guarantees above:

- **Handles write to the real journal during build.** Only `Registered` goes to the temporary journal. A `ConfigProperty`, `StateProperty` or `EventEmitter` created in `build` already points at the real journal. If `build` calls `set` or `emit` before returning, that record:
  - is exported even if the build later fails, and
  - lands **before** the `Registered` record, because the temporary journal is imported afterwards.
- **Clamping changes values without recording them.** `ConfigProperty::set_min_clamped` and `set_max_clamped` may change the current value and the default, but they only record `ConstraintsChanged`. A controller rebuilding state from the journal ends up with the old value.
- **Config `set` records no-op writes.** `ConfigProperty::set` with an unchanged value still records `Written { changed: false }`, while `StateProperty::set` records nothing in that case. A machine that calls `config.set(x)` every cycle produces about 300 records per report.
- **Allocation on the hot path.** Every record allocates a `String` for its path (`path.to_string()`), events also allocate for their JSON payload, and the buffers are `Vec`s that grow without limit until the next export. This is fine at current rates, but it contradicts the "no allocation in `act`" goal.
- **`import` isn't timestamp-ordered.** It appends the temporary records after whatever is already in the real journal, which is harmless only while nothing writes to the real journal during build (see the first point).
