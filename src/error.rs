use std::error::Error;
use std::fmt;
use std::io;

pub type EnvResult<T> = Result<T, EnvError>;

#[derive(Debug)]
pub enum EnvError {
    Io(io::Error),
    InvalidEnvironment(String),
    InvalidPath(String),
    PathTraversal,
    NotFound(String),
    AlreadyExists(String),
    NotDirectory(String),
    IsDirectory(String),
    PermissionDenied(String),
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvError::Io(err) => write!(f, "I/O error: {}", err),
            EnvError::InvalidEnvironment(msg) => write!(f, "Invalid environment: {}", msg),
            EnvError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            EnvError::PathTraversal => write!(f, "Path traversal attempted"),
            EnvError::NotFound(msg) => write!(f, "Not found: {}", msg),
            EnvError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            EnvError::NotDirectory(msg) => write!(f, "Not a directory: {}", msg),
            EnvError::IsDirectory(msg) => write!(f, "Is a directory: {}", msg),
            EnvError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl Error for EnvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EnvError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for EnvError {
    fn from(error: io::Error) -> Self {
        EnvError::Io(error)
    }
}
