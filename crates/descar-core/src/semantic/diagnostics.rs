use crate::{error::compile_error::CompileError, location::source_span::SourceSpan};

/// Severity assigned to a semantic diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
    Fatal,
    Internal,
    Recovery,
}

#[derive(Debug)]
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    error: Option<CompileError>,
    message: String,
    span: Option<SourceSpan>,
}

impl Diagnostic {
    /// Creates a diagnostic backed by a compiler error.
    #[must_use]
    pub fn error(error: CompileError) -> Self {
        let message = error.to_string();
        let span = error.span().cloned();
        Self { severity: DiagnosticSeverity::Error, error: Some(error), message, span }
    }

    /// Creates an internal diagnostic with an optional source span.
    #[must_use]
    pub fn internal(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self { severity: DiagnosticSeverity::Internal, error: None, message: message.into(), span }
    }

    /// Returns the diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the human-readable diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the source span associated with the diagnostic, when present.
    #[must_use]
    pub const fn span(&self) -> Option<&SourceSpan> {
        self.span.as_ref()
    }

    /// Returns the underlying compiler error, when the diagnostic has one.
    #[must_use]
    pub const fn compile_error(&self) -> Option<&CompileError> {
        self.error.as_ref()
    }
}

#[derive(Debug, Default)]
pub struct DiagnosticEngine {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticEngine {
    /// Creates an empty diagnostic engine.
    #[must_use]
    pub const fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }

    /// Records a diagnostic in insertion order.
    pub fn emit(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Returns all diagnostics currently recorded by the engine.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether the engine contains an error-level or internal diagnostic.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| {
            matches!(d.severity, DiagnosticSeverity::Error | DiagnosticSeverity::Fatal | DiagnosticSeverity::Internal)
        })
    }

    /// Returns the number of diagnostics currently stored.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns whether no diagnostics have been recorded.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
