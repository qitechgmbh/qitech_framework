use std::collections::BTreeMap;
use std::time::Duration;

use kdl::KdlDocument;
use kdl::KdlNode;

use crate::schema::Issues;
use crate::system::ModbusRtuBus;
use crate::system::ModbusRtuDevice;
use crate::system::Parity;
use crate::system::parser::binding::parse_binding;
use crate::system::parser::helpers::check_entries;
use crate::system::parser::helpers::integer;
use crate::system::parser::helpers::property;

const BUS_PROPERTIES: &[&str] = &["path", "baudrate", "parity", "stop_bits", "data_bits"];

pub(super) fn parse_modbus_rtu(
    node: &KdlNode,
    machines: &BTreeMap<String, String>,
    issues: &mut Issues,
) -> Option<ModbusRtuBus> {
    let before = issues.count();
    let offset = node.span().offset();

    check_entries(node, "`modbus_rtu`", BUS_PROPERTIES, issues);

    let path = match property(node, "path") {
        Some(entry) => match entry.value().as_string() {
            Some(path) if !path.is_empty() => Some(path.to_owned()),
            _ => {
                issues.push(
                    entry.span().offset(),
                    "`path` of `modbus_rtu` must be a string like \"/dev/ttyUSB0\"",
                );
                None
            }
        },
        None => {
            issues.push(offset, "`modbus_rtu` is missing `path`");
            None
        }
    };

    let baudrate = match property(node, "baudrate") {
        Some(entry) => match integer::<u32>(entry).filter(|b| *b > 0) {
            Some(baudrate) => Some(baudrate),
            None => {
                issues.push(
                    entry.span().offset(),
                    "`baudrate` of `modbus_rtu` must be a positive integer",
                );
                None
            }
        },
        None => {
            issues.push(offset, "`modbus_rtu` is missing `baudrate`");
            None
        }
    };

    let parity = property(node, "parity").and_then(|entry| match entry.value().as_string() {
        Some("none") => Some(Parity::None),
        Some("even") => Some(Parity::Even),
        Some("odd") => Some(Parity::Odd),
        _ => {
            issues.push(
                entry.span().offset(),
                "`parity` of `modbus_rtu` must be `none`, `even` or `odd`",
            );
            None
        }
    });

    let stop_bits = property(node, "stop_bits").and_then(|entry| {
        let value = integer::<u8>(entry).filter(|v| matches!(v, 1 | 2));
        if value.is_none() {
            issues.push(
                entry.span().offset(),
                "`stop_bits` of `modbus_rtu` must be 1 or 2",
            );
        }
        value
    });

    let data_bits = property(node, "data_bits").and_then(|entry| {
        let value = integer::<u8>(entry).filter(|v| (5..=8).contains(v));
        if value.is_none() {
            issues.push(
                entry.span().offset(),
                "`data_bits` of `modbus_rtu` must be between 5 and 8",
            );
        }
        value
    });

    let devices = parse_devices(node.children(), machines, issues);

    if issues.count() > before {
        return None;
    }

    Some(ModbusRtuBus {
        path: path?,
        baudrate: baudrate?,
        parity,
        stop_bits,
        data_bits,
        devices,
    })
}

/// `device address=1 binding=just_some_laser.device timeout="1000ms"`; `slave_id=` is an alias for `address=`.
fn parse_devices(
    children: Option<&KdlDocument>,
    machines: &BTreeMap<String, String>,
    issues: &mut Issues,
) -> BTreeMap<u8, ModbusRtuDevice> {
    let mut devices = BTreeMap::new();

    let Some(children) = children else {
        return devices;
    };

    for node in children.nodes() {
        let offset = node.span().offset();

        if node.name().value() != "device" {
            issues.push(
                offset,
                format!(
                    "unknown node `{}` in `modbus_rtu` (expected `device`)",
                    node.name().value()
                ),
            );
            continue;
        }

        let before = issues.count();

        if node.children().is_some() {
            issues.push(offset, "`device` cannot have children");
        }
        check_entries(
            node,
            "`device`",
            &["address", "slave_id", "binding", "timeout"],
            issues,
        );

        let address = match (property(node, "address"), property(node, "slave_id")) {
            (Some(_), Some(entry)) => {
                issues.push(
                    entry.span().offset(),
                    "`device` takes either `address` or `slave_id`, not both",
                );
                None
            }
            (Some(entry), None) | (None, Some(entry)) => {
                let address = integer::<u8>(entry).filter(|a| (1..=247).contains(a));
                if address.is_none() {
                    issues.push(
                        entry.span().offset(),
                        "address of `device` must be an integer between 1 and 247",
                    );
                }
                address
            }
            (None, None) => {
                issues.push(offset, "`device` is missing `address`");
                None
            }
        };

        let binding = match property(node, "binding") {
            Some(entry) => parse_binding(entry, machines, issues),
            None => {
                issues.push(offset, "`device` is missing `binding`");
                None
            }
        };

        let timeout = property(node, "timeout").and_then(|entry| {
            let timeout = entry.value().as_string().and_then(parse_duration);
            if timeout.is_none() {
                issues.push(
                    entry.span().offset(),
                    "`timeout` of `device` must be a duration like \"1000ms\" or \"1s\"",
                );
            }
            timeout
        });

        if issues.count() > before {
            continue;
        }
        let (Some(address), Some(binding)) = (address, binding) else {
            continue;
        };

        if devices.contains_key(&address) {
            issues.push(offset, format!("duplicate device address {address}"));
            continue;
        }
        devices.insert(address, ModbusRtuDevice { binding, timeout });
    }

    devices
}

/// `"1000ms"` or `"1s"`.
fn parse_duration(text: &str) -> Option<Duration> {
    if let Some(ms) = text.strip_suffix("ms") {
        ms.parse().ok().map(Duration::from_millis)
    } else if let Some(s) = text.strip_suffix('s') {
        s.parse().ok().map(Duration::from_secs)
    } else {
        None
    }
}
