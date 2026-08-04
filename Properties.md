# Configuration vs Commands

This system distinguishes between **configuration** and **commands**. The difference is based on the purpose of the operation, not on how many arguments it has or whether it causes internal side effects.

## Configuration

Configuration represents the desired state of the system.

A configuration value answers:

> "How should the system be configured?"

Examples:

```
puller.regulation_mode = Diameter
motor.max_speed = 2000
pid.kp = 1.2
network.ip_address = "192.168.1.10"
```

Configuration values:

* represent persistent or semi-persistent state
* can be read back
* can be displayed and edited by an HMI
* should have validation rules
* should remain meaningful after the write operation completes

A configuration change may have internal consequences. For example:

```
puller.regulation_mode = Diameter
```

may reset adaptive modulation or update internal controller state. This is still configuration because the action exists to keep the runtime state consistent with the new configuration.

Configuration changes should use callbacks/hooks when needed:

```
on_change(config_value):
    validate
    update dependent state
    maintain invariants
```

## Commands

Commands represent an explicit request to perform an operation.

A command answers:

> "What should the system do?"

Examples:

```
reboot()
calibrate()
home_axis()
clear_faults()
start_production()
```

Commands:

* trigger an operation
* are not themselves persistent state
* may have parameters
* usually have a completion/result
* may not make sense to read back afterward

A command should not be used as a replacement for configuration.

Bad:

```
set_regulation_mode(Diameter)
```

when the intent is to configure the machine.

Good:

```
puller.regulation_mode = Diameter
```

because regulation mode is a property of the machine.

## Do not use commands as hidden configuration

It is possible to create a command that changes state:

```
set_mode(Diameter)
```

or:

```
enable_feature()
```

However, if the result should be visible, persistent, editable, and discoverable as part of the machine state, it belongs in configuration.

Otherwise the HMI has no reliable way to answer:

* What is the current value?
* Was this setting saved?
* Can this be edited?
* What values are allowed?
* What happens after restart?

## Decision guide

Ask:

### "Would I want to display this value as a setting?"

If yes:

```
Configuration
```

Examples:

```
temperature.target = 80
speed.limit = 1000
regulation_mode = Diameter
```

### "Would I press a button to make this happen?"

If yes:

```
Command
```

Examples:

```
Start
Stop
Reset
Calibrate
Home
Apply
```

## Special case: Applying configuration

Some systems require configuration to be activated:

```
config.pid.kp = 1.2
config.pid.ki = 0.5
config.pid.kd = 0.01

command.apply_pid()
```

The PID values are configuration. Applying them is a command because it represents an operation that transitions the runtime system.

## Summary

Configuration describes the machine.

Commands control the machine.

A configuration write may trigger internal updates, but those updates exist to maintain consistency with the configured state. A command exists because an action itself is meaningful.
