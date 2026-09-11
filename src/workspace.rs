use std::fs;
use std::path::Path;

use crate::error::EnvResult;

pub fn initialize_workspace(path: &Path) -> EnvResult<()> {
    fs::create_dir_all(path)?;

    create_directory(path, "projects")?;
    create_directory(path, "temp")?;
    create_directory(path, "data")?;
    create_directory(path, "logs")?;
    create_directory(path, "models")?;

    Ok(())
}

fn create_directory(root: &Path, name: &str) -> EnvResult<()> {
    fs::create_dir_all(root.join(name))?;
    Ok(())
}
