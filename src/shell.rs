use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ShellType {
    Cmd,
    PowerShell,
}

#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub shell_type: ShellType,
    pub working_directory: PathBuf,
}

impl ShellConfig {
    pub fn cmd(working_directory: PathBuf) -> Self {
        Self {
            shell_type: ShellType::Cmd,
            working_directory,
        }
    }

    pub fn program(&self) -> &str {
        match self.shell_type {
            ShellType::Cmd => "cmd.exe",
            ShellType::PowerShell => "powershell.exe",
        }
    }

    pub fn arguments(&self) -> &[&str] {
        match self.shell_type {
            ShellType::Cmd => &[],
            ShellType::PowerShell => &["-NoLogo"],
        }
    }
}
