//! Diagnostics collected during loading and validation, reported together at the end.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub file: String,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{tag}: {}: {}", self.file, self.message)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn error(&mut self, file: impl Into<String>, message: impl Into<String>) {
        self.items.push(Diagnostic {
            severity: Severity::Error,
            file: file.into(),
            message: message.into(),
        });
    }

    pub fn warning(&mut self, file: impl Into<String>, message: impl Into<String>) {
        self.items.push(Diagnostic {
            severity: Severity::Warning,
            file: file.into(),
            message: message.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter().filter(|d| d.severity == Severity::Error)
    }

    /// Whether any error message contains `needle` (used heavily by tests).
    pub fn has_error_containing(&self, needle: &str) -> bool {
        self.errors().any(|d| d.message.contains(needle))
    }

    pub fn report(&self) -> String {
        let mut out = String::new();
        for d in &self.items {
            out.push_str(&d.to_string());
            out.push('\n');
        }
        out
    }
}
