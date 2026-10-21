mod binding;
mod helpers;
mod machines;
mod modbus_rtu;

use std::collections::BTreeMap;

use kdl::KdlDocument;
use kdl::KdlNode;

use crate::schema::Issues;
use crate::system::ModbusRtuBus;
use crate::system::SystemSchema;
use crate::system::parser::machines::parse_machines;
use crate::system::parser::modbus_rtu::parse_modbus_rtu;

const SECTIONS: &[&str] = &["machines", "modbus_rtu"];

// --- system ---
pub(super) fn parse_system(document: &KdlDocument, issues: &mut Issues) -> Option<SystemSchema> {
    let mut imports: Vec<String> = Vec::new();
    let mut machines_node: Option<&KdlNode> = None;
    let mut buses: Vec<&KdlNode> = Vec::new();

    // imports first; sections are collected and parsed afterwards
    for node in document.nodes() {
        let offset = node.span().offset();

        match node.name().value() {
            "import" => {
                if machines_node.is_some() || !buses.is_empty() {
                    issues.push(offset, "`import` must come before all sections");
                }
                let Some(import) = parse_import(node, issues) else {
                    continue;
                };
                if imports.contains(&import) {
                    issues.push(offset, format!("duplicate import `{import}`"));
                    continue;
                }
                imports.push(import);
            }
            "machines" => {
                if machines_node.is_some() {
                    issues.push(offset, "duplicate section `machines`");
                    continue;
                }
                machines_node = Some(node);
            }
            // several serial ports may each have their own bus
            "modbus_rtu" => buses.push(node),
            other => issues.push(
                offset,
                format!(
                    "unknown top-level node `{other}` (expected `import` or one of: {})",
                    SECTIONS.join(", ")
                ),
            ),
        }
    }

    let machines = match machines_node {
        Some(node) => parse_machines(node, &imports, issues),
        None => BTreeMap::new(),
    };

    let mut modbus_rtu: Vec<ModbusRtuBus> = Vec::new();
    for node in buses {
        let Some(bus) = parse_modbus_rtu(node, &machines, issues) else {
            continue;
        };
        if modbus_rtu.iter().any(|b| b.path == bus.path) {
            issues.push(
                node.span().offset(),
                format!("duplicate `modbus_rtu` bus on `{}`", bus.path),
            );
            continue;
        }
        modbus_rtu.push(bus);
    }

    Some(SystemSchema {
        imports,
        machines,
        modbus_rtu,
    })
}

/// `import winder_v1`: exactly one string argument, nothing else.
fn parse_import(node: &KdlNode, issues: &mut Issues) -> Option<String> {
    let offset = node.span().offset();

    if node.children().is_some() {
        issues.push(offset, "`import` cannot have children");
    }

    match node.entries() {
        [entry] if entry.name().is_none() => match entry.value().as_string() {
            Some(value) if !value.is_empty() => Some(value.to_owned()),
            _ => {
                issues.push(
                    entry.span().offset(),
                    "`import` must be a name like `winder_v1`",
                );
                None
            }
        },
        _ => {
            issues.push(offset, "`import` takes exactly one name");
            None
        }
    }
}
