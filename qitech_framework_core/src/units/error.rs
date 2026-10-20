use core::fmt;

// --- error ---
#[derive(Debug, Clone, PartialEq)]
pub struct UnitCatalogError {
    pub issues: Vec<Issue>,
}

impl UnitCatalogError {
    pub(super) fn from_kdl(input: &str, error: kdl::KdlError) -> Self {
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

// --- issue ---
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

// --- issue collector ---
/// Collects every issue instead of stopping at the first one.
pub(super) struct Issues<'a> {
    input: &'a str,
    pub(super) issues: Vec<Issue>,
}

impl<'a> Issues<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self {
            input,
            issues: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, offset: usize, message: impl Into<String>) {
        let (line, column) = line_column(self.input, offset);
        self.issues.push(Issue {
            line,
            column,
            message: message.into(),
        });
    }

    pub(super) fn finish<T>(self, value: T) -> Result<T, UnitCatalogError> {
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
