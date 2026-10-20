//! Unit catalog read from `units.kdl`.
//!
//! Top-level nodes are quantities, their children are units:
//!
//! ```kdl
//! length {
//!     meter      symbol="m"  base=#true
//!     millimeter symbol="mm" factor=1e-3
//! }
//!
//! sound_pressure {
//!     pascal symbol="Pa" base=#true
//!     db_spl symbol="dB" log10_multiplier=20 log10_reference=2e-5
//! }
//! ```
//!
//! A unit is referenced as `quantity.unit`, e.g. `length.millimeter`.

use core::fmt;

use kdl::KdlDocument;
use kdl::KdlNode;
use kdl::KdlValue;
use serde::Deserialize;
use serde::Serialize;

// --- catalog ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitCatalog {
    pub quantities: Vec<Quantity>,
}

impl UnitCatalog {
    pub fn parse_str(input: &str) -> Result<Self, UnitCatalogError> {
        let document: KdlDocument = input
            .parse()
            .map_err(|e: kdl::KdlError| UnitCatalogError::from_kdl(input, e))?;

        let mut issues = Issues::new(input);
        let catalog = parse_catalog(&document, &mut issues);
        issues.finish(catalog)
    }

    pub fn quantity(&self, name: &str) -> Option<&Quantity> {
        self.quantities.iter().find(|q| q.name == name)
    }

    /// Looks up a unit by its reference, e.g. `length.millimeter`.
    pub fn unit(&self, reference: &str) -> Option<(&Quantity, &Unit)> {
        let (quantity, unit) = reference.split_once('.')?;
        let quantity = self.quantity(quantity)?;
        Some((quantity, quantity.unit(unit)?))
    }
}

// --- quantity ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub name: String,
    pub units: Vec<Unit>,
}

impl Quantity {
    pub fn unit(&self, name: &str) -> Option<&Unit> {
        self.units.iter().find(|u| u.name == name)
    }

    /// The base unit. `None` means units of this quantity are not convertible.
    pub fn base(&self) -> Option<&Unit> {
        self.units.iter().find(|u| u.base)
    }

    pub fn is_convertible(&self) -> bool {
        self.base().is_some()
    }
}

// --- unit ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub name: String,
    pub symbol: String,
    pub base: bool,
    pub conversion: Conversion,
}

/// Conversion of a unit value into the base unit of its quantity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Conversion {
    /// The quantity has no base unit; values can only be displayed.
    None,
    /// `base = value * factor + offset`
    Affine { factor: f64, offset: f64 },
    /// `base = reference * 10^(value / multiplier)`
    Log10 { multiplier: f64, reference: f64 },
}

impl Conversion {
    pub const IDENTITY: Self = Self::Affine {
        factor: 1.0,
        offset: 0.0,
    };

    pub fn to_base(&self, value: f64) -> Option<f64> {
        match *self {
            Self::None => None,
            Self::Affine { factor, offset } => Some(value * factor + offset),
            Self::Log10 {
                multiplier,
                reference,
            } => Some(reference * 10f64.powf(value / multiplier)),
        }
    }

    pub fn from_base(&self, value: f64) -> Option<f64> {
        match *self {
            Self::None => None,
            Self::Affine { factor, offset } => Some((value - offset) / factor),
            Self::Log10 {
                multiplier,
                reference,
            } => Some(multiplier * (value / reference).log10()),
        }
    }
}

// --- errors ---
#[derive(Debug, Clone, PartialEq)]
pub struct UnitCatalogError {
    pub issues: Vec<Issue>,
}

impl UnitCatalogError {
    fn from_kdl(input: &str, error: kdl::KdlError) -> Self {
        let mut issues = Issues::new(input);

        for diagnostic in error.diagnostics {
            let message = diagnostic
                .message
                .or(diagnostic.label)
                .unwrap_or_else(|| "syntax error".to_owned());
            issues.push(diagnostic.span.offset(), message);
        }

        Self {
            issues: issues.issues,
        }
    }
}

impl fmt::Display for UnitCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, issue) in self.issues.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{issue}")?;
        }
        Ok(())
    }
}

