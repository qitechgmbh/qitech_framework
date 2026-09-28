# TUI

The TUI (`qitech_framework_tui`) is a terminal user interface for running and testing machines. It is a full [controller](protocol.md): it connects to the Runtime, shows every machine's resources live, and lets you change config properties, run commands and manage subscriptions from the keyboard.

Use it while developing and commissioning machines. For production setups with persistence, APIs or several clients, use the Hub (`qitech_framework_hub`).

## Starting it

The TUI runs in the **same process** as the Runtime:

```rust
use qitech_framework::{run_with_tui, TuiConfiguration};

#[tokio::main]
async fn main() {
    let config = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .machine::<MyMachine>();

    run_with_tui(config, TuiConfiguration::default()).await.unwrap();
}
```

`run_with_tui` does three things:

1. It creates an in-process session with `session::mpsc(64)`.
2. It starts the Runtime on its own thread.
3. It runs the TUI on the current tokio runtime until you quit.

When the TUI exits, `run_with_tui` waits for the Runtime thread and returns the first error from either side. See *Known issues* for how quitting currently works.

### Configuration

```rust
TuiConfiguration::default()                         // redraw at 32 Hz
    .refresh_rate(Duration::from_millis(10))        // redraw at 100 Hz (e.g. for fast scales)
```

`refresh_rate` sets how long the TUI waits for a key press before it redraws. That is also how often incoming reports are applied. The default, 1/32 s, matches the Runtime's default report rate.

### Terminal handling

The TUI switches the terminal to raw mode and the alternate screen, and enables mouse capture. It restores the terminal when it exits, and also on a **panic** anywhere in the process, through a panic hook, so a crashing Runtime doesn't leave your shell unusable.

Log output (`println!`, `tracing` to stdout) draws over the TUI. Send logs to a file or to stderr redirected elsewhere.

## Screen layout

