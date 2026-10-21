use kdl::KdlEntry;
use kdl::KdlNode;

use crate::schema::Issues;

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

/// Rejects arguments and properties outside `allowed`.
pub(super) fn check_entries(node: &KdlNode, what: &str, allowed: &[&str], issues: &mut Issues) {
    for entry in node.entries() {
        match entry.name() {
            None => issues.push(
                entry.span().offset(),
                format!(
                    "{what} takes no arguments (allowed properties: {})",
                    allowed.join(", ")
                ),
            ),
            Some(key) if !allowed.contains(&key.value()) => issues.push(
                entry.span().offset(),
                format!(
                    "unknown property `{}` on {what} (allowed: {})",
                    key.value(),
                    allowed.join(", ")
                ),
            ),
            Some(_) => {}
        }
    }
}

/// Last occurrence wins, as in the KDL spec.
pub(super) fn property<'a>(node: &'a KdlNode, key: &str) -> Option<&'a KdlEntry> {
    node.entries()
        .iter()
        .rev()
        .find(|e| e.name().is_some_and(|n| n.value() == key))
}

pub(super) fn integer<T: TryFrom<i128>>(entry: &KdlEntry) -> Option<T> {
    entry.value().as_integer().and_then(|v| T::try_from(v).ok())
}
