use crate::error::EnvResult;
use crate::process::ProcessManager;
use crate::shell::ShellConfig;
use crate::shell_session::ShellSession;

pub struct Terminal {
    session: ShellSession,
}

impl Terminal {
    pub fn start(
        process_manager: &mut ProcessManager,
        shell_config: &ShellConfig,
    ) -> EnvResult<Self> {
        let session = ShellSession::start(process_manager, shell_config)?;

        Ok(Self { session })
    }

    pub fn pid(&self) -> u32 {
        self.session.pid()
    }

    pub fn execute(&self, process_manager: &mut ProcessManager, command: &str) -> EnvResult<()> {
        self.session.send_command(process_manager, command)
    }

    pub fn read_output(&self) -> Vec<String> {
        let mut output = Vec::new();

        while let Some(line) = self.session.try_read_output() {
            output.push(line);
        }

        output
    }
}