```
┌ QiTech Control (Terminal Edition) ────────────────────────────────┐
│┌ Status ─────────────────────────────────────────────────────────┐│
││Runtime:  🟢 Running                                             ││
││EtherCAT: 🟢 Op                                                  ││
│└─────────────────────────────────────────────────────────────────┘│
│┌ Machines │ Transactions ───────────────────────────────────────┐│  ← pages
││┌ machine ───────────────────────┐                              ││
│││ laser_v1 (1)                   │                              ││  ← machine picker
││└────────────────────────────────┘                              ││
││┌ Config │ State │ Measurements │ Commands │ Events │ Subscr… ─┐││  ← resource tabs
│││ diameter.target            1.75                            │││
│││ diameter.tolerance.upper   0.05                            │││
│││ …                                                          │││
││└────────────────────────────────────────────────────────────┘││
│└───────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Status bar

**Runtime** shows the session state:

| Indicator | Meaning |
|---|---|
| 🔴 Offline | no session yet |
| 🟡 EtherCAT Discovery, Modbus RTU Discovery, XTREM Discovery, Building Machines, Finalizing, … | the Runtime is initializing. The label is the current phase, from the latest init event. |
| 🟢 Running | reports are arriving |
| 🔴 Disconnected | the session ended. The Runtime has stopped (see [protocol.md](protocol.md#roles-and-rules)). |

**EtherCAT** shows the bus state from the latest `EtherCATStateUpdate`: No Interface, Boot, Init, PreOp, PreopPdi or Op. It should be 🟢 **Op** once initialization has finished.

## Navigation

Everything is operated with the keyboard. The screen is a hierarchy of panels, and the focused one has a **blue border**.

- **↑ / ↓** move within a list. At the top or bottom of a list, they move focus **out** to the parent panel, or **into** the next panel down.
- **← / →** switch tabs. Inside the machine's resource tabs, they switch tabs even while a list is focused.
- **Enter** opens, edits or runs the selected item.
- **Space** looks at the selected item's history or details.
- **Esc** goes back one level (closes an editor, history or chart).
- **q** quits, as long as no editor or dialog is using the key.

### Key reference

| Where | Key | Action |
|---|---|---|
| Anywhere (not typing) | `q` | quit |
| Status bar | `↓` | focus pages |
| Page tabs | `←` `→` | switch between **Machines** and **Transactions** |
| Machine picker | `Enter` | open the list, `↑` `↓` to choose, then `Enter` to select or `Esc` to cancel |
| Resource tabs | `←` `→` | switch between Config, State, Measurements, Commands, Events and Subscriptions |
| **Config** | `Enter` | edit the value (only if external writes are allowed) |
| | `Space` | history of the property |
| **State** | `Space` | history of the property |
| **Measurements** | `Space` | chart of the measurement |
| **Commands** | `Enter` | run the command |
| | `Space` | history of the command |
| **Events** | `Space` | list of emitted events, then `Space` on an entry to see its payload |
| **Subscriptions** | `+` | pick a provider to subscribe to (`↑` `↓`, `Enter`, `Esc`) |
| | `-` | end the selected subscription |
| History lists | `↑` `↓`, `Space` | select a record and look at its details |
| Chart | `←` `→` | pan back and forward in time |
| | `+` `-` | zoom in and out (up to 64×) |
| Value editor | type, `Backspace` | edit the value |
| | `Enter` | send it (only if you changed something) |
| | `Esc` | cancel |
| **Transactions** | `Space` | details of the selected request and its result |
| Any detail view | `Esc` | back |

## Pages

### Machines

The **machine picker** lists every machine instance that built successfully, as `name (serial)`, for example `laser_v1 (1)`. A machine appears once its `MachineBuildCompleted` init event arrives with a success. Machines whose build failed are not listed.

The tabs below the picker show the selected machine's [resources](resources.md). Values show **N/A** until the first report delivers their `Registered` record.

| Tab | Shows | Actions |
|---|---|---|
| **Config** | every config property with its current value | edit (`Enter`), history (`Space`). In the editor, the property's constraints (min/max, allowed variants, …) are shown next to the input. |
| **State** | every state property with its current value | history (`Space`) |
| **Measurements** | every measurement with its latest sampled value | chart (`Space`). The TUI keeps the last 4096 samples per measurement, about 2 minutes at the default report rate. |
| **Commands** | every command and whether it can run right now | run (`Enter`), history (`Space`) |
| **Events** | every event type the machine can emit | list of occurrences with their JSON payload (`Space`) |
| **Subscriptions** | the providers this machine is subscribed to | subscribe (`+`), unsubscribe (`-`). See [subscriptions](concepts/subscriptions.md). |

Histories are the machine's [journal](concepts/journals.md) records for that resource, collected since the TUI started: every write with its origin and outcome, capability and constraint changes, command executions and so on.

#### Entering values

The editor starts with the current value. You type text, and the TUI converts it according to the property's schema type:

| Schema type | Enter | Example |
|---|---|---|
| `!boolean` | `true` / `false` | `true` |
| `!integer` | a whole number | `42` |
| `!float` and units | a decimal number, **in the schema's unit** | `1.75` for `!millimeter` means 1.75 mm |
| `!enum` | a variant name exactly as it appears in the schema | `running` |
| `!string` | any text | `batch-17` |

If the text can't be converted, for example `abc` for a number or an unknown enum variant, **nothing is sent**, and the editor closes without a message.

If the value converts, the TUI sends a `SetConfigProperty` request. The Runtime may still reject it (not writable, constraint violated, …). You'll see the outcome in the property's history and on the **Transactions** page.

### Transactions

Every request the TUI sends (config writes, command runs, subscribe and unsubscribe) is listed here, newest first, with its id, time, request and result. `Space` shows the details, including the full error if the Runtime rejected it.

## Connecting to a Runtime in another process

`Tui::run` accepts any `ControllerSessionProvider`, so in principle the TUI can connect to a Runtime in another process, for example over the Unix socket transport (see [protocol.md](protocol.md#transports)):

```rust
let tui = qitech_framework_tui::Tui::create(TuiConfiguration::default())?;
tui.run(provider).await?;
```

Today only the in-process transport has a ready-made provider. `session::unix::controller_tokio(path)` returns a `SessionHandshake` directly, so you would need a small `ControllerSessionProvider` wrapper around it.

## Architecture

This section is for people working on the TUI itself.

### Tasks and data flow

```
 Runtime thread            tokio task: session::run              TUI loop (main task)
┌──────────────┐  mpsc   ┌───────────────────────────┐ crossbeam ┌──────────────────────────┐
│ Runtime      │────────►│ handshake, schema sync,   │──────────►│ apply SessionMessage     │
│              │ reports │ init events, reports      │ Session-  │ to AppState              │
│              │◄────────│ forward requests          │ Message   │ poll keys → AppAction    │
└──────────────┘ requests└───────────────────────────┘◄──────────│ → RuntimeRequest         │
                                                         requests │ redraw                   │
                                                                  └──────────────────────────┘
