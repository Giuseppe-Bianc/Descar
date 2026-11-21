// crates/descar-core/src/error/internal.rs

//! Errors about source files and I/O operations.
//!
//! [`DescarError`] describes a source file with an unsupported extension, a source file
//! that cannot be read, and an I/O failure.

use thiserror::Error;

/// Errors about source files and I/O operations.
#[derive(Debug, Error)]
pub enum DescarError {
    /// The source file has an unsupported extension.
    #[error("unsupported source file extension: expected `.dr`, found `{extension}`")]
    UnsupportedSourceExtension {
        /// The extension of the source file.
        extension: String,
    },

    /// A source file could not be read.
    #[error("failed to read source file `{path}`: {message}")]
    SourceRead {
        /// The path of the source file.
        path: String,
        /// The reason that the read operation failed.
        message: String,
    },

    /// An I/O operation failed.
    ///
    /// The variant wraps a [`std::io::Error`]. The `From` implementation converts a
    /// [`std::io::Error`] into this variant.
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
