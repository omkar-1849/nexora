pub mod backend;
pub mod metadata;
pub mod metadata_store;
pub mod node;
pub mod path;
pub mod permissions;
pub mod user;


use std::collections::HashMap;
use std::path::PathBuf;

use backend::StorageBackend;
use metadata_store::MetadataStore;
use permissions::check_permission;
use user::UserManager;

pub use metadata::FileMetadata;
pub use node::{FileNode, NodeType};
pub use path::VirtualPath;
pub use permissions::{Permission, DEFAULT_DIRECTORY_PERMISSIONS, DEFAULT_FILE_PERMISSIONS};
pub use user::{User, DEFAULT_USER_ID, ROOT_USER_ID};

use crate::error::{EnvError, EnvResult};

pub struct FileSystem {
    backend: StorageBackend,

    // Persistent Tokio runtime.
    // This runtime lives as long as the FileSystem.
    tokio_runtime: tokio::runtime::Runtime,

    metadata_store: MetadataStore,

    users: UserManager,
    nodes: HashMap<String, FileNode>,
    current_user: u32,
}

impl FileSystem {
    pub fn new(environment_root: PathBuf, postgres_url: &str) -> EnvResult<Self> {
        let backend = StorageBackend::new(environment_root)?;

        // Create ONE persistent Tokio runtime.
        let tokio_runtime = tokio::runtime::Runtime::new().map_err(|error| {
            EnvError::InvalidEnvironment(format!("Failed to create Tokio runtime: {}", error))
        })?;

        // Connect PostgreSQL using that same persistent runtime.
        let metadata_store = tokio_runtime.block_on(MetadataStore::connect(postgres_url))?;

        let mut filesystem = Self {
            backend,
            tokio_runtime,
            metadata_store,
            users: UserManager::new(),
            nodes: HashMap::new(),
            current_user: DEFAULT_USER_ID,
        };

        filesystem.initialize_metadata()?;
        filesystem.load_metadata_from_database()?;
        filesystem.verify_storage_consistency()?;

        Ok(filesystem)
    }

    // ---------------------------------------------------------
    // ASYNC BRIDGE
    // ---------------------------------------------------------
    //
    // IMPORTANT:
    // We DO NOT create a new Tokio runtime here.
    //
    // Every async filesystem/database operation runs on the
    // persistent runtime owned by this FileSystem.
    //

