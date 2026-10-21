use kdl::KdlDocument;

use crate::schema::EventDefinition;
use crate::schema::error::Issues;
use crate::schema::parser::Context;
use crate::schema::parser::helpers::Paths;
use crate::schema::parser::helpers::has_arguments;
use crate::schema::parser::helpers::join;
use crate::schema::parser::helpers::node_name;
use crate::schema::parser::values::parse_values;

// --- events ---
/// A node without children, or whose children have types, is an event; otherwise it is a group.
pub(super) fn parse_events(
    children: Option<&KdlDocument>,
    prefix: &str,
    ctx: &Context,
    issues: &mut Issues,
    paths: &mut Paths,
    out: &mut Vec<EventDefinition>,
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

        if !node.entries().is_empty() {
            issues.push(
                offset,
                format!("event or group `{path}` takes no arguments or properties"),
            );
        }

        let kids = node.children().map(|c| c.nodes()).unwrap_or(&[]);

        if kids.is_empty() {
            out.push(EventDefinition {
                path,
                fields: Vec::new(),
            });
        } else if kids.iter().any(has_arguments) {
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
            out.push(EventDefinition { path, fields });
        } else {
            parse_events(node.children(), &path, ctx, issues, paths, out);
        }
    }
}
