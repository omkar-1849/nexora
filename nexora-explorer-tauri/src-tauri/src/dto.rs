use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileTypeTag {
    Folder,
    Rust,
    Python,
    Cpp,
    Config,
    Markdown,
    Image,
    Shell,
    Model,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNodeDto {
    pub name: String,
    pub virtual_path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<FileTypeTag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListingDto {
    pub path: String,
    pub entries: Vec<FileNodeDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsErrorDto {
    pub kind: String,
    pub message: String,
}

// Convert EnvError to FsErrorDto
impl From<ai_native_env::error::EnvError> for FsErrorDto {
    fn from(error: ai_native_env::error::EnvError) -> Self {
        let (kind, message) = match &error {
            ai_native_env::error::EnvError::NotFound(_) => ("notFound", error.to_string()),
            ai_native_env::error::EnvError::PermissionDenied(_) => ("permissionDenied", error.to_string()),
            ai_native_env::error::EnvError::InvalidPath(_) => ("invalidPath", error.to_string()),
            ai_native_env::error::EnvError::Io(_) => ("ioError", error.to_string()),
            _ => ("unknown", error.to_string()),
        };
        // Scrub physical Windows paths like C:\ or \\?\C:\
        let mut scrubbed = message.clone();
        if let Some(idx) = scrubbed.find(r"\\?\") {
            scrubbed = scrubbed.replace(&scrubbed[idx..idx+4], "");
        }
        let re = regex::Regex::new(r"(?i)[A-Z]:\\(?:[^/\\]+[/\\])*").unwrap();
        scrubbed = re.replace_all(&scrubbed, "/").to_string();

        Self {
            kind: kind.to_string(),
            message: scrubbed,
        }
    }
}