    fn run_async<F, T>(&self, future: F) -> EnvResult<T>
    where
        F: std::future::Future<Output = EnvResult<T>>,
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| self.tokio_runtime.block_on(future))
        } else {
            self.tokio_runtime.block_on(future)
        }
    }

    // ---------------------------------------------------------
    // METADATA INITIALIZATION
    // ---------------------------------------------------------

    fn initialize_metadata(&mut self) -> EnvResult<()> {
        let root_existing = self.run_async(self.metadata_store.load_node_metadata("/"))?;

        if root_existing.is_none() {
            let root = VirtualPath::root();
            let root_node =
                FileNode::new_directory(root, ROOT_USER_ID, DEFAULT_DIRECTORY_PERMISSIONS);
            self.persist_node(&root_node)?;
        }

        let root_user = self
            .users
            .get_user(ROOT_USER_ID)
            .ok_or_else(|| EnvError::InvalidEnvironment("Root user missing.".to_string()))?
            .clone();

        self.run_async(self.metadata_store.save_user(&root_user))?;

        let default_user = self
            .users
            .get_user(DEFAULT_USER_ID)
            .ok_or_else(|| EnvError::InvalidEnvironment("Default user missing.".to_string()))?
            .clone();

        self.run_async(self.metadata_store.save_user(&default_user))?;

        Ok(())
    }

    fn persist_node(&self, node: &FileNode) -> EnvResult<()> {
        self.run_async(self.metadata_store.save_node(node))
    }

    fn load_metadata_from_database(&mut self) -> EnvResult<()> {
        let nodes = self.run_async(self.metadata_store.load_all_nodes())?;

        for (path, node_type, metadata) in nodes {
            let node = FileNode {
                path: path.clone(),
                node_type,
                metadata,
            };

            self.nodes.insert(path.to_string(), node);
        }

        Ok(())
    }

    // ---------------------------------------------------------
    // STORAGE CONSISTENCY
    // ---------------------------------------------------------

    pub fn verify_storage_consistency(&self) -> EnvResult<()> {
        for node in self.nodes.values() {
            let physical = self.backend.resolve(&node.path)?;

            match node.node_type {
                NodeType::File => {
                    if !physical.exists() {
                        return Err(EnvError::NotFound(format!(
                            "Metadata exists but physical file is missing: {}",
                            node.path
                        )));
                    }

                    if !physical.is_file() {
                        return Err(EnvError::InvalidEnvironment(format!(
                            "Expected file but found non-file storage object: {}",
                            node.path
                        )));
                    }
                }

                NodeType::Directory => {
                    if !physical.exists() {
                        return Err(EnvError::NotFound(format!(
                            "Metadata exists but physical directory is missing: {}",
                            node.path
                        )));
                    }

                    if !physical.is_dir() {
                        return Err(EnvError::InvalidEnvironment(format!(
                            "Expected directory but found non-directory storage object: {}",
                            node.path
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    // ---------------------------------------------------------
    // PATH LOCK
    // ---------------------------------------------------------

    fn with_path_lock<T, F>(&mut self, path: &str, operation: F) -> EnvResult<T>
    where
        F: FnOnce(&mut Self) -> EnvResult<T>,
    {
        // IMPORTANT:
        // acquire and release now happen on the SAME persistent
        // PostgreSQL connection/runtime lifecycle.

        self.run_async(self.metadata_store.acquire_path_lock(path))?;

        let result = operation(self);

        let unlock_result = self.run_async(self.metadata_store.release_path_lock(path));

        match (result, unlock_result) {
            (Ok(value), Ok(())) => Ok(value),

            (Err(error), Ok(())) => Err(error),

            (Ok(_), Err(error)) => Err(error),

            (Err(operation_error), Err(unlock_error)) => {
                Err(EnvError::InvalidEnvironment(format!(
                    "Filesystem operation failed: {:?}; \
                     additionally failed to release lock: {:?}",
                    operation_error, unlock_error
                )))
            }
        }
    }

    // ---------------------------------------------------------
    // DIRECTORY CREATION
    // ---------------------------------------------------------

    pub fn create_directory(&mut self, path: &VirtualPath) -> EnvResult<()> {
        let path_string = path.to_string();

        if path.is_root() {
            return Err(EnvError::AlreadyExists("/".to_string()));
        }

        if self.exists(path) {
            return Err(EnvError::AlreadyExists(path_string));
        }

        self.ensure_parent_directory(path)?;

        let parent = path
            .parent()
            .ok_or_else(|| EnvError::InvalidPath("Directory must have a parent.".to_string()))?;

        let parent_node = self
            .node(&parent)
            .ok_or_else(|| EnvError::NotFound(parent.to_string()))?;

        check_permission(&parent_node.metadata, self.current_user, Permission::Write)?;

        check_permission(
            &parent_node.metadata,
            self.current_user,
            Permission::Execute,
        )?;

        self.with_path_lock(&path_string, |filesystem| {
            if filesystem.exists(path) {
                return Err(EnvError::AlreadyExists(path_string.clone()));
            }

            filesystem.backend.create_directory(path)?;

            let node = FileNode::new_directory(
                path.clone(),
                filesystem.current_user,
                DEFAULT_DIRECTORY_PERMISSIONS,
            );

            filesystem.persist_node(&node)?;

            filesystem.nodes.insert(path_string.clone(), node);

            Ok(())
        })
    }

    // ---------------------------------------------------------
    // FILE CREATION
    // ---------------------------------------------------------

    pub fn create_file(&mut self, path: &VirtualPath) -> EnvResult<()> {
        let path_string = path.to_string();

        if path.is_root() {
            return Err(EnvError::InvalidPath("Root cannot be a file.".to_string()));
        }

        if self.exists(path) {
            return Err(EnvError::AlreadyExists(path_string));
        }

        self.ensure_parent_directory(path)?;

        let parent = path
            .parent()
            .ok_or_else(|| EnvError::InvalidPath("File must have a parent.".to_string()))?;

        let parent_node = self
            .node(&parent)
            .ok_or_else(|| EnvError::NotFound(parent.to_string()))?;

        check_permission(&parent_node.metadata, self.current_user, Permission::Write)?;

        check_permission(
            &parent_node.metadata,
            self.current_user,
            Permission::Execute,
        )?;

        self.with_path_lock(&path_string, |filesystem| {
            if filesystem.exists(path) {
                return Err(EnvError::AlreadyExists(path_string.clone()));
            }

            filesystem.backend.create_file(path)?;

            let node = FileNode::new_file(
                path.clone(),
                filesystem.current_user,
                DEFAULT_FILE_PERMISSIONS,
            );

            filesystem.persist_node(&node)?;

            filesystem.nodes.insert(path_string.clone(), node);

            Ok(())
        })
    }

    // ---------------------------------------------------------
    // READ
    // ---------------------------------------------------------

    pub fn read_file(&mut self, path: &VirtualPath) -> EnvResult<Vec<u8>> {
        let node = self
            .node(path)
            .ok_or_else(|| EnvError::NotFound(path.to_string()))?;

        if !node.is_file() {
            return Err(EnvError::IsDirectory(path.to_string()));
        }

        check_permission(&node.metadata, self.current_user, Permission::Read)?;

        let bytes = self.backend.read_file(path)?;

        if let Some(node) = self.nodes.get_mut(&path.to_string()) {
            node.metadata.access();
        }

        Ok(bytes)
    }

    pub fn read_text(&mut self, path: &VirtualPath) -> EnvResult<String> {
        let data = self.read_file(path)?;

        String::from_utf8(data).map_err(|error| {
            EnvError::InvalidEnvironment(format!("File is not valid UTF-8: {}", error))
        })
    }

    // ---------------------------------------------------------
    // WRITE
    // ---------------------------------------------------------

    pub fn write_file(&mut self, path: &VirtualPath, data: &[u8]) -> EnvResult<()> {
        let path_string = path.to_string();

        let node = self
            .node(path)
            .ok_or_else(|| EnvError::NotFound(path_string.clone()))?
            .clone();

        if !node.is_file() {
            return Err(EnvError::IsDirectory(path_string));
        }

        check_permission(&node.metadata, self.current_user, Permission::Write)?;

        self.with_path_lock(&path_string, |filesystem| {
            let current_node = filesystem
                .node(path)
                .ok_or_else(|| EnvError::NotFound(path_string.clone()))?
                .clone();

            if !current_node.is_file() {
                return Err(EnvError::IsDirectory(path_string.clone()));
            }

            filesystem.backend.write_file(path, data)?;

            let mut updated_node = current_node;

            updated_node.metadata.update_size(data.len() as u64);

            filesystem.persist_node(&updated_node)?;

            filesystem.nodes.insert(path_string.clone(), updated_node);

            Ok(())
        })
    }

    pub fn write_text(&mut self, path: &VirtualPath, text: &str) -> EnvResult<()> {
        self.write_file(path, text.as_bytes())
    }

    // ---------------------------------------------------------
    // LIST DIRECTORY
    // ---------------------------------------------------------

    pub fn list_directory(&mut self, path: &VirtualPath) -> EnvResult<Vec<String>> {
        let node = self
            .node(path)
            .ok_or_else(|| EnvError::NotFound(path.to_string()))?;

        if !node.is_directory() {
            return Err(EnvError::NotDirectory(path.to_string()));
        }

        check_permission(&node.metadata, self.current_user, Permission::Read)?;

        check_permission(&node.metadata, self.current_user, Permission::Execute)?;

        self.backend.list_directory(path)
    }


    pub fn change_owner(
        &mut self,
        path: &VirtualPath,
        new_owner_id: u32,
    ) -> EnvResult<()> {
        let key = path.to_string();

        if self.current_user != ROOT_USER_ID {
            return Err(EnvError::PermissionDenied(
                "Only root can change file ownership.".to_string(),
            ));
        }

        if !self.users.contains(new_owner_id) {
            return Err(EnvError::NotFound(format!(
                "User {} does not exist.",
                new_owner_id
            )));
        }

        if !self.exists(path) {
            return Err(EnvError::NotFound(key.clone()));
        }

        self.with_path_lock(&key, |filesystem| {
            let mut node = filesystem
                .nodes
                .get(&key)
                .ok_or_else(|| EnvError::NotFound(key.clone()))?
                .clone();

            node.metadata.owner_id = new_owner_id;
            node.metadata.touch();

            filesystem.persist_node(&node)?;

            filesystem.nodes.insert(key.clone(), node);

            Ok(())
        })
    }

    // ---------------------------------------------------------
    // REMOVE FILE
    // ---------------------------------------------------------

    pub fn remove_file(&mut self, path: &VirtualPath) -> EnvResult<()> {
        let path_string = path.to_string();

        let node = self
            .node(path)
            .ok_or_else(|| EnvError::NotFound(path_string.clone()))?
            .clone();

        if !node.is_file() {
            return Err(EnvError::IsDirectory(path_string));
        }

        let parent = path
            .parent()
            .ok_or_else(|| EnvError::InvalidPath("File must have a parent.".to_string()))?;

        let parent_node = self
            .node(&parent)
            .ok_or_else(|| EnvError::NotFound(parent.to_string()))?;

        check_permission(&parent_node.metadata, self.current_user, Permission::Write)?;
        check_permission(
            &parent_node.metadata,
            self.current_user,
            Permission::Execute,
        )?;

        self.with_path_lock(&path_string, |filesystem| {
            if !filesystem.exists(path) {
                return Err(EnvError::NotFound(path_string.clone()));
            }

            filesystem.backend.remove_file(path)?;

            filesystem.run_async(filesystem.metadata_store.delete_node(&path_string))?;

            filesystem.nodes.remove(&path_string);

            Ok(())
        })
    }

    // ---------------------------------------------------------
    // REMOVE DIRECTORY
    // ---------------------------------------------------------

    pub fn remove_directory(&mut self, path: &VirtualPath) -> EnvResult<()> {
        let path_string = path.to_string();

        if path.is_root() {
            return Err(EnvError::PermissionDenied(
                "Root directory cannot be removed.".to_string(),
            ));
        }

        let node = self
            .node(path)
            .ok_or_else(|| EnvError::NotFound(path_string.clone()))?
            .clone();

        if !node.is_directory() {
            return Err(EnvError::NotDirectory(path_string));
        }

        let parent = path
            .parent()
            .ok_or_else(|| EnvError::InvalidPath("Directory must have a parent.".to_string()))?;

        let parent_node = self
            .node(&parent)
            .ok_or_else(|| EnvError::NotFound(parent.to_string()))?;

        check_permission(&parent_node.metadata, self.current_user, Permission::Write)?;
        check_permission(
            &parent_node.metadata,
            self.current_user,
            Permission::Execute,
        )?;

        let children = self.backend.list_directory(path)?;

        if !children.is_empty() {
            return Err(EnvError::InvalidEnvironment(format!(
                "Directory is not empty: {}",
                path
            )));
        }

        self.with_path_lock(&path_string, |filesystem| {
            if !filesystem.exists(path) {
                return Err(EnvError::NotFound(path_string.clone()));
            }

            let children = filesystem.backend.list_directory(path)?;

            if !children.is_empty() {
                return Err(EnvError::InvalidEnvironment(format!(
                    "Directory is not empty: {}",
                    path
                )));
            }

            filesystem.backend.remove_directory(path)?;

            filesystem.run_async(filesystem.metadata_store.delete_node(&path_string))?;

            filesystem.nodes.remove(&path_string);

            Ok(())
        })
    }

    // ---------------------------------------------------------
    // HELPERS
    // ---------------------------------------------------------

    fn ensure_parent_directory(&self, path: &VirtualPath) -> EnvResult<()> {
        let parent = path
            .parent()
            .ok_or_else(|| EnvError::InvalidPath("Path has no parent.".to_string()))?;

        if !self.exists(&parent) {
            return Err(EnvError::NotFound(format!(
                "Parent directory does not exist: {}",
                parent
            )));
        }

        let parent_node = self
            .node(&parent)
            .ok_or_else(|| EnvError::NotFound(parent.to_string()))?;

        if !parent_node.is_directory() {
            return Err(EnvError::NotDirectory(parent.to_string()));
        }

        Ok(())
    }

    pub fn current_user(&self) -> Option<&User> {
        self.users.get_user(self.current_user)
    }

    pub fn set_current_user(&mut self, user_id: u32) -> EnvResult<()> {
        if !self.users.contains(user_id) {
            return Err(EnvError::NotFound(format!(
                "User {} does not exist.",
                user_id
            )));
        }

        self.current_user = user_id;

        Ok(())
    }

    pub fn exists(&self, path: &VirtualPath) -> bool {
        self.nodes.contains_key(&path.to_string())
    }

    pub fn node(&self, path: &VirtualPath) -> Option<&FileNode> {
        self.nodes.get(&path.to_string())
    }

    pub fn metadata(&self, path: &VirtualPath) -> Option<&FileMetadata> {
        self.node(path).map(|node| &node.metadata)
    }



    pub fn adopt_directory(
    &mut self,
    path: &VirtualPath,
    owner_id: u32,
    permissions: u16,
) -> EnvResult<()> {
    if self.exists(path) {
        return Ok(());
    }

    let physical = self.backend.resolve(path)?;

    if !physical.exists() {
        return self.create_directory(path);
    }

    if !physical.is_dir() {
        return Err(EnvError::AlreadyExists(path.to_string()));
    }

    let node = FileNode::new_directory(
        path.clone(),
        owner_id,
        permissions,
    );

    self.persist_node(&node)?;
    self.nodes.insert(path.to_string(), node);

    Ok(())
}


}
