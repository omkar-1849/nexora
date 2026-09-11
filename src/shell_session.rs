use std::sync::mpsc::Receiver;

use crate::error::EnvResult;
use crate::process::{ProcessManager, ProcessOutput};
use crate::shell::ShellConfig;

pub struct ShellSession {
    pid: u32,
    output: Receiver<String>,
}

impl ShellSession {
    pub fn start(process_manager: &mut ProcessManager, config: &ShellConfig) -> EnvResult<Self> {
        let (pid, ProcessOutput { receiver }) = process_manager.start_process(
            config.program(),
            config.arguments(),
            &config.working_directory,
        )?;

        Ok(Self {
            pid,
            output: receiver,
        })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn send_command(
        &self,
        process_manager: &mut ProcessManager,
        command: &str,
    ) -> EnvResult<()> {
        let command = format!("{}\r\n", command);

        process_manager.send_input(self.pid, &command)
    }

    pub fn try_read_output(&self) -> Option<String> {
        self.output.try_recv().ok()
    }
}
