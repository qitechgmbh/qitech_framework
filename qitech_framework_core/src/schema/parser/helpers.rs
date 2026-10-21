use std::collections::HashSet;

use kdl::KdlDocument;
use kdl::KdlEntry;
use kdl::KdlNode;
use kdl::KdlValue;

use crate::schema::error::Issues;

#[derive(Default)]
pub(super) struct Paths(HashSet<String>);

impl Paths {
    pub(super) fn claim(&mut self, path: &str, offset: usize, issues: &mut Issues) -> bool {
        if self.0.insert(path.to_owned()) {
            true
        } else {
            issues.push(offset, format!("duplicate `{path}`"));
            false
        }
    }
}

pub(super) fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}

pub(super) fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('a'..='z'))
        && chars.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
}

pub(super) fn node_name(node: &KdlNode, issues: &mut Issues) -> Option<String> {
    let name = node.name().value();
    if is_identifier(name) {
        Some(name.to_owned())
    } else {
        issues.push(
            node.span().offset(),
            format!(
                "`{name}` is not a valid name (use lowercase letters, digits and `_`, starting with a letter)"
            ),
        );
        None
    }
}

/// `import si` / `name winder_v1`: exactly one identifier argument, nothing else.
pub(super) fn single_identifier(node: &KdlNode, what: &str, issues: &mut Issues) -> Option<String> {
    let offset = node.span().offset();

    if node.children().is_some() {
        issues.push(offset, format!("`{what}` cannot have children"));
    }

    match node.entries() {
        [entry] if entry.name().is_none() => {
            match entry.value().as_string().filter(|s| is_identifier(s)) {
                Some(value) => Some(value.to_owned()),
                None => {
                    issues.push(
                        entry.span().offset(),
                        format!("`{what}` must be a lowercase name like `winder_v1`"),
                    );
                    None
                }
            }
        }
        _ => {
            issues.push(offset, format!("`{what}` takes exactly one name"));
            None
        }
    }
}

pub(super) fn arguments(node: &KdlNode) -> Vec<&KdlEntry> {
    node.entries()
        .iter()
        .filter(|e| e.name().is_none())
        .collect()
}

pub(super) fn has_arguments(node: &KdlNode) -> bool {
    node.entries().iter().any(|e| e.name().is_none())
}

pub(super) fn has_properties(node: &KdlNode) -> bool {
    node.entries().iter().any(|e| e.name().is_some())
}

/// Last occurrence wins, as in the KDL spec.
pub(super) fn property<'a>(node: &'a KdlNode, key: &str) -> Option<&'a KdlEntry> {
    node.entries()
        .iter()
        .rev()
        .find(|e| e.name().is_some_and(|n| n.value() == key))
}

pub(super) fn bool_property(
    node: &KdlNode,
    key: &str,
    path: &str,
    issues: &mut Issues,
) -> Option<bool> {
    let entry = property(node, key)?;
    match entry.value() {
        KdlValue::Bool(b) => Some(*b),
        _ => {
            issues.push(
                entry.span().offset(),
                format!("`{key}` of `{path}` must be #true or #false"),
            );
            None
        }
    }
}

pub(super) fn non_empty(children: Option<&KdlDocument>) -> Option<&KdlDocument> {
    children.filter(|c| !c.nodes().is_empty())
}
