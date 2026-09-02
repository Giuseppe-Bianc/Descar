//! Error types produced by the Descar compiler core.

use thiserror::Error;

/// Errors that can occur while processing Descar source code.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DescarError {
    /// The source file has an unsupported extension.
    #[error("unsupported source file extension: expected `.dr`, found `{extension}`")]
    UnsupportedSourceExtension { extension: String },

    /// A source file could not be read.
    #[error("failed to read source file `{path}`: {message}")]
    SourceRead { path: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::DescarError;

    #[test]
    fn formats_unsupported_extension() {
        let error = DescarError::UnsupportedSourceExtension { extension: "txt".into() };

        assert_eq!(error.to_string(), "unsupported source file extension: expected `.dr`, found `txt`");
    }

    #[test]
    fn preserves_source_read_context() {
        let error = DescarError::SourceRead { path: "examples/hello.dr".into(), message: "permission denied".into() };

        assert_eq!(error.to_string(), "failed to read source file `examples/hello.dr`: permission denied");
    }
}
