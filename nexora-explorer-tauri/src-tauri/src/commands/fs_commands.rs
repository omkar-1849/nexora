
use tauri::State;
use ai_native_env::filesystem::VirtualPath;
use crate::dto::{DirectoryListingDto, FileNodeDto, FileTypeTag, FsErrorDto};

use ai_native_env::runtime::Runtime;

use tokio::sync::Mutex;

pub struct AppState {
    pub runtime: Mutex<Runtime>,
}

fn map_node_to_dto(node: &ai_native_env::filesystem::FileNode) -> FileNodeDto {
    let name = node.path.file_name().unwrap_or_default().to_string();
    let virtual_path = node.path.to_string();
    
    let kind = if node.is_directory() { "directory".to_string() } else { "file".to_string() };
    
    let file_type = if node.is_directory() {
        None
    } else {
        // very basic mapping
        if name.ends_with(".rs") { Some(FileTypeTag::Rust) }
        else if name.ends_with(".py") { Some(FileTypeTag::Python) }
        else if name.ends_with(".md") { Some(FileTypeTag::Markdown) }
        else { Some(FileTypeTag::Unknown) }
    };
    
    FileNodeDto {
        name,
        virtual_path,
        kind,
        file_type,
        size_bytes: if node.is_directory() { None } else { Some(node.metadata.size) },
        owner: Some(node.metadata.owner_id.to_string()),
        permissions: Some(format!("{:o}", node.metadata.permissions)),
        created_at: None, // need system time formatting
        modified_at: None,
        accessed_at: None,
    }
}

#[tauri::command]
pub async fn list_directory(
    path: String,
    state: State<'_, AppState>,
) -> Result<DirectoryListingDto, FsErrorDto> {
    let mut runtime = state.runtime.lock().await;
    let fs = runtime.filesystem_mut();
    let vpath = VirtualPath::parse(&path).map_err(ai_native_env::error::EnvError::from)?;
    
    let child_paths = fs.list_directory(&vpath)?;
    
    let mut entries = Vec::new();
    for p in child_paths {
        if let Ok(child_vpath) = vpath.join(&p) {
            if let Some(node) = fs.node(&child_vpath) {
                entries.push(map_node_to_dto(&node));
            }
        }
    }
    
    Ok(DirectoryListingDto {
        path: path.clone(),
        entries,
    })
}

#[tauri::command]
pub async fn get_metadata(
    path: String,
    state: State<'_, AppState>,
) -> Result<FileNodeDto, FsErrorDto> {
    let mut runtime = state.runtime.lock().await;
    let fs = runtime.filesystem_mut();
    let vpath = VirtualPath::parse(&path).map_err(ai_native_env::error::EnvError::from)?;
    
    let node = fs.node(&vpath).ok_or_else(|| {
        FsErrorDto {
            kind: "notFound".to_string(),
            message: format!("Path not found: {}", path),
        }
    })?;
    
    Ok(map_node_to_dto(&node))
}
