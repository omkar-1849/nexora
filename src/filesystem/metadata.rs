use std::time::SystemTime;

/// Metadata belonging to an environment filesystem node.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Size of the file in bytes.
    ///
    /// Directories normally have size 0 at the environment layer.
    pub size: u64,

    /// Creation time.
    pub created_at: SystemTime,

    /// Last modification time.
    pub modified_at: SystemTime,

    /// Last access time.
    pub accessed_at: SystemTime,

    /// User ID of the owner.
    pub owner_id: u32,

    /// Permission bits.
    pub permissions: u16,
}

impl FileMetadata {
    pub fn new(owner_id: u32, permissions: u16) -> Self {
        let now = SystemTime::now();

        Self {
            size: 0,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            owner_id,
            permissions,
        }
    }

    pub fn update_size(&mut self, size: u64) {
        self.size = size;
        self.modified_at = SystemTime::now();
    }

    pub fn touch(&mut self) {
        self.modified_at = SystemTime::now();
    }

    pub fn access(&mut self) {
        self.accessed_at = SystemTime::now();
    }
}
