use std::fmt;
use std::path::PathBuf;

/// Identifica la sorgente. L'ID è valido per l'intera compilazione.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceId {
    /// Sorgente corrispondente a un file sul filesystem.
    FilePath { path: PathBuf },

    /// Sorgente di una risorsa virtuale (URI, JAR, URL).
    VirtualResource { uri: String },

    /// Sorgente di un modulo in memoria (REPL, eval).
    InMemoryModule { module_name: String },

    /// Sorgente generata dal compilatore (macro, ecc.).
    Generated { description: String },
}

impl SourceId {
    #[must_use]
    pub const fn file_path(path: PathBuf) -> Self {
        // PathBuf in Rust non può essere nullo.
        Self::FilePath { path }
    }

    /// # Errors
    ///
    /// Returns `Err` if `uri` is blank (empty or whitespace-only).
    pub fn virtual_resource(uri: String) -> Result<Self, &'static str> {
        if uri.trim().is_empty() {
            return Err("uri must not be blank");
        }
        Ok(Self::VirtualResource { uri })
    }

    /// # Errors
    ///
    /// Returns `Err` if `module_name` is blank (empty or whitespace-only).
    pub fn in_memory_module(module_name: String) -> Result<Self, &'static str> {
        if module_name.trim().is_empty() {
            return Err("moduleName must not be blank");
        }
        Ok(Self::InMemoryModule { module_name })
    }

    /// # Errors
    ///
    /// Returns `Err` if `description` is blank (empty or whitespace-only).
    pub fn generated(description: String) -> Result<Self, &'static str> {
        if description.trim().is_empty() {
            return Err("description must not be blank");
        }
        Ok(Self::Generated { description })
    }

    /// Identificatore testuale stabile della sorgente.
    #[must_use]
    pub fn identifier(&self) -> String {
        match self {
            Self::FilePath { path } => path.to_string_lossy().into_owned(),
            Self::VirtualResource { uri } => uri.clone(),
            Self::InMemoryModule { module_name } => module_name.clone(),
            Self::Generated { description } => format!("<generated:{description}>"),
        }
    }

    /// Descrizione leggibile dall'utente, utile per log e messaggi diagnostici.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::FilePath { path } => format!("file: {}", path.display()),
            Self::VirtualResource { uri } => format!("virtual: {uri}"),
            Self::InMemoryModule { module_name } => format!("in-memory module: {module_name}"),
            Self::Generated { description } => format!("generated: {description}"),
        }
    }
}

// In Rust è idiomatico implementare Display se un tipo ha una rappresentazione testuale "human-readable"
impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}
