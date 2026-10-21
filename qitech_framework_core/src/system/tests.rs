use std::time::Duration;

use crate::system::Binding;
use crate::system::Parity;
use crate::system::SystemSchema;

/// The system from the design checklist, without the not yet supported `ethercat` bus.
const SYSTEM: &str = r#"
import winder_v1
import laser_v1

machines {
    just_some_winder winder_v1
    just_some_laser laser_v1
}

modbus_rtu path="/dev/ttyUSB0" baudrate=9600 parity=none stop_bits=1 data_bits=8 {
    device address=1 binding=just_some_laser.device timeout="1000ms"
}
"#;

fn messages(input: &str) -> Vec<String> {
    SystemSchema::parse_str(input)
        .expect_err("expected the system to be rejected")
        .issues
        .into_iter()
        .map(|i| i.message)
        .collect()
}

fn assert_rejected(input: &str, needle: &str) {
    let messages = messages(input);
    assert!(
        messages.iter().any(|m| m.contains(needle)),
        "expected an issue containing `{needle}`, got: {messages:#?}"
    );
}

#[test]
fn parses_checklist_system() {
    let system = SystemSchema::parse_str(SYSTEM).unwrap();

    assert_eq!(system.imports, ["winder_v1", "laser_v1"]);
    assert_eq!(system.machines["just_some_winder"], "winder_v1");
    assert_eq!(system.machines["just_some_laser"], "laser_v1");

    let [bus] = system.modbus_rtu.as_slice() else {
        panic!("expected one bus");
    };
    assert_eq!(bus.path, "/dev/ttyUSB0");
    assert_eq!(bus.baudrate, 9600);
    assert_eq!(bus.parity, Some(Parity::None));
    assert_eq!(bus.stop_bits, Some(1));
    assert_eq!(bus.data_bits, Some(8));

    let device = &bus.devices[&1];
    assert_eq!(
        device.binding,
        Binding {
            machine: "just_some_laser".to_owned(),
            path: "device".to_owned(),
        }
    );
    assert_eq!(device.timeout, Some(Duration::from_millis(1000)));
}

#[test]
fn optional_bus_and_device_properties() {
    let system = SystemSchema::parse_str(
        r#"
        import laser_v1
        machines { laser laser_v1 }
        modbus_rtu path="/dev/ttyUSB1" baudrate=19200 {
            device slave_id=7 binding=laser.sensor.diameter
        }
        "#,
    )
    .unwrap();

    let bus = &system.modbus_rtu[0];
    assert_eq!(bus.parity, None);
    assert_eq!(bus.stop_bits, None);
    assert_eq!(bus.data_bits, None);

    let device = &bus.devices[&7];
    assert_eq!(device.binding.path, "sensor.diameter");
    assert_eq!(device.timeout, None);
}

#[test]
fn empty_system() {
    let system = SystemSchema::parse_str("").unwrap();
    assert!(system.imports.is_empty());
    assert!(system.machines.is_empty());
    assert!(system.modbus_rtu.is_empty());
}

#[test]
fn rejects_missing_required_bus_properties() {
    let messages = messages("modbus_rtu parity=even");
    assert!(messages.iter().any(|m| m.contains("missing `path`")));
    assert!(messages.iter().any(|m| m.contains("missing `baudrate`")));
}

#[test]
fn rejects_unimported_machine_type() {
    assert_rejected("machines { winder winder_v1 }", "is not imported");
}

#[test]
fn rejects_unknown_binding_machine() {
    assert_rejected(
        r#"
        import laser_v1
        machines { laser laser_v1 }
        modbus_rtu path="/dev/ttyUSB0" baudrate=9600 {
            device address=1 binding=winder.device
        }
        "#,
        "unknown machine `winder`",
    );
}

#[test]
fn rejects_duplicate_device_address() {
    assert_rejected(
        r#"
        import laser_v1
        machines { a laser_v1; b laser_v1 }
        modbus_rtu path="/dev/ttyUSB0" baudrate=9600 {
            device address=1 binding=a.device
            device slave_id=1 binding=b.device
        }
        "#,
        "duplicate device address 1",
    );
}

#[test]
fn rejects_invalid_values() {
    let messages = messages(
        r#"
        import laser_v1
        machines { laser laser_v1 }
        modbus_rtu path="/dev/ttyUSB0" baudrate=9600 parity=mark stop_bits=3 data_bits=9 speed=1 {
            device address=0 binding=laser timeout="soon"
        }
        "#,
    );
    for needle in [
        "`parity`",
        "`stop_bits`",
        "`data_bits`",
        "unknown property `speed`",
        "between 1 and 247",
        "`binding` must look like",
        "`timeout`",
    ] {
        assert!(
            messages.iter().any(|m| m.contains(needle)),
            "expected an issue containing `{needle}`, got: {messages:#?}"
        );
    }
}

#[test]
fn rejects_unsupported_top_level_nodes() {
    assert_rejected(
        "ethercat interface=eth0",
        "unknown top-level node `ethercat`",
    );
}

#[test]
fn rejects_import_after_sections() {
    assert_rejected(
        "machines {}\nimport laser_v1",
        "`import` must come before all sections",
    );
}
