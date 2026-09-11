use crate::config::EnvironmentConfig;
use crate::error::{EnvError, EnvResult};
use crate::filesystem::{FileSystem, VirtualPath, DEFAULT_USER_ID, ROOT_USER_ID};
use crate::process::ProcessManager;
use crate::shell::ShellConfig;
use crate::terminal::Terminal;

pub struct Runtime {
    config: EnvironmentConfig,
    process_manager: ProcessManager,
    filesystem: FileSystem,
    terminal: Option<Terminal>,
    running: bool,
}

impl Runtime {
    pub fn new(config: EnvironmentConfig) -> Self {
        let filesystem = FileSystem::new(config.environment_root.clone(), &config.postgres_url)
            .expect("Failed to initialize environment filesystem");

        Self {
            config,
            process_manager: ProcessManager::new(),
            filesystem,
            terminal: None,
            running: false,
        }
    }

    pub fn start(&mut self) -> EnvResult<()> {
        if self.running {
            return Ok(());
        }

        self.initialize_filesystem()?;

        self.running = true;

        println!("Runtime started.");
        println!("Environment filesystem initialized.");

        Ok(())
    }

    fn initialize_filesystem(&mut self) -> EnvResult<()> {
        let root = VirtualPath::root();

        let previous_user = self
            .filesystem
            .current_user()
            .map(|u| u.id)
            .unwrap_or(DEFAULT_USER_ID);
        self.filesystem.set_current_user(ROOT_USER_ID)?;

        let directories = ["home", "workspace", "projects", "data", "models", "tmp"];

        for directory in directories {
            let path = root.join(directory)?;

            if !self.filesystem.exists(&path) {
                self.filesystem.create_directory(&path)?;
            }
        }

        let workspace = root.join("workspace")?;
        if self.filesystem.exists(&workspace) {
            self.filesystem.change_owner(&workspace, DEFAULT_USER_ID)?;
        }

        let home = root.join("home")?;
        if self.filesystem.exists(&home) {
            self.filesystem.change_owner(&home, DEFAULT_USER_ID)?;
        }

        self.filesystem.set_current_user(previous_user)?;

        Ok(())
    }

    pub fn filesystem(&self) -> &FileSystem {
        &self.filesystem
    }

    pub fn filesystem_mut(&mut self) -> &mut FileSystem {
        &mut self.filesystem
    }

    // ---------------------------------------------------------
    // OLD TERMINAL LAYER
    // ---------------------------------------------------------
    //
    // This remains temporarily so the existing GUI compiles.
    // We are NOT extending the CMD architecture.
    //

    pub fn start_terminal(&mut self) -> EnvResult<u32> {
        if !self.running {
            return Err(EnvError::InvalidEnvironment(
                "Runtime is not running.".to_string(),
            ));
        }

        let shell_config = ShellConfig::cmd(self.config.workspace_root.clone());

        let terminal = Terminal::start(&mut self.process_manager, &shell_config)?;

        let pid = terminal.pid();

        self.terminal = Some(terminal);

        println!("Terminal backend started.");
        println!("Terminal PID: {}", pid);

        Ok(pid)
    }

    pub fn execute_terminal_command(&mut self, command: &str) -> EnvResult<()> {
        if let Some(terminal) = &self.terminal {
            terminal.execute(&mut self.process_manager, command)?;

            Ok(())
        } else {
            Err(EnvError::InvalidEnvironment(
                "Terminal is not running.".to_string(),
            ))
        }
    }

    pub fn read_terminal_output(&self) -> Vec<String> {
        if let Some(terminal) = &self.terminal {
            terminal.read_output()
        } else {
            Vec::new()
        }
    }

    pub fn stop(&mut self) {
        if !self.running {
            return;
        }

        self.process_manager.stop_all();

        self.terminal = None;
        self.running = false;

        println!("Runtime stopped.");
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
