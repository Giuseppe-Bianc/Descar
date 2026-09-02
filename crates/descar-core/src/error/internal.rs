//! Error types produced by the Descar compiler core.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpanError {
    /// `end` precedes `start`.
    #[error("end offset ({end_offset}) must not precede start offset ({start_offset})")]
    EndBeforeStart { start_offset: i64, end_offset: i64 },

    #[error("offset ({offset}) is out of range for usize")]
    OffsetOutOfRange { offset: i64 },
}

/// Errors that can occur while processing Descar source code.
#[derive(Debug, Error)]
pub enum DescarError {
    /// The source file has an unsupported extension.
    #[error("unsupported source file extension: expected `.dr`, found `{extension}`")]
    UnsupportedSourceExtension { extension: String },

    /// A source file could not be read.
    #[error("failed to read source file `{path}`: {message}")]
    SourceRead { path: String, message: String },

    #[error("error from span: {source}")]
    SpanError {
        #[from]
        source: SpanError,
    },

    /// I/O operation failure during compilation (e.g., file access issues).
    ///
    /// Wraps the standard [`std::io::Error`] for seamless error propagation.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
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