```

- **`session.rs`** runs as a tokio task. It drives the controller session phases and forwards everything to the UI as `SessionMessage` (`Schemas`, `InitEvent`, `Report`, `Disconnected`) over a bounded crossbeam channel. It sends requests from the UI back to the Runtime.
- **`lib.rs` (`Tui::run`)** is the UI loop. Each iteration:
  1. waits up to `refresh_rate` for a key,
  2. turns it into an `AppAction`,
  3. turns that into a `RuntimeRequest`,
  4. drains all pending `SessionMessage`s into `AppState`, and
  5. redraws.

### State

`AppState` (`types.rs`) is the TUI's view of the Runtime, **rebuilt entirely from the protocol**:

- **Schemas** from the schema sync. Their paths define the rows in each tab.
- **Machines** are added by `MachineBuildCompleted { result: Ok }`.
- **Report records** are applied in `on_report`:
  - config and state `Registered` records initialize a field,
  - `Written { Accepted { changed: true } }` and `ValueChanged` update it,
  - measurement snapshots go into a ring buffer (`utils/timeseries.rs`), and
  - every record is also appended to the field's history.
- **Transactions** are appended when a request is sent, and updated when its `RuntimeResponse` arrives. The `request_id` is the index into this list.

This is the reference implementation of "rebuild state from the journal" described in [journals.md](concepts/journals.md#for-controller-authors).

### UI components

```
UIRoot                         root.rs: status bar + page tabs, focus
├─ StatusDisplay               widgets/status.rs
└─ TabView<AppContext>         widgets/tab_view.rs
   ├─ MachinesPage             widgets/machines_view.rs: picker + resource tabs
   │  ├─ DropDown              controls/drop_down.rs
   │  └─ TabView<MachinesContext>
   │     ├─ ConfigPage         widgets/config.rs
   │     ├─ StatePage          widgets/state.rs
   │     ├─ MeasurementsPage   widgets/measurements.rs
   │     ├─ CommandsView       widgets/command.rs
   │     ├─ EventsView         widgets/events.rs
   │     └─ SubscriptionsView  widgets/subscriptions.rs
   └─ TransactionsPage         widgets/transactions.rs
```

Reusable pieces live in `components/`: `Navigation` (list cursor), `EditMenu` (value editor), `EventLogMenu` (history list), `InspectView` (record details) and `ChartComponent`.

Conventions:

- **Pages implement `TabItem<Ctx>`**, which has `on_key(code, ctx) -> KeyResult<AppAction>` and `render(frame, area, in_focus, ctx)`.
- **Key events bubble.** A component returns `KeyResult::Handled(action)` if it used the key, or `KeyResult::Bubble(code)` to pass it to its parent. Focus moves between panels when ↑/↓ bubble out of a list. `q` only quits if it bubbles all the way to the root.
- **Pages are modal state machines.** For example, `ConfigPage` switches between `Navigate`, `Editing`, `History` and `Inspect`. Each mode has its own `on_key_*` and `render_*` functions.
- **Components don't send requests.** They return an `AppAction` (`SetConfig`, `ExecuteCommand`, `Subscribe`, `Unsubscribe`), and `Tui::run` turns it into a request.
- **Context is passed as raw pointers.** `AppContext` and `MachinesContext` are `Copy` structs of raw pointers into `AppState`, so they can be handed down the tree during `on_key` and `render`. They are only valid for that one call. Never store them.

## Known issues

- **Requests are only sent after a report arrives, one per report.** `session::run` checks for a pending request once after each report. Requests therefore wait up to one report interval, and several quick requests queue up at 32 per second. If reports stop, requests aren't sent at all. The loop should `select!` on both channels.
- **Pending requests show as "Success".** A transaction is created with `result: Ok(())` before the Runtime has answered. It should have a pending state.
- **The Transactions list can't be scrolled.** `TransactionsPage::on_key_navigate` limits ↓ with `self.entries.len()`, but `entries` is never filled, so only the newest transaction can be selected.
- **Invalid input is silently dropped.** The value editor gives no feedback when the text doesn't convert, or when a property is not writable (`// TODO: flash red`).
- **Nullable values can't be set to null.** The editor doesn't accept a null input.
- **Removed machines stay listed.** `RuntimeEvent::RemovedMachine` is ignored, so a machine removed after an `Irrecoverable` error still shows its last values.
- **Quitting brings the process down through panics.** The Runtime loop never returns by itself. The likely sequence after `q` (not yet confirmed in a test):
  1. the session task's `tx.send(..).expect(..)` panics because the UI side has gone,
  2. its transport is dropped,
  3. the Runtime's `send_report(..).unwrap()` panics, and
  4. `runtime_thread.join().unwrap()` in `run_with_tui` re-raises that panic.

  The terminal is restored by the panic hook, but quitting should be a clean shutdown signal instead.
- **Disconnects are final.** After `Disconnected`, the TUI doesn't reconnect. Restart the application.
- **Mouse capture is enabled but unused**, which prevents normal text selection in most terminals.
