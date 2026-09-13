use crate::{error::compile_error::CompileError, location::source_span::SourceSpan};

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
    #[must_use]
    pub fn error(error: CompileError) -> Self {
        let message = error.to_string();
        let span = error.span().cloned();
        Self { severity: DiagnosticSeverity::Error, error: Some(error), message, span }
    }

    #[must_use]
    pub fn internal(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self { severity: DiagnosticSeverity::Internal, error: None, message: message.into(), span }
    }

    #[must_use]
    pub fn severity(&self) -> DiagnosticSeverity { self.severity }

    #[must_use]
    pub fn message(&self) -> &str { &self.message }

    #[must_use]
    pub fn span(&self) -> Option<&SourceSpan> { self.span.as_ref() }

    #[must_use]
    pub fn compile_error(&self) -> Option<&CompileError> { self.error.as_ref() }
}

#[derive(Debug, Default)]
pub struct DiagnosticEngine {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticEngine {
    #[must_use]
    pub const fn new() -> Self { Self { diagnostics: Vec::new() } }

    pub fn emit(&mut self, diagnostic: Diagnostic) { self.diagnostics.push(diagnostic); }

    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] { &self.diagnostics }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.severity, DiagnosticSeverity::Error | DiagnosticSeverity::Fatal | DiagnosticSeverity::Internal))
    }

    #[must_use]
    pub fn len(&self) -> usize { self.diagnostics.len() }

    #[must_use]
    pub const fn is_empty(&self) -> bool { self.diagnostics.is_empty() }
}