impl std::error::Error for UnitCatalogError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// 1-based line in the source.
    pub line: usize,
    /// 1-based column in the source.
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

/// Collects every issue instead of stopping at the first one.
struct Issues<'a> {
    input: &'a str,
    issues: Vec<Issue>,
}

impl<'a> Issues<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            issues: Vec::new(),
        }
    }

    fn push(&mut self, offset: usize, message: impl Into<String>) {
        let (line, column) = line_column(self.input, offset);
        self.issues.push(Issue {
            line,
            column,
            message: message.into(),
        });
    }

    fn finish<T>(self, value: T) -> Result<T, UnitCatalogError> {
        if self.issues.is_empty() {
            Ok(value)
        } else {
            Err(UnitCatalogError {
                issues: self.issues,
            })
        }
    }
}

fn line_column(input: &str, offset: usize) -> (usize, usize) {
    let before = &input[..offset.min(input.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, column)
}

// --- parser ---
fn parse_catalog(document: &KdlDocument, issues: &mut Issues) -> UnitCatalog {
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

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
length {
    meter      symbol="m"  base=#true
    millimeter symbol="mm" factor=1e-3
    inch       symbol="in" factor=0.0254
}

temperature {
    kelvin  symbol="K"  base=#true
    celsius symbol="°C" offset=273.15
}

sound_pressure {
    pascal symbol="Pa" base=#true
    db_spl symbol="dB" log10_multiplier=20 log10_reference=2e-5
}

hardness_shore_a {
    shore_a symbol="Shore A"
}
"#;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
    }

    #[test]
    fn parses_example() {
        let catalog = UnitCatalog::parse_str(EXAMPLE).unwrap();
        assert_eq!(catalog.quantities.len(), 4);

        let (quantity, mm) = catalog.unit("length.millimeter").unwrap();
        assert_eq!(quantity.base().unwrap().name, "meter");
        assert_eq!(mm.symbol, "mm");
        assert!(approx(mm.conversion.to_base(1500.0).unwrap(), 1.5));

        let (_, celsius) = catalog.unit("temperature.celsius").unwrap();
        assert!(approx(celsius.conversion.to_base(20.0).unwrap(), 293.15));
        assert!(approx(celsius.conversion.from_base(273.15).unwrap(), 0.0));

        let (_, db) = catalog.unit("sound_pressure.db_spl").unwrap();
        assert!(approx(
            db.conversion.to_base(94.0).unwrap(),
            1.0023744672545445
        ));
        assert!(approx(db.conversion.from_base(2e-5).unwrap(), 0.0));

        let shore = catalog.quantity("hardness_shore_a").unwrap();
        assert!(!shore.is_convertible());
        assert_eq!(shore.units[0].conversion, Conversion::None);
    }

    #[test]
    fn collects_all_issues() {
        let input = r#"
length {
    meter symbol="m" base=#true
    foot symbol="ft" base=#true
    inch factor=0.0254
    yard symbol="yd" factor=0.9144 log10_reference=1.0
    rod symbol="rd" colour="red"
}
length {
    meter symbol="m" base=#true
}
"#;
        let error = UnitCatalog::parse_str(input).unwrap_err();
        let messages: Vec<&str> = error.issues.iter().map(|i| i.message.as_str()).collect();

        assert!(
            messages
                .iter()
                .any(|m| m.contains("more than one base unit"))
        );
        assert!(messages.iter().any(|m| m.contains("missing `symbol`")));
        assert!(messages.iter().any(|m| m.contains("mixes")));
        assert!(
            messages
                .iter()
                .any(|m| m.contains("unknown property `colour`"))
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("duplicate quantity `length`"))
        );
        assert_eq!(error.issues[0].line, 4);
    }

    #[test]
    fn rejects_conversion_without_base() {
        let input = r#"
scale {
    a symbol="a"
    b symbol="b" factor=2
}
"#;
        let error = UnitCatalog::parse_str(input).unwrap_err();
        assert!(error.issues[0].message.contains("has no base unit"));
    }

    #[test]
    fn reports_syntax_errors_with_position() {
        let error = UnitCatalog::parse_str("length {\n    meter symbol=\"m\n").unwrap_err();
        assert!(!error.issues.is_empty());
    }
}
