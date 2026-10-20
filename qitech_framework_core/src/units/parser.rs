use kdl::KdlDocument;
use kdl::KdlNode;
use kdl::KdlValue;

use crate::units::Conversion;
use crate::units::Quantity;
use crate::units::Unit;
use crate::units::UnitCatalog;
use crate::units::error::Issues;

// --- parser ---
pub(super) fn parse_catalog(document: &KdlDocument, issues: &mut Issues) -> UnitCatalog {
    let mut quantities: Vec<Quantity> = Vec::new();

    for node in document.nodes() {
        let Some(quantity) = parse_quantity(node, issues) else {
            continue;
        };

        if quantities.iter().any(|q| q.name == quantity.name) {
            issues.push(
                node.span().offset(),
                format!("duplicate quantity `{}`", quantity.name),
            );
            continue;
        }

        quantities.push(quantity);
    }

    UnitCatalog { quantities }
}

fn parse_quantity(node: &KdlNode, issues: &mut Issues) -> Option<Quantity> {
    let name = node.name().value().to_owned();

    if let Some(entry) = node.entries().first() {
        issues.push(
            entry.span().offset(),
            format!("quantity `{name}` takes no arguments or properties"),
        );
    }

    let Some(children) = node.children().filter(|c| !c.nodes().is_empty()) else {
        issues.push(
            node.span().offset(),
            format!("quantity `{name}` has no units"),
        );
        return None;
    };

    // keeps the node offset next to each unit for later issues
    let mut units: Vec<(Unit, usize)> = Vec::new();
    let mut base_count = 0;

    for unit_node in children.nodes() {
        let Some(unit) = parse_unit(&name, unit_node, issues) else {
            continue;
        };

        if units.iter().any(|(u, _)| u.name == unit.name) {
            issues.push(
                unit_node.span().offset(),
                format!("duplicate unit `{name}.{}`", unit.name),
            );
            continue;
        }

        if unit.base {
            base_count += 1;
            if base_count > 1 {
                issues.push(
                    unit_node.span().offset(),
                    format!("quantity `{name}` has more than one base unit"),
                );
            }
        }

        units.push((unit, unit_node.span().offset()));
    }

    // without a base unit, conversions have nothing to convert into
    if base_count == 0 {
        for (unit, offset) in &mut units {
            if unit.conversion != Conversion::IDENTITY {
                issues.push(
                    *offset,
                    format!(
                        "unit `{name}.{}` defines a conversion, but quantity `{name}` has no base unit",
                        unit.name
                    ),
                );
            }
            unit.conversion = Conversion::None;
        }
    }

    Some(Quantity {
        name,
        units: units.into_iter().map(|(unit, _)| unit).collect(),
    })
}

const UNIT_PROPERTIES: &[&str] = &[
    "symbol",
    "base",
    "factor",
    "offset",
    "log10_multiplier",
    "log10_reference",
];

fn parse_unit(quantity: &str, node: &KdlNode, issues: &mut Issues) -> Option<Unit> {
    let name = node.name().value().to_owned();
    let id = format!("{quantity}.{name}");
    let before = issues.issues.len();

    for entry in node.entries() {
        match entry.name() {
            None => issues.push(
                entry.span().offset(),
                format!("unit `{id}` takes no arguments, only properties"),
            ),
            Some(key) if !UNIT_PROPERTIES.contains(&key.value()) => issues.push(
                entry.span().offset(),
                format!(
                    "unknown property `{}` on unit `{id}` (expected one of: {})",
                    key.value(),
                    UNIT_PROPERTIES.join(", ")
                ),
            ),
            Some(_) => {}
        }
    }

    if node.children().is_some() {
        issues.push(
            node.span().offset(),
            format!("unit `{id}` cannot have children"),
        );
    }

    let symbol = match node.get("symbol") {
        Some(KdlValue::String(s)) => s.clone(),
        Some(_) => {
            issues.push(
                node.span().offset(),
                format!("`symbol` of `{id}` must be a string"),
            );
            String::new()
        }
        None => {
            issues.push(
                node.span().offset(),
                format!("unit `{id}` is missing `symbol`"),
            );
            String::new()
        }
    };

    let base = match node.get("base") {
        Some(KdlValue::Bool(b)) => *b,
        Some(_) => {
            issues.push(
                node.span().offset(),
                format!("`base` of `{id}` must be #true or #false"),
            );
            false
        }
        None => false,
    };

    let factor = number(node, "factor", &id, issues);
    let offset = number(node, "offset", &id, issues);
    let multiplier = number(node, "log10_multiplier", &id, issues);
    let reference = number(node, "log10_reference", &id, issues);

    let has_affine = factor.is_some() || offset.is_some();
    let has_log = multiplier.is_some() || reference.is_some();

    let conversion = if base && (has_affine || has_log) {
        issues.push(
            node.span().offset(),
            format!("base unit `{id}` cannot define a conversion"),
        );
        Conversion::IDENTITY
    } else if has_affine && has_log {
        issues.push(
            node.span().offset(),
            format!("unit `{id}` mixes `factor`/`offset` with `log10_*`"),
        );
        Conversion::IDENTITY
    } else if has_log {
        match (multiplier, reference) {
            (Some(multiplier), Some(reference)) => {
                if multiplier == 0.0 {
                    issues.push(
                        node.span().offset(),
                        format!("`log10_multiplier` of `{id}` cannot be 0"),
                    );
                }
                if reference <= 0.0 {
                    issues.push(
                        node.span().offset(),
                        format!("`log10_reference` of `{id}` must be positive"),
                    );
                }
                Conversion::Log10 {
                    multiplier,
                    reference,
                }
            }
            _ => {
                issues.push(
                    node.span().offset(),
                    format!("unit `{id}` needs both `log10_multiplier` and `log10_reference`"),
                );
                Conversion::IDENTITY
            }
        }
    } else {
        let factor = factor.unwrap_or(1.0);
        if factor == 0.0 {
            issues.push(
                node.span().offset(),
                format!("`factor` of `{id}` cannot be 0"),
            );
        }
        Conversion::Affine {
            factor,
            offset: offset.unwrap_or(0.0),
        }
    };

    if issues.issues.len() > before {
        return None;
    }

    Some(Unit {
        name,
        symbol,
        base,
        conversion,
    })
}

/// Reads an optional finite number; integers are accepted as well.
fn number(node: &KdlNode, key: &str, id: &str, issues: &mut Issues) -> Option<f64> {
    let value = match node.get(key)? {
        KdlValue::Float(v) => *v,
        KdlValue::Integer(v) => *v as f64,
        _ => {
            issues.push(
                node.span().offset(),
                format!("`{key}` of `{id}` must be a number"),
            );
            return None;
        }
    };

    if !value.is_finite() {
        issues.push(
            node.span().offset(),
            format!("`{key}` of `{id}` must be finite"),
        );
        return None;
    }

    Some(value)
}
