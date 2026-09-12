use std::process::Command;

fn main() {
    // We want cargo run --bin explorer to automatically build the frontend
    // if we are actually building the app.
    
    // Check if we are building the app, not just checking or running RLS
    if std::env::var("PROFILE").is_ok() {
        println!("cargo:warning=Building React frontend via npm run build...");
        let status = Command::new("cmd")
            .args(&["/C", "npm", "run", "build"])
            .current_dir("..")
            .status();
            
        if let Ok(status) = status {
            if !status.success() {
                println!("cargo:warning=npm run build failed!");
            }
        } else {
            println!("cargo:warning=Failed to execute npm run build.");
        }
    }
    
    tauri_build::build()
}
