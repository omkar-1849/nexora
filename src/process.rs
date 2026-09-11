use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};

use crate::error::{EnvError, EnvResult};

pub struct ProcessOutput {
    pub receiver: Receiver<String>,
}

pub struct ProcessManager {
    processes: HashMap<u32, Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn start_process(
        &mut self,
        program: &str,
        args: &[&str],
        working_directory: &Path,
    ) -> EnvResult<(u32, ProcessOutput)> {
        let mut child = Command::new(program)
            .args(args)
            .current_dir(working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();

        let stdout = child.stdout.take().expect("stdout was not captured");

        let stderr = child.stderr.take().expect("stderr was not captured");

        let (sender, receiver) = mpsc::channel();

        let stdout_sender = sender.clone();

        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let _ = stdout_sender.send(line);
                    }
                    Err(_) => break,
                }
            }
        });

        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);

            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let _ = sender.send(format!("[stderr] {}", line));
                    }
                    Err(_) => break,
                }
            }
        });

        self.processes.insert(pid, child);

        println!("Process started: {}", program);
        println!("PID: {}", pid);
        println!("Working directory: {}", working_directory.display());

        Ok((pid, ProcessOutput { receiver }))
    }

    pub fn reap_exited(&mut self) {
        self.processes.retain(|_pid, child| {
            match child.try_wait() {
                Ok(Some(_status)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        });
    }

    pub fn send_input(&mut self, pid: u32, input: &str) -> EnvResult<()> {
        let child = self.processes.get_mut(&pid).ok_or_else(|| {
            EnvError::NotFound(format!("Process with PID {} not found", pid))
        })?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(input.as_bytes())?;
            stdin.flush()?;
        }

        Ok(())
    }

    pub fn stop_process(&mut self, pid: u32) -> EnvResult<()> {
        if let Some(mut child) = self.processes.remove(&pid) {
            let _ = child.kill();
            let _ = child.wait();

            println!("Process stopped: PID {}", pid);
            Ok(())
        } else {
            Err(EnvError::NotFound(format!("Process with PID {} not found", pid)))
        }
    }

    pub fn stop_all(&mut self) {
        let mut processes = std::mem::take(&mut self.processes);

        for (pid, child) in processes.iter_mut() {
            let _ = child.kill();
            let _ = child.wait();

            println!("Process stopped: PID {}", pid);
        }
    }
}
