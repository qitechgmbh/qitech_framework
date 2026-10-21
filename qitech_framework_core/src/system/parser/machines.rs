use std::collections::BTreeMap;

use kdl::KdlNode;

use crate::schema::Issues;
use crate::system::parser::helpers::node_name;

/// `machines { just_some_winder winder_v1 }`: instance name -> imported machine type.
pub(super) fn parse_machines(
    node: &KdlNode,
    imports: &[String],
    issues: &mut Issues,
) -> BTreeMap<String, String> {
    let mut machines = BTreeMap::new();

    if !node.entries().is_empty() {
        issues.push(
            node.span().offset(),
            "section `machines` takes no arguments or properties",
        );
    }

    let Some(children) = node.children() else {
        return machines;
    };

    for machine in children.nodes() {
        let offset = machine.span().offset();
        let Some(name) = node_name(machine, issues) else {
            continue;
        };

        if machine.children().is_some() {
            issues.push(offset, format!("machine `{name}` cannot have children"));
        }

        let ty = match machine.entries() {
            [entry] if entry.name().is_none() => match entry.value().as_string() {
                Some(ty) => ty.to_owned(),
                None => {
                    issues.push(
                        entry.span().offset(),
                        format!("type of machine `{name}` must be a name like `winder_v1`"),
                    );
                    continue;
                }
            },
            _ => {
                issues.push(
                    offset,
                    format!("machine `{name}` takes exactly one type (e.g. `{name} winder_v1`)"),
                );
                continue;
            }
        };

        if !imports.contains(&ty) {
            issues.push(
                offset,
                format!("machine `{name}` uses `{ty}`, but it is not imported (add `import {ty}`)"),
            );
            continue;
        }

        if machines.contains_key(&name) {
            issues.push(offset, format!("duplicate machine `{name}`"));
            continue;
        }
        machines.insert(name, ty);
    }

    machines
}
