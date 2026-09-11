use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{EnvError, EnvResult};
use crate::filesystem::VirtualPath;

/// Controlled storage backend for the AI-Native filesystem.
///
/// This is the ONLY layer that translates environment-native
/// virtual paths into physical host paths.
#[derive(Debug, Clone)]
pub struct StorageBackend {
    root: PathBuf,
}

impl StorageBackend {
    /// Create a backend rooted at the supplied host directory.
    pub fn new(root: impl Into<PathBuf>) -> EnvResult<Self> {
        let root = root.into();

        fs::create_dir_all(&root)?;

        let root = fs::canonicalize(&root)?;

        Ok(Self { root })
    }

    /// Return the physical root used by this backend.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Convert an environment VirtualPath into a controlled
    /// physical path.
    ///
    /// The resulting path is guaranteed to remain inside the
    /// backend root.
    pub fn resolve(&self, virtual_path: &VirtualPath) -> EnvResult<PathBuf> {
        let relative = virtual_path.to_relative_path();

        let physical = self.root.join(relative);

        self.validate_containment(&physical)?;

        Ok(physical)
    }

    /// Create a directory.
    pub fn create_directory(&self, virtual_path: &VirtualPath) -> EnvResult<()> {
        let physical = self.resolve(virtual_path)?;

        if physical.exists() {
            return Err(EnvError::AlreadyExists(virtual_path.to_string()));
        }

        fs::create_dir(&physical)?;

        Ok(())
    }

    /// Create an empty file.
    pub fn create_file(&self, virtual_path: &VirtualPath) -> EnvResult<()> {
        let physical = self.resolve(virtual_path)?;

        if physical.exists() {
            return Err(EnvError::AlreadyExists(virtual_path.to_string()));
        }

        fs::File::create(&physical)?;

        Ok(())
    }

    /// Read file contents.
    pub fn read_file(&self, virtual_path: &VirtualPath) -> EnvResult<Vec<u8>> {
        let physical = self.resolve(virtual_path)?;

        if !physical.exists() {
            return Err(EnvError::NotFound(virtual_path.to_string()));
        }

        if physical.is_dir() {
            return Err(EnvError::IsDirectory(virtual_path.to_string()));
        }

        Ok(fs::read(physical)?)
    }

    /// Write file contents.
    pub fn write_file(&self, virtual_path: &VirtualPath, data: &[u8]) -> EnvResult<()> {
        let physical = self.resolve(virtual_path)?;

        if !physical.exists() {
            return Err(EnvError::NotFound(virtual_path.to_string()));
        }

        if physical.is_dir() {
            return Err(EnvError::IsDirectory(virtual_path.to_string()));
        }

        fs::write(physical, data)?;

        Ok(())
    }

    /// List the direct children of a directory.
    pub fn list_directory(&self, virtual_path: &VirtualPath) -> EnvResult<Vec<String>> {
        let physical = self.resolve(virtual_path)?;

        if !physical.exists() {
            return Err(EnvError::NotFound(virtual_path.to_string()));
        }

        if !physical.is_dir() {
            return Err(EnvError::NotDirectory(virtual_path.to_string()));
        }

        let mut entries = Vec::new();

        for entry in fs::read_dir(physical)? {
            let entry = entry?;

            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }

        entries.sort();

        Ok(entries)
    }

    /// Remove a file.
    pub fn remove_file(&self, virtual_path: &VirtualPath) -> EnvResult<()> {
        let physical = self.resolve(virtual_path)?;

        if !physical.exists() {
            return Err(EnvError::NotFound(virtual_path.to_string()));
        }

        if physical.is_dir() {
            return Err(EnvError::IsDirectory(virtual_path.to_string()));
        }

        fs::remove_file(physical)?;

        Ok(())
    }

    /// Remove an empty directory.
    pub fn remove_directory(&self, virtual_path: &VirtualPath) -> EnvResult<()> {
        let physical = self.resolve(virtual_path)?;

        if !physical.exists() {
            return Err(EnvError::NotFound(virtual_path.to_string()));
        }

        if !physical.is_dir() {
            return Err(EnvError::NotDirectory(virtual_path.to_string()));
        }

        // Deliberately use remove_dir instead of remove_dir_all.
        // Recursive deletion will be implemented separately and
        // protected by the filesystem permission layer.
        fs::remove_dir(physical)?;

        Ok(())
    }

    /// Ensure a physical path cannot escape our backend root.
    fn validate_containment(&self, physical: &Path) -> EnvResult<()> {
        // The path may not exist yet, so canonicalize the nearest
        // existing ancestor.
        let mut existing = physical;

        while !existing.exists() {
            existing = existing.parent().ok_or_else(|| {
                EnvError::InvalidPath("Unable to resolve storage path.".to_string())
            })?;
        }

        let canonical_existing = fs::canonicalize(existing)?;

        if !canonical_existing.starts_with(&self.root) {
            return Err(EnvError::PathTraversal);
        }

        Ok(())
    }
}
