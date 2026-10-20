# Framework Architecture Goals/Design

## Information and Responsibility distribution:

### Static 

In the runtime side application merely machines, their capabilities, data and hardware slots are registered. 
-> Define capability and allow dynamic bindings to use them.

#### Defining Slots in Rust



### Dynamic

KDL System graph defines the relationships and existing hardware, as well as units.
Units can then be matched when we register a machine.


## Split of the Runtime and Controller

The framework is split into two parts with clearly separated responsibilities: the **runtime** executes, the **controller** decides.

### Runtime: execution and hardware

The runtime owns the hardware and the machines. It runs a deterministic, cyclic tick loop that drives the buses (EtherCAT, Modbus RTU, …), binds devices to machine slots as described by the KDL system graph, and calls every machine on each tick.

Machines do not decide what the process should do. They merely expose their **capabilities** through a typed interface defined by their schema:

- **config**: behaviour the machine allows to be set (setpoints, limits, modes)
- **inputs**: context data the machine works with, fed from other machines or the controller
- **commands**: actions and transitions the machine can perform
- **state**, **measurements** and **events**: what the machine is, what it measures and what happened

Within that interface the runtime enforces everything that has to be correct within a tick or must keep working when the controller is gone: control loops, motion profiles, interlocks, limits, constraint validation, the machine lifecycle (unavailable → unconfigured → ready) and a safe state on controller loss. Local input bindings between machines on the same runtime are also executed here, because they are part of the deterministic data flow.

The runtime is **policy-free**: it makes no process decisions on its own. It executes what it is told within the limits the machines define, and reports what happens.

### Controller: orchestration

The controller is the orchestrator. It connects to the runtime through the session protocol, receives the machine schemas and unit catalog during the handshake, and from then on instructs the machines: it writes config, feeds inputs, assigns input bindings and sends commands, for example by streaming queued steps for sequential execution.

Everything that is process knowledge, depends on external systems or tolerates latency lives here:

- sequencing, recipes, job queues and coordination of several machines
- data processing such as image analysis, whose results are reduced to a few values before they reach a machine
- connections to databases and external services (recording measurements and events, persisting and restoring config)
- exposing APIs to user interfaces or other systems

The controller is deliberately slim. The framework provides an SDK with the session, typed machine clients, commands as futures and a few presets (cyclic and sequential execution), while the actual business logic belongs to the application built on top of it.

### Boundary

Runtime and controller normally run as separate processes connected through a transport, so a failure in the controller cannot take down the tick loop, and both can be deployed and updated independently. Exactly one controller session controls a runtime. Only small, scalar data crosses the boundary: instructions go in, states and measurements come out. Anything that needs rich data at that boundary is usually a sign that processing is happening on the wrong side.

