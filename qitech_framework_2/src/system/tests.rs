use std::any::TypeId;

use qitech_framework_core::units::UnitCatalog;

use crate::descriptor::MachineDescriptor;
use crate::system::System;
use crate::system::SystemBuilder;
use crate::system::SystemLoadError;

const SI: &str = r#"
length {
    meter symbol="m" base=#true
}

velocity {
    meter_per_second symbol="m/s" base=#true
}
"#;

struct WinderV1;

impl MachineDescriptor for WinderV1 {
    const SCHEMA: &'static str = r#"
        import si
        name winder_v1
        config {
            puller {
                speed float quantity=si.velocity
            }
        }
    "#;
}

struct LaserV1;

impl MachineDescriptor for LaserV1 {
    const SCHEMA: &'static str = r#"
        import si
        name laser_v1
        measurements {
            diameter float quantity=si.length
        }
    "#;
}

struct BrokenV1;

impl MachineDescriptor for BrokenV1 {
    const SCHEMA: &'static str = r#"
        import si
        name broken_v1
        config {
            speed float quantity=si.acceleration
        }
    "#;
}

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

fn builder() -> SystemBuilder {
    System::builder()
        .units("si", UnitCatalog::parse_str(SI).unwrap())
        .machine::<WinderV1>()
        .machine::<LaserV1>()
}

fn resolve_issues(result: Result<System, SystemLoadError>) -> Vec<String> {
    match result {
        Err(SystemLoadError::Resolve(issues)) => issues,
        other => panic!("expected a resolve error, got {other:?}"),
    }
}

fn assert_issue(issues: &[String], needle: &str) {
    assert!(
        issues.iter().any(|i| i.contains(needle)),
        "expected an issue containing `{needle}`, got: {issues:#?}"
    );
}

#[test]
fn loads_checklist_system() {
    let system = builder().load_str(SYSTEM).unwrap();

    let machines: Vec<(&str, TypeId)> = system
        .machines()
        .map(|(instance, ty)| (instance, ty.type_id))
        .collect();
    assert_eq!(
        machines,
        [
            ("just_some_laser", TypeId::of::<LaserV1>()),
            ("just_some_winder", TypeId::of::<WinderV1>()),
        ]
    );

    assert_eq!(
        system.schema().modbus_rtu[0].devices[&1].binding.machine,
        "just_some_laser"
    );
    assert!(system.units("si").is_some());
}

#[test]
fn rejects_unregistered_import() {
    let issues = resolve_issues(System::builder().machine::<WinderV1>().load_str(SYSTEM));
    assert_issue(&issues, "`import laser_v1` is not registered");
}

#[test]
fn rejects_machine_type_without_its_unit_catalog() {
    let issues = resolve_issues(
        System::builder()
            .machine::<WinderV1>()
            .machine::<LaserV1>()
            .load_str(SYSTEM),
    );
    assert_issue(
        &issues,
        "imports `si`, but no unit catalog `si` is registered",
    );
}

#[test]
fn rejects_unknown_quantity_in_used_machine() {
    let system = r#"
        import broken_v1
        machines { broken broken_v1 }
    "#;
    let issues = resolve_issues(builder().machine::<BrokenV1>().load_str(system));
    assert_issue(&issues, "unknown quantity `si.acceleration`");
}

#[test]
fn ignores_registered_but_unused_machine_types() {
    builder().machine::<BrokenV1>().load_str(SYSTEM).unwrap();
}

#[test]
fn rejects_unit_catalog_as_machine_type() {
    let system = r#"
        import si
        machines { oops si }
    "#;
    let issues = resolve_issues(builder().load_str(system));
    assert_issue(&issues, "which is a unit catalog, not a machine type");
}

#[test]
fn rejects_duplicate_registration() {
    let issues = resolve_issues(builder().machine::<WinderV1>().load_str(SYSTEM));
    assert_issue(&issues, "`winder_v1` (from `");
}

#[test]
fn reports_parse_errors() {
    let result = builder().load_str("machines { a }");
    assert!(matches!(result, Err(SystemLoadError::Parse(_))));
}
