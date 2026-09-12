use std::path::PathBuf;

use ai_native_env::filesystem::{FileSystem, VirtualPath, DEFAULT_USER_ID, ROOT_USER_ID};

fn test_storage_root() -> PathBuf {
    std::env::temp_dir().join("ai_native_env_persistence_test")
}

fn database_url() -> String {
    std::env::var("AI_NATIVE_TEST_DATABASE_URL")
        .unwrap_or_else(|_| "host=localhost user=postgres password=postgres dbname=ai_native_env_test".to_string())
}

fn cleanup_test_metadata(database_url: &str, paths: &[&str]) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime for test cleanup");
    rt.block_on(async {
        if let Ok((client, connection)) =
            tokio_postgres::connect(database_url, tokio_postgres::NoTls).await
        {
            tokio::spawn(async move {
                let _ = connection.await;
            });

            for path in paths {
                let pattern = format!("{}/%", path);
                let _ = client
                    .execute(
                        "DELETE FROM filesystem_nodes WHERE path = $1 OR path LIKE $2",
                        &[path, &pattern],
                    )
                    .await;
            }
        }
    });
}

#[test]
fn filesystem_metadata_persists_across_instances() {
    let storage_root = test_storage_root();

    if storage_root.exists() {
        std::fs::remove_dir_all(&storage_root).unwrap();
    }

    let database_url = database_url();
    cleanup_test_metadata(&database_url, &["/workspace"]);

    // ---------------------------------------------------------
    // INSTANCE #1
    // ---------------------------------------------------------

    {
        let mut filesystem = FileSystem::new(storage_root.clone(), &database_url)
            .expect("Failed to create first filesystem");

        let workspace = VirtualPath::parse("/workspace").unwrap();

        if !filesystem.exists(&workspace) {
            filesystem.set_current_user(ROOT_USER_ID).unwrap();

            filesystem.create_directory(&workspace).unwrap();

            filesystem.change_owner(&workspace, DEFAULT_USER_ID).unwrap();

            filesystem.set_current_user(DEFAULT_USER_ID).unwrap();
        }

        let file = workspace.join("persistence.txt").unwrap();

        filesystem.create_file(&file).unwrap();

        filesystem
            .write_text(&file, "Persistent filesystem test")
            .unwrap();

        assert!(filesystem.exists(&file));

        let content = filesystem.read_text(&file).unwrap();

        assert_eq!(content, "Persistent filesystem test");

        // filesystem is intentionally dropped here.
    }

    // ---------------------------------------------------------
    // INSTANCE #2
    // ---------------------------------------------------------

    {
        let mut filesystem = FileSystem::new(storage_root.clone(), &database_url)
            .expect("Failed to create second filesystem");

        let file = VirtualPath::parse("/workspace/persistence.txt").unwrap();

        // Metadata should have been reconstructed
        // from PostgreSQL.
        assert!(
            filesystem.exists(&file),
            "File should persist across filesystem instances"
        );

        // Physical contents should still be accessible.
        let content = filesystem.read_text(&file).unwrap();

        assert_eq!(content, "Persistent filesystem test");

        // Metadata should also be available.
        let metadata = filesystem.metadata(&file).expect("Metadata should exist");

        assert_eq!(metadata.owner_id, DEFAULT_USER_ID);
        assert_eq!(metadata.size, 26);

        let workspace = VirtualPath::parse("/workspace").unwrap();
        let ws_metadata = filesystem.metadata(&workspace).expect("Workspace metadata should exist");
        assert_eq!(ws_metadata.owner_id, DEFAULT_USER_ID);
    }

    // Cleanup.
    std::fs::remove_dir_all(storage_root).unwrap();
    cleanup_test_metadata(&database_url, &["/workspace"]);
}

