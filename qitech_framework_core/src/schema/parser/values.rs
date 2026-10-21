use kdl::KdlDocument;
use kdl::KdlNode;
use kdl::KdlValue;

use crate::schema::EnumVariant;
use crate::schema::QuantityRef;
use crate::schema::ValueDefinition;
use crate::schema::ValueType;
use crate::schema::error::Issues;
use crate::schema::parser::Context;
use crate::schema::parser::helpers::Paths;
use crate::schema::parser::helpers::arguments;
use crate::schema::parser::helpers::bool_property;
use crate::schema::parser::helpers::has_arguments;
use crate::schema::parser::helpers::has_properties;
use crate::schema::parser::helpers::is_identifier;
use crate::schema::parser::helpers::join;
use crate::schema::parser::helpers::node_name;
use crate::schema::parser::helpers::non_empty;
use crate::schema::parser::helpers::property;

const VALUE_TYPES: &[&str] = &["boolean", "integer", "float", "string", "enum"];

// --- values ---
/// Parses leaves (`name <type> [props]`) and, if `allow_groups`, nested groups.
pub(super) fn parse_values(
    children: Option<&KdlDocument>,
    prefix: &str,
    ctx: &Context,
    issues: &mut Issues,
    paths: &mut Paths,
    out: &mut Vec<ValueDefinition>,
    allow_groups: bool,
) {
    let Some(children) = children else {
        return;
    };

    for node in children.nodes() {
        let offset = node.span().offset();
        let Some(name) = node_name(node, issues) else {
            continue;
        };
        let path = join(prefix, &name);
        if !paths.claim(&path, offset, issues) {
            continue;
        }

        if has_arguments(node) {
            if let Some(value) = parse_leaf(node, path, ctx, issues) {
                out.push(value);
            }
            continue;
        }

        if !allow_groups {
            issues.push(
                offset,
                format!("`{path}` has no type (expected e.g. `{name} float`)"),
            );
            continue;
        }

        if has_properties(node) {
            issues.push(
                offset,
                format!("`{path}` has properties but no type (expected e.g. `{name} float`)"),
            );
            continue;
        }

        match non_empty(node.children()) {
            Some(group) => parse_values(Some(group), &path, ctx, issues, paths, out, true),
            None => issues.push(offset, format!("`{path}` has neither a type nor children")),
        }
    }
}

fn parse_leaf(
    node: &KdlNode,
    path: String,
    ctx: &Context,
    issues: &mut Issues,
) -> Option<ValueDefinition> {
    let before = issues.count();
    let args = arguments(node);

    let type_name = match args[0].value() {
        KdlValue::String(s) if VALUE_TYPES.contains(&s.as_str()) => s.as_str(),
        _ => {
            issues.push(
                args[0].span().offset(),
                format!(
                    "unknown type for `{path}` (expected one of: {})",
                    VALUE_TYPES.join(", ")
                ),
            );
            return None;
        }
    };

    let allowed: &[&str] = match type_name {
        "integer" | "float" => &["quantity", "optional"],
        "string" => &["max_len", "optional"],
        _ => &["optional"],
    };

    for entry in node.entries() {
        if let Some(key) = entry.name()
            && !allowed.contains(&key.value())
        {
            issues.push(
                entry.span().offset(),
                format!(
                    "unknown property `{}` on {type_name} `{path}` (allowed: {})",
                    key.value(),
                    allowed.join(", ")
                ),
            );
        }
    }

    if let Some(extra) = args.get(1) {
        let message = if type_name == "enum" {
            format!("list the variants of enum `{path}` as children, e.g. `{path} enum {{ a; b }}`")
        } else {
            format!("unexpected argument after the type of `{path}`")
        };
        issues.push(extra.span().offset(), message);
    }

    if type_name != "enum" && node.children().is_some() {
        issues.push(
            node.span().offset(),
            format!("{type_name} `{path}` cannot have children"),
        );
    }

    let optional = bool_property(node, "optional", &path, issues).unwrap_or(false);

    let ty = match type_name {
        "boolean" => ValueType::Boolean,
        "integer" => ValueType::Integer {
            quantity: quantity_property(node, &path, ctx, issues),
        },
        "float" => ValueType::Float {
            quantity: quantity_property(node, &path, ctx, issues),
        },
        "string" => ValueType::String {
            max_len: max_len_property(node, &path, issues),
        },
        "enum" => ValueType::Enum {
            variants: parse_enum(node, &path, issues),
        },
        _ => unreachable!("type names are checked above"),
    };

    if issues.count() > before {
        return None;
    }

    Some(ValueDefinition { path, ty, optional })
}

