use kdl::KdlDocument;
use kdl::KdlNode;

use crate::schema::CommandDefinition;
use crate::schema::CommandKind;
use crate::schema::ErrorDefinition;
use crate::schema::error::Issues;
use crate::schema::parser::Context;
use crate::schema::parser::helpers::Paths;
use crate::schema::parser::helpers::join;
use crate::schema::parser::helpers::node_name;
use crate::schema::parser::helpers::non_empty;
use crate::schema::parser::helpers::property;
use crate::schema::parser::values::parse_values;

// --- commands ---
pub(super) fn parse_commands(
    children: Option<&KdlDocument>,
    prefix: &str,
    ctx: &Context,
    issues: &mut Issues,
    paths: &mut Paths,
    out: &mut Vec<CommandDefinition>,
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

        if property(node, "kind").is_some() {
            if let Some(command) = parse_command(node, path, ctx, issues) {
                out.push(command);
            }
            continue;
        }

        match non_empty(node.children()) {
            Some(group) if node.entries().is_empty() => {
                parse_commands(Some(group), &path, ctx, issues, paths, out)
            }
            _ => issues.push(
                offset,
                format!("command `{path}` is missing `kind=action` or `kind=transition`"),
            ),
        }
    }
}

fn parse_command(
    node: &KdlNode,
    path: String,
    ctx: &Context,
    issues: &mut Issues,
) -> Option<CommandDefinition> {
    let before = issues.count();

    for entry in node.entries() {
        match entry.name().map(|n| n.value()) {
            None => issues.push(
                entry.span().offset(),
                format!("command `{path}` takes no arguments"),
            ),
            Some("kind") => {}
            Some(other) => issues.push(
                entry.span().offset(),
                format!("unknown property `{other}` on command `{path}` (allowed: kind)"),
            ),
        }
    }

    let kind_entry = property(node, "kind")?;
    let kind = match kind_entry.value().as_string() {
        Some("action") => Some(CommandKind::Action),
        Some("transition") => Some(CommandKind::Transition),
        _ => {
            issues.push(
                kind_entry.span().offset(),
                format!("`kind` of command `{path}` must be `action` or `transition`"),
            );
            None
        }
    };

    let mut params = Vec::new();
    let mut returns = Vec::new();
    let mut errors = Vec::new();
    let mut seen: Vec<&str> = Vec::new();

    if let Some(children) = node.children() {
        for block in children.nodes() {
            let offset = block.span().offset();
            let block_name = block.name().value();

            if !matches!(block_name, "params" | "returns" | "errors") {
                issues.push(
                    offset,
                    format!(
                        "unknown block `{block_name}` in command `{path}` (expected params, returns, errors)"
                    ),
                );
                continue;
            }
            if seen.contains(&block_name) {
                issues.push(
                    offset,
                    format!("duplicate `{block_name}` in command `{path}`"),
                );
                continue;
            }
            seen.push(block_name);

            if !block.entries().is_empty() {
                issues.push(
                    offset,
                    format!("`{block_name}` of command `{path}` takes no arguments or properties"),
                );
            }

            let mut block_paths = Paths::default();
            match block_name {
                "params" => {
                    if kind == Some(CommandKind::Transition) {
                        issues.push(
                            offset,
                            format!("transition `{path}` cannot take parameters"),
                        );
                    }
                    parse_values(
                        block.children(),
                        "",
                        ctx,
                        issues,
                        &mut block_paths,
                        &mut params,
                        true,
                    )
                }
                "returns" => parse_values(
                    block.children(),
                    "",
                    ctx,
                    issues,
                    &mut block_paths,
                    &mut returns,
                    true,
                ),
                _ => parse_errors(block.children(), &path, ctx, issues, &mut errors),
            }
        }
    }

    if issues.count() > before {
        return None;
    }

    Some(CommandDefinition {
        path,
        kind: kind?,
        params,
        returns,
        errors,
    })
}

fn parse_errors(
    children: Option<&KdlDocument>,
    command: &str,
    ctx: &Context,
    issues: &mut Issues,
    out: &mut Vec<ErrorDefinition>,
) {
    let Some(children) = children else {
        return;
    };

    for node in children.nodes() {
        let offset = node.span().offset();
        let Some(name) = node_name(node, issues) else {
            continue;
        };

        if out.iter().any(|e| e.name == name) {
            issues.push(
                offset,
                format!("duplicate error `{name}` in command `{command}`"),
            );
            continue;
        }
        if !node.entries().is_empty() {
            issues.push(
                offset,
                format!("error `{name}` of command `{command}` takes no arguments or properties"),
            );
        }

        let mut fields = Vec::new();
        parse_values(
            node.children(),
            "",
            ctx,
            issues,
            &mut Paths::default(),
            &mut fields,
            false,
        );
        out.push(ErrorDefinition { name, fields });
    }
}
