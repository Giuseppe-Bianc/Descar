use std::path::PathBuf;
use std::fmt;

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

    pub fn file_path(path: PathBuf) -> Self {
        // PathBuf in Rust non può essere nullo.
        SourceId::FilePath { path }
    }

    pub fn virtual_resource(uri: String) -> Result<Self, &'static str> {
        if uri.trim().is_empty() {
            return Err("uri must not be blank");
        }
        Ok(SourceId::VirtualResource { uri })
    }

    pub fn in_memory_module(module_name: String) -> Result<Self, &'static str> {
        if module_name.trim().is_empty() {
            return Err("moduleName must not be blank");
        }
        Ok(SourceId::InMemoryModule { module_name })
    }

    pub fn generated(description: String) -> Result<Self, &'static str> {
        if description.trim().is_empty() {
            return Err("description must not be blank");
        }
        Ok(SourceId::Generated { description })
    }
    
    /// Identificatore testuale stabile della sorgente.
    pub fn identifier(&self) -> String {
        match self {
            SourceId::FilePath { path } => path.to_string_lossy().into_owned(),
            SourceId::VirtualResource { uri } => uri.clone(),
            SourceId::InMemoryModule { module_name } => module_name.clone(),
            SourceId::Generated { description } => format!("<generated:{description}>"),
        }
    }

    /// Descrizione leggibile dall'utente, utile per log e messaggi diagnostici.
    pub fn describe(&self) -> String {
        match self {
            SourceId::FilePath { path } => format!("file: {}", path.display()),
            SourceId::VirtualResource { uri } => format!("virtual: {uri}"),
            SourceId::InMemoryModule { module_name } => format!("in-memory module: {module_name}"),
            SourceId::Generated { description } => format!("generated: {description}"),
        }
    }
}

// In Rust è idiomatico implementare Display se un tipo ha una rappresentazione testuale "human-readable"
impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}