fn quantity_property(
    node: &KdlNode,
    path: &str,
    ctx: &Context,
    issues: &mut Issues,
) -> Option<QuantityRef> {
    let entry = property(node, "quantity")?;
    let offset = entry.span().offset();

    let Some(text) = entry.value().as_string() else {
        issues.push(offset, format!("`quantity` of `{path}` must be a string"));
        return None;
    };

    let Some((import, quantity)) = text
        .split_once('.')
        .filter(|(i, q)| is_identifier(i) && is_identifier(q))
    else {
        issues.push(
            offset,
            format!("`quantity` of `{path}` must look like `<import>.<quantity>`, got `{text}`"),
        );
        return None;
    };

    if !ctx.imports.iter().any(|i| i == import) {
        issues.push(
            offset,
            format!(
                "`{path}` uses `{text}`, but `{import}` is not imported (add `import {import}`)"
            ),
        );
        return None;
    }

    Some(QuantityRef {
        import: import.to_owned(),
        quantity: quantity.to_owned(),
    })
}

fn max_len_property(node: &KdlNode, path: &str, issues: &mut Issues) -> Option<usize> {
    let entry = property(node, "max_len")?;

    match entry.value() {
        KdlValue::Integer(v) if *v > 0 => usize::try_from(*v).ok(),
        _ => {
            issues.push(
                entry.span().offset(),
                format!("`max_len` of `{path}` must be a positive integer"),
            );
            None
        }
    }
}

fn parse_enum(node: &KdlNode, path: &str, issues: &mut Issues) -> Vec<EnumVariant> {
    let offset = node.span().offset();
    let mut variants: Vec<(EnumVariant, usize)> = Vec::new();

    // `mode enum { standby; hold value=3 }`
    let Some(block) = non_empty(node.children()) else {
        issues.push(offset, format!("enum `{path}` has no variants"));
        return Vec::new();
    };

    let mut explicit = 0;

    for (index, variant) in block.nodes().iter().enumerate() {
        let offset = variant.span().offset();
        let Some(name) = node_name(variant, issues) else {
            continue;
        };

        if variant.children().is_some() {
            issues.push(
                offset,
                format!("variant `{name}` of enum `{path}` cannot have children"),
            );
        }

        let mut value = index as i64;
        for entry in variant.entries() {
            match entry.name().map(|n| n.value()) {
                Some("value") => match entry
                    .value()
                    .as_integer()
                    .and_then(|v| i64::try_from(v).ok())
                {
                    Some(v) => {
                        value = v;
                        explicit += 1;
                    }
                    None => issues.push(
                        entry.span().offset(),
                        format!("`value` of variant `{name}` must be an integer"),
                    ),
                },
                _ => issues.push(
                    entry.span().offset(),
                    format!("variant `{name}` of enum `{path}` only accepts `value=`"),
                ),
            }
        }

        variants.push((EnumVariant { name, value }, offset));
    }

    if explicit > 0 && explicit < variants.len() {
        issues.push(
            offset,
            format!("either all variants of enum `{path}` have `value=` or none"),
        );
    }

    for (i, (variant, offset)) in variants.iter().enumerate() {
        let earlier = &variants[..i];
        if earlier.iter().any(|(v, _)| v.name == variant.name) {
            issues.push(
                *offset,
                format!("duplicate variant `{}` in enum `{path}`", variant.name),
            );
        } else if earlier.iter().any(|(v, _)| v.value == variant.value) {
            issues.push(
                *offset,
                format!("duplicate value {} in enum `{path}`", variant.value),
            );
        }
    }

    variants.into_iter().map(|(variant, _)| variant).collect()
}
