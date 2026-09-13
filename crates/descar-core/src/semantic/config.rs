use std::sync::Arc;

/// Version of the language rules used by semantic analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LanguageVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl LanguageVersion {
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
}

/// Configuration frozen for the lifetime of a semantic context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticConfig {
    language_version: LanguageVersion,
    runtime_namespace: Arc<str>,
}

impl SemanticConfig {
    #[must_use]
    pub fn new(language_version: LanguageVersion) -> Self {
        Self { language_version, runtime_namespace: Arc::from("__descar_builtin") }
    }

    #[must_use]
    pub fn with_runtime_namespace(mut self, namespace: impl Into<Arc<str>>) -> Self {
        self.runtime_namespace = namespace.into();
        self
    }

    #[must_use]
    pub const fn language_version(&self) -> LanguageVersion {
        self.language_version
    }

    #[must_use]
    pub fn runtime_namespace(&self) -> &str {
        &self.runtime_namespace
    }
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self::new(LanguageVersion::new(0, 1, 0))
    }
}
