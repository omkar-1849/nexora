use std::fmt;
use std::path::PathBuf;

use crate::error::{EnvError, EnvResult};

/// A path inside the AI-Native Environment.
///
/// Examples:
///     /
///     /home
///     /workspace
///     /workspace/projects
///     /workspace/projects/main.rs
///
/// This type never exposes or stores a Windows filesystem path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualPath {
    components: Vec<String>,
}

fn validate_component(component: &str) -> EnvResult<()> {
    if component.is_empty() {
        return Err(EnvError::InvalidPath("Path component cannot be empty.".to_string()));
    }

    if component == "." || component == ".." {
        return Err(EnvError::PathTraversal);
    }

    if component.contains('/') || component.contains('\\') {
        return Err(EnvError::InvalidPath(
            "Backslashes and slashes are not allowed in path components.".to_string(),
        ));
    }

    if component.contains(':') {
        return Err(EnvError::InvalidPath(
            "Colons are not allowed in path components.".to_string(),
        ));
    }

    if component.ends_with('.') || component.ends_with(' ') {
        return Err(EnvError::InvalidPath(
            "Path components cannot end with a dot or space.".to_string(),
        ));
    }

    let base = component.split('.').next().unwrap_or(component).to_ascii_uppercase();
    const RESERVED_NAMES: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    if RESERVED_NAMES.contains(&base.as_str()) {
        return Err(EnvError::InvalidPath(format!(
            "'{}' is a reserved device name.",
            component
        )));
    }

    Ok(())
}

impl VirtualPath {
    /// Create the environment root path: /
    pub fn root() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Parse an environment-native path.
    pub fn parse(path: &str) -> EnvResult<Self> {
        if path.is_empty() {
            return Err(EnvError::InvalidPath("Path cannot be empty.".to_string()));
        }

        // Environment paths must always be absolute.
        if !path.starts_with('/') {
            return Err(EnvError::InvalidPath(
                "Environment paths must start with '/'.".to_string(),
            ));
        }

        let mut components = Vec::new();

        for component in path.split('/') {
            if component.is_empty() || component == "." {
                continue;
            }

            // Never allow traversal outside our environment.
            if component == ".." {
                return Err(EnvError::PathTraversal);
            }

            validate_component(component)?;

            components.push(component.to_string());
        }

        Ok(Self { components })
    }

    /// Return the root path.
    pub fn is_root(&self) -> bool {
        self.components.is_empty()
    }

    /// Return the final component of the path.
    pub fn file_name(&self) -> Option<&str> {
        self.components.last().map(String::as_str)
    }

    /// Return the parent environment path.
    pub fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            return None;
        }

        let mut components = self.components.clone();
        components.pop();

        Some(Self { components })
    }

    /// Append one component to this virtual path.
    pub fn join(&self, name: &str) -> EnvResult<Self> {
        validate_component(name)?;

        let mut components = self.components.clone();
        components.push(name.to_string());

        Ok(Self { components })
    }

    /// Convert the virtual path to a host-relative PathBuf.
    ///
    /// This is deliberately relative. The filesystem backend is
    /// responsible for attaching the controlled environment root.
    pub fn to_relative_path(&self) -> PathBuf {
        let mut path = PathBuf::new();

        for component in &self.components {
            path.push(component);
        }

        path
    }

    /// Access the individual path components.
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

impl fmt::Display for VirtualPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.components.is_empty() {
            return write!(formatter, "/");
        }

        write!(formatter, "/{}", self.components.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_path_works() {
        let path = VirtualPath::parse("/").unwrap();

        assert!(path.is_root());
        assert_eq!(path.to_string(), "/");
    }

    #[test]
    fn normal_path_works() {
        let path = VirtualPath::parse("/workspace/projects/main.rs").unwrap();

        assert_eq!(path.to_string(), "/workspace/projects/main.rs");
        assert_eq!(path.file_name(), Some("main.rs"));
    }

    #[test]
    fn parent_works() {
        let path = VirtualPath::parse("/workspace/projects/main.rs").unwrap();

        let parent = path.parent().unwrap();

        assert_eq!(parent.to_string(), "/workspace/projects");
    }

    #[test]
    fn relative_paths_are_rejected() {
        let result = VirtualPath::parse("workspace/projects");

        assert!(result.is_err());
    }

    #[test]
    fn traversal_is_rejected() {
        let result = VirtualPath::parse("/workspace/../Windows");

        assert!(matches!(result, Err(EnvError::PathTraversal)));
    }

    #[test]
    fn backslashes_are_rejected() {
        let result = VirtualPath::parse(r"/workspace\projects\file.txt");

        assert!(result.is_err());
    }

    #[test]
    fn join_works() {
        let root = VirtualPath::root();

        let workspace = root.join("workspace").unwrap();
        let file = workspace.join("test.txt").unwrap();

        assert_eq!(file.to_string(), "/workspace/test.txt");
    }
}
