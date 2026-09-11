use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    pub environment_root: PathBuf,
    pub workspace_root: PathBuf,
    pub postgres_url: String,
}

impl EnvironmentConfig {
    pub fn new() -> Self {
        let base = dirs_fallback();

        Self {
            environment_root: base.join("AI-Native-Environment"),
            workspace_root: base.join("AI-Native-Environment").join("workspace"),

            postgres_url: std::env::var("AI_NATIVE_DATABASE_URL")
                .expect("AI_NATIVE_DATABASE_URL environment variable must be set"),
        }
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self::new()
    }
}


fn dirs_fallback() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
