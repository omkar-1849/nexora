use std::fs;

use crate::config::EnvironmentConfig;
use crate::error::EnvResult;
use crate::workspace::initialize_workspace;

pub struct Environment {
    pub config: EnvironmentConfig,
    running: bool,
}

impl Environment {
    pub fn new(config: EnvironmentConfig) -> Self {
        Self {
            config,
            running: false,
        }
    }

    pub fn initialize(&self) -> EnvResult<()> {
        fs::create_dir_all(&self.config.environment_root)?;

        initialize_workspace(&self.config.workspace_root)?;

        Ok(())
    }

    pub fn start(&mut self) -> EnvResult<()> {
        if self.running {
            return Ok(());
        }

        self.initialize()?;

        self.running = true;

        println!("AI-Native Environment started.");
        println!("Environment: {}", self.config.environment_root.display());
        println!("Workspace: {}", self.config.workspace_root.display());

        Ok(())
    }

    pub fn stop(&mut self) {
        if !self.running {
            return;
        }

        self.running = false;

        println!("AI-Native Environment stopped.");
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
