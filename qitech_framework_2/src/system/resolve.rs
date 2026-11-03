use std::collections::BTreeMap;
use std::collections::BTreeSet;

use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::schema::ValueDefinition;
use qitech_framework_core::schema::ValueType;
use qitech_framework_core::system::SystemSchema;
use qitech_framework_core::units::UnitCatalog;

use crate::system::MachineType;

/// Checks a parsed system against the registry; the parser already checked it is self-consistent.
pub(super) fn resolve(
    schema: &SystemSchema,
    units: &BTreeMap<String, UnitCatalog>,
    machine_types: &BTreeMap<String, MachineType>,
    issues: &mut Vec<String>,
) {
    for import in &schema.imports {
        if !units.contains_key(import) && !machine_types.contains_key(import) {
            let registered: Vec<&str> = units
                .keys()
                .chain(machine_types.keys())
                .map(String::as_str)
                .collect();
            issues.push(format!(
                "`import {import}` is not registered (registered: {})",
                registered.join(", ")
            ));
        }
    }

    for (instance, ty) in &schema.machines {
        if units.contains_key(ty) {
            issues.push(format!(
                "machine `{instance}` uses `{ty}`, which is a unit catalog, not a machine type"
            ));
        }
    }

    // only check machine types the system actually uses
    let used: BTreeSet<&String> = schema
        .machines
        .values()
        .filter(|ty| machine_types.contains_key(*ty))
        .collect();
    for ty in used {
        check_units(&machine_types[ty], units, issues);
    }

    // TODO: check binding paths once machine schemas declare their hardware slots
}

/// Every unit import and `quantity=` of a machine schema must exist in a registered catalog.
fn check_units(
    machine_type: &MachineType,
    units: &BTreeMap<String, UnitCatalog>,
    issues: &mut Vec<String>,
) {
    let schema = &machine_type.schema;
    let name = &schema.name;

    for import in &schema.imports {
        if !units.contains_key(import) {
            issues.push(format!(
                "machine type `{name}` imports `{import}`, but no unit catalog `{import}` is registered"
            ));
        }
    }

    for value in value_definitions(schema) {
        let (ValueType::Integer { quantity } | ValueType::Float { quantity }) = &value.ty else {
            continue;
        };
        let Some(quantity) = quantity else {
            continue;
        };
        // a missing catalog is reported above
        let Some(catalog) = units.get(&quantity.import) else {
            continue;
        };
        if catalog.quantity(&quantity.quantity).is_none() {
            issues.push(format!(
                "`{}` of machine type `{name}` uses unknown quantity `{}.{}`",
                value.path, quantity.import, quantity.quantity
            ));
        }
    }
}

fn value_definitions(schema: &MachineSchema) -> impl Iterator<Item = &ValueDefinition> {
    let sections = [
        &schema.config,
        &schema.state,
        &schema.measurements,
        &schema.inputs,
    ];
    let commands = schema.commands.iter().flat_map(|c| {
        c.params
            .iter()
            .chain(&c.returns)
            .chain(c.errors.iter().flat_map(|e| &e.fields))
    });
    let events = schema.events.iter().flat_map(|e| &e.fields);

    sections.into_iter().flatten().chain(commands).chain(events)
}
