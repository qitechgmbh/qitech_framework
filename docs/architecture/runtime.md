# Runtime Architecture

The Runtime is the machine-control process. Its primary responsibility is controlling the configured machines and maintaining their internal state. Communication with external systems is secondary and must never interfere with machine execution.

A Runtime has exactly one controller session. The controller is responsible for consuming, buffering, persisting, and distributing Runtime data.

## Startup

The Runtime is configured with the machines that are available. Machines are defined by the user through the Runtime's machine traits and registered in the Runtime configuration.

Before entering the running state, the Runtime establishes its controller session through a multi-stage handshake:

### Initializing the Session


## Controller communication

The Runtime does not act as a data broker or persistence layer.

The controller is responsible for:

- receiving reports
- buffering data if necessary
- persisting data
- distributing data to other consumers
- sending requests to the Runtime

The Runtime must not block machine execution because the controller is slow.

If the controller cannot accept a report because of backpressure, the Runtime terminates the controller session rather than buffering indefinitely or dropping the report.

## Session loss

If that session is disconnected or the controller can no longer receive reports, the Runtime becomes orphaned.

The Runtime does not attempt to reconnect while continuing to operate the machines. Re-establishing a controller session requires synchronization in both directions, and continuing machine execution during that process could result in state being missed or becoming inconsistent.

The Runtime's shutdown is intentional and deterministic. It is preferable to continuing operation without a controller and potentially producing an incomplete or unverifiable state history.

## Core invariants

1. **Exactly one controller session** exists at a time.
2. **Machine execution is never blocked by data delivery.**
3. **Reports are ordered and are not intentionally discarded.**
4. **A report stream remains internally consistent; missing reports are not tolerated.**
5. **The controller is responsible for data consumption and persistence.**
6. **Loss of the controller session terminates the Runtime.**
7. **A new session must perform synchronization before the Runtime can operate again.**
