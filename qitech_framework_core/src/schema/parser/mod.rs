mod commands;
mod events;
mod helpers;
mod values;

use kdl::KdlDocument;
use kdl::KdlNode;

use crate::schema::MachineSchema;
use crate::schema::error::Issues;
use crate::schema::parser::commands::parse_commands;
use crate::schema::parser::events::parse_events;
use crate::schema::parser::helpers::Paths;
use crate::schema::parser::helpers::single_identifier;
use crate::schema::parser::values::parse_values;

const SECTIONS: &[&str] = &[
    "config",
    "state",
    "measurements",
    "inputs",
    "commands",
    "events",
];

pub(super) struct Context<'a> {
    pub(super) imports: &'a [String],
}

// --- schema ---
pub(super) fn parse_schema(document: &KdlDocument, issues: &mut Issues) -> Option<MachineSchema> {
    let mut imports: Vec<String> = Vec::new();
    let mut name: Option<String> = None;
    let mut name_seen = false;
    let mut sections: Vec<&KdlNode> = Vec::new();

    // header: imports first, then name; sections are collected and parsed afterwards
    for node in document.nodes() {
        let offset = node.span().offset();

        match node.name().value() {
            "import" => {
                if name_seen || !sections.is_empty() {
                    issues.push(offset, "`import` must come before `name` and all sections");
                }
                let Some(import) = single_identifier(node, "import", issues) else {
                    continue;
                };
                if imports.contains(&import) {
                    issues.push(offset, format!("duplicate import `{import}`"));
                    continue;
                }
                imports.push(import);
            }
            "name" => {
                if name_seen {
                    issues.push(offset, "duplicate `name`");
                    continue;
                }
                name_seen = true;
                if !sections.is_empty() {
                    issues.push(offset, "`name` must come before all sections");
                }
                name = single_identifier(node, "name", issues);
            }
            section if SECTIONS.contains(&section) => {
                if sections.iter().any(|n| n.name().value() == section) {
                    issues.push(offset, format!("duplicate section `{section}`"));
                    continue;
                }
                sections.push(node);
            }
            other => issues.push(
                offset,
                format!(
                    "unknown top-level node `{other}` (expected `import`, `name` or one of: {})",
                    SECTIONS.join(", ")
                ),
            ),
        }
    }

    if !name_seen {
        issues.push(0, "missing `name`");
    }

    let ctx = Context { imports: &imports };
    let mut config = Vec::new();
    let mut state = Vec::new();
    let mut measurements = Vec::new();
    let mut inputs = Vec::new();
    let mut commands = Vec::new();
    let mut events = Vec::new();

    for node in sections {
        let section = node.name().value();
        if !node.entries().is_empty() {
            issues.push(
                node.span().offset(),
                format!("section `{section}` takes no arguments or properties"),
            );
        }

        let children = node.children();
        let mut paths = Paths::default();

        match section {
            "config" => parse_values(children, "", &ctx, issues, &mut paths, &mut config, true),
            "state" => parse_values(children, "", &ctx, issues, &mut paths, &mut state, true),
            "measurements" => parse_values(
                children,
                "",
                &ctx,
                issues,
                &mut paths,
                &mut measurements,
                true,
            ),
            "inputs" => parse_values(children, "", &ctx, issues, &mut paths, &mut inputs, true),
            "commands" => parse_commands(children, "", &ctx, issues, &mut paths, &mut commands),
            "events" => parse_events(children, "", &ctx, issues, &mut paths, &mut events),
            _ => unreachable!("only known sections are collected"),
        }
    }

    Some(MachineSchema {
        imports: imports.clone(),
        name: name?,
        config,
        state,
        measurements,
        inputs,
        commands,
        events,
    })
}
