use crate::filesystem::metadata::FileMetadata;
use crate::filesystem::VirtualPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: VirtualPath,
    pub node_type: NodeType,
    pub metadata: FileMetadata,
}

impl FileNode {
    pub fn new_file(path: VirtualPath, owner_id: u32, permissions: u16) -> Self {
        Self {
            path,
            node_type: NodeType::File,
            metadata: FileMetadata::new(owner_id, permissions),
        }
    }

    pub fn new_directory(path: VirtualPath, owner_id: u32, permissions: u16) -> Self {
        Self {
            path,
            node_type: NodeType::Directory,
            metadata: FileMetadata::new(owner_id, permissions),
        }
    }

    pub fn is_file(&self) -> bool {
        self.node_type == NodeType::File
    }

    pub fn is_directory(&self) -> bool {
        self.node_type == NodeType::Directory
    }
}
