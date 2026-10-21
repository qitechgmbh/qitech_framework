use std::collections::BTreeMap;

use kdl::KdlEntry;

use crate::schema::Issues;
use crate::system::Binding;
use crate::system::parser::helpers::is_identifier;

pub(super) fn parse_binding(
    entry: &KdlEntry,
    machines: &BTreeMap<String, String>,
    issues: &mut Issues,
) -> Option<Binding> {
    let offset = entry.span().offset();

    let Some((machine, path)) = entry
        .value()
        .as_string()
        .and_then(|s| s.split_once('.'))
        .filter(|(m, p)| is_identifier(m) && p.split('.').all(is_identifier))
    else {
        issues.push(
            offset,
            "`binding` must look like `<machine>.<path>`, e.g. `just_some_laser.device`",
        );
        return None;
    };

    if !machines.contains_key(machine) {
        issues.push(
            offset,
            format!("`binding` refers to unknown machine `{machine}` (declare it in `machines`)"),
        );
        return None;
    }

    Some(Binding {
        machine: machine.to_owned(),
        path: path.to_owned(),
    })
}
