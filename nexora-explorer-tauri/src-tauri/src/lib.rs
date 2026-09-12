pub mod dto;
pub mod commands;

use commands::fs_commands::{list_directory, get_metadata, AppState};
use ai_native_env::config::EnvironmentConfig;
use ai_native_env::runtime::Runtime;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = EnvironmentConfig::new();
    let mut runtime = Runtime::new(config);
    runtime.start().expect("Failed to initialize environment runtime");

    tauri::Builder::default()
        .manage(AppState {
            runtime: Mutex::new(runtime),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_directory,
            get_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
