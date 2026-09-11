use std::time::SystemTime;

use tokio_postgres::{Client, NoTls};

use crate::error::{EnvError, EnvResult};
use crate::filesystem::{FileMetadata, FileNode, NodeType, User, VirtualPath};

pub struct MetadataStore {
    client: Client,
}

impl MetadataStore {
    pub async fn connect(connection_string: &str) -> EnvResult<Self> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!("PostgreSQL connection failed: {}", error))
            })?;

        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("PostgreSQL connection error: {}", error);
            }
        });

        let store = Self { client };

        store.initialize_schema().await?;

        Ok(store)
    }

    async fn initialize_schema(&self) -> EnvResult<()> {
        self.client
            .batch_execute(
                r#"
                CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY,
                    username TEXT NOT NULL UNIQUE
                );

                CREATE TABLE IF NOT EXISTS filesystem_nodes (
                    path TEXT PRIMARY KEY,
                    node_type TEXT NOT NULL,
                    owner_id INTEGER NOT NULL,
                    permissions INTEGER NOT NULL,
                    size BIGINT NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ NOT NULL,
                    modified_at TIMESTAMPTZ NOT NULL,
                    accessed_at TIMESTAMPTZ NOT NULL
                );
                "#,
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!(
                    "Failed to initialize metadata schema: {}",
                    error
                ))
            })?;

        Ok(())
    }

    pub async fn save_user(&self, user: &User) -> EnvResult<()> {
        self.client
            .execute(
                r#"
                INSERT INTO users (id, username)
                VALUES ($1, $2)
                ON CONFLICT (id)
                DO UPDATE SET username = EXCLUDED.username
                "#,
                &[&(user.id as i32), &user.username],
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!("Failed to save user: {}", error))
            })?;

        Ok(())
    }

    /*
     * IMPORTANT:
     *
     * These are SESSION-level PostgreSQL advisory locks.
     *
     * pg_advisory_lock() remains held on this PostgreSQL connection
     * until pg_advisory_unlock() is explicitly called.
     *
     * This allows FileSystem to:
     *
     *   acquire lock
     *       ↓
     *   physical filesystem operation
     *       ↓
     *   metadata operation
     *       ↓
     *   release lock
     *
     * Other processes using the same lock key must wait.
     */

    pub async fn acquire_path_lock(&self, path: &str) -> EnvResult<()> {
        self.client
            .execute("SELECT pg_advisory_lock(hashtextextended($1, 0))", &[&path])
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!(
                    "Failed to acquire filesystem lock for '{}': {}",
                    path, error
                ))
            })?;

        Ok(())
    }

    pub async fn release_path_lock(&self, path: &str) -> EnvResult<()> {
        let row = self
            .client
            .query_one(
                "SELECT pg_advisory_unlock(hashtextextended($1, 0))",
                &[&path],
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!(
                    "Failed to release filesystem lock for '{}': {}",
                    path, error
                ))
            })?;

        let released: bool = row.get(0);

        if !released {
            return Err(EnvError::InvalidEnvironment(format!(
                "Filesystem lock for '{}' was not held by this connection.",
                path
            )));
        }

        Ok(())
    }

    pub async fn save_node(&self, node: &FileNode) -> EnvResult<()> {
        let node_type = match node.node_type {
            NodeType::File => "file",
            NodeType::Directory => "directory",
        };

        self.client
            .execute(
                r#"
                INSERT INTO filesystem_nodes (
                    path,
                    node_type,
                    owner_id,
                    permissions,
                    size,
                    created_at,
                    modified_at,
                    accessed_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (path)
                DO UPDATE SET
                    node_type = EXCLUDED.node_type,
                    owner_id = EXCLUDED.owner_id,
                    permissions = EXCLUDED.permissions,
                    size = EXCLUDED.size,
                    modified_at = EXCLUDED.modified_at,
                    accessed_at = EXCLUDED.accessed_at
                "#,
                &[
                    &node.path.to_string(),
                    &node_type,
                    &(node.metadata.owner_id as i32),
                    &(node.metadata.permissions as i32),
                    &(node.metadata.size as i64),
                    &node.metadata.created_at,
                    &node.metadata.modified_at,
                    &node.metadata.accessed_at,
                ],
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!("Failed to save filesystem node: {}", error))
            })?;

        Ok(())
    }

    pub async fn delete_node(&self, path: &str) -> EnvResult<()> {
        self.client
            .execute("DELETE FROM filesystem_nodes WHERE path = $1", &[&path])
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!("Failed to delete filesystem node: {}", error))
            })?;

        Ok(())
    }

    pub async fn delete_node_and_children(&self, path: &str) -> EnvResult<()> {
        let pattern = format!("{}/%", path);
        self.client
            .execute(
                "DELETE FROM filesystem_nodes WHERE path = $1 OR path LIKE $2",
                &[&path, &pattern],
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!(
                    "Failed to delete filesystem nodes under '{}': {}",
                    path, error
                ))
            })?;

        Ok(())
    }

    pub async fn load_node_metadata(&self, path: &str) -> EnvResult<Option<FileMetadata>> {
        let row = self
            .client
            .query_opt(
                r#"
                SELECT
                    owner_id,
                    permissions,
                    size,
                    created_at,
                    modified_at,
                    accessed_at
                FROM filesystem_nodes
                WHERE path = $1
                "#,
                &[&path],
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!(
                    "Failed to load filesystem metadata: {}",
                    error
                ))
            })?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(FileMetadata {
            owner_id: row.get::<_, i32>(0) as u32,
            permissions: row.get::<_, i32>(1) as u16,
            size: row.get::<_, i64>(2) as u64,
            created_at: row.get::<_, SystemTime>(3),
            modified_at: row.get::<_, SystemTime>(4),
            accessed_at: row.get::<_, SystemTime>(5),
        }))
    }

    pub async fn load_all_nodes(&self) -> EnvResult<Vec<(VirtualPath, NodeType, FileMetadata)>> {
        let rows = self
            .client
            .query(
                r#"
                SELECT
                    path,
                    node_type,
                    owner_id,
                    permissions,
                    size,
                    created_at,
                    modified_at,
                    accessed_at
                FROM filesystem_nodes
                ORDER BY path
                "#,
                &[],
            )
            .await
            .map_err(|error| {
                EnvError::InvalidEnvironment(format!("Failed to load filesystem nodes: {}", error))
            })?;

        let mut nodes = Vec::new();

        for row in rows {
            let path_string: String = row.get(0);

            let path = VirtualPath::parse(&path_string)?;

            let node_type_string: String = row.get(1);

            let node_type = match node_type_string.as_str() {
                "file" => NodeType::File,
                "directory" => NodeType::Directory,
                _ => {
                    return Err(EnvError::InvalidEnvironment(format!(
                        "Unknown node type '{}' for '{}'.",
                        node_type_string, path_string
                    )));
                }
            };

            let metadata = FileMetadata {
                owner_id: row.get::<_, i32>(2) as u32,
                permissions: row.get::<_, i32>(3) as u16,
                size: row.get::<_, i64>(4) as u64,
                created_at: row.get::<_, SystemTime>(5),
                modified_at: row.get::<_, SystemTime>(6),
                accessed_at: row.get::<_, SystemTime>(7),
            };

            nodes.push((path, node_type, metadata));
        }

        Ok(nodes)
    }
}
