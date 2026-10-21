use core::fmt;

// --- error ---
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaError {
    pub issues: Vec<Issue>,
}

impl SchemaError {
    pub(crate) fn from_kdl(input: &str, error: kdl::KdlError) -> Self {
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

impl fmt::Display for SchemaError {
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

impl std::error::Error for SchemaError {}

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
pub(crate) struct Issues<'a> {
    input: &'a str,
    issues: Vec<Issue>,
}

impl<'a> Issues<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            issues: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, offset: usize, message: impl Into<String>) {
        let (line, column) = line_column(self.input, offset);
        self.issues.push(Issue {
            line,
            column,
            message: message.into(),
        });
    }

    pub(crate) fn count(&self) -> usize {
        self.issues.len()
    }

    pub(crate) fn finish<T>(self, value: Option<T>) -> Result<T, SchemaError> {
        match value {
            Some(value) if self.issues.is_empty() => Ok(value),
            _ => Err(SchemaError {
                issues: self.issues,
            }),
        }
    }
}

fn line_column(input: &str, offset: usize) -> (usize, usize) {
    let before = &input[..offset.min(input.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, column)
}
