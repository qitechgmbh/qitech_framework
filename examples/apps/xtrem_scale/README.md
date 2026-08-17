# xtrem_scale

A machine driven by a GRAM **XTREM / XTREM-S** weighing module over UDP, the XTREM counterpart to
[`qitech_laser`](../qitech_laser) (Modbus RTU).

`ScaleV1` publishes `net` / `gross` / `tare` as measurements, `stable` / `overload` as state, and
exposes `zero`, `tare`, and `clear_tare` as commands.

## Running

```bash
cargo run -p xtrem_scale
```

The modules must be reachable on the same subnet. Bus defaults are the factory ones — bind
`0.0.0.0:5555` (register `0700h`, the port modules send to) and broadcast `255.255.255.255:4444`
(register `0701h`, the port they listen on). Override them by passing a populated
`XtremConfig` to `.xtrem(..)` instead of `XtremConfig::default()`.

The bind address must stay `0.0.0.0`. A module replies to the subnet broadcast address, never to
the requester's unicast IP, so a socket bound to one specific LAN address silently receives
nothing — `xtrem::transport::udp::bind_socket` refuses any other bind for that reason.

## Finding your modules' serial numbers

Each `.xtrem_device(..)` line claims one module by its **serial number** (register `0000h`), which
is factory-set and unique. The device id is not usable for this: every module ships as `01`.

Start the app and read the `XtremDiscoveryCompleted` init event — it lists every module that
answered the sweep, claimed or not, so a newly added scale shows up there with its serial. Or
probe the bus directly:

```bash
cargo run -p xtrem --example discover -- --bind 0.0.0.0:5555 --broadcast <subnet>.255:4444
```

Put the serials into `SCALE_A_SERIAL` / `SCALE_B_SERIAL` in [src/main.rs](src/main.rs).

## Adding a scale

`.machine::<ScaleV1>()` registers the machine *type* once; each `.xtrem_device` line claims one
module for one `MachineIdentificationUnique`, so N lines produce N machines. To add a third:

1. Give the new module a unique device id — `cargo run -p xtrem --example assign_ids`. Do this one
   module at a time; while several share an id, a broadcast write renumbers all of them.
2. Add `.xtrem_device::<XtremScale>(<serial>, ScaleV1::IDENTIFICATION.unique(3), ScaleMode::Poll)`.

Step 1 is not optional. The bus routes replies by the `ID_O` field, so two modules answering to
one device id would feed each other's readings into both drivers. Init detects this and reports
`XtremDeviceIdCollision`, skipping the affected modules rather than publishing wrong weights.

## Commands

| Command | Effect |
|---|---|
| `zero` | Resets the zero point of the **gross** weight. Use on an empty platform that reads non-zero. |
| `tare` | Stores the current load as the tare, so **net** reads 0. The module waits for a stable reading and refuses if it never settles. |
| `clear_tare` | Drops the stored tare, so **net** reads **gross** again. |

A refusal is not fatal: it surfaces through `XtremScale::take_error` as `XtremError::Execute` and
is logged, leaving the machine running.

## Poll vs stream

`ScaleMode::Poll` re-reads register `0107h` every request slot. It is deterministic and it is the
only mode where silence is distinguishable from a stalled module.

`ScaleMode::Stream { interval_ms }` asks the module to push readings instead — cheaper on the
wire, and the driver re-arms the stream if it goes quiet. Change the third argument to
`.xtrem_device(..)` to switch.
