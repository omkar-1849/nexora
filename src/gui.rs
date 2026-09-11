use eframe::egui;

use ai_native_env::config::EnvironmentConfig;
use ai_native_env::runtime::Runtime;
pub struct EnvironmentApp {
    runtime: Runtime,
    command: String,
    output: Vec<String>,
    terminal_started: bool,
}

impl EnvironmentApp {
    pub fn new() -> Self {
        let config = EnvironmentConfig::new();
        let mut runtime = Runtime::new(config);

        let mut output = Vec::new();
        let mut terminal_started = false;

        match runtime.start() {
            Ok(_) => {
                output.push("Environment started.".to_string());

                match runtime.start_terminal() {
                    Ok(pid) => {
                        output.push(format!("Terminal started. PID: {}", pid));
                        terminal_started = true;
                    }
                    Err(error) => {
                        output.push(format!("Terminal error: {:?}", error));
                    }
                }
            }
            Err(error) => {
                output.push(format!("Runtime error: {:?}", error));
            }
        }

        Self {
            runtime,
            command: String::new(),
            output,
            terminal_started,
        }
    }
}

impl eframe::App for EnvironmentApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.terminal_started {
            let new_output = self.runtime.read_terminal_output();
            if !new_output.is_empty() {
                self.output.extend(new_output);
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
        }

        ui.heading("AI-Native Computing Environment");

        ui.separator();

        ui.label("Terminal");

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for line in &self.output {
                    ui.monospace(line);
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut self.command);

            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            let execute_clicked = ui.button("Run").clicked();

            if self.terminal_started && (execute_clicked || enter_pressed) {
                let command = self.command.trim().to_string();

                if !command.is_empty() {
                    self.output.push(format!("> {}", command));

                    match self.runtime.execute_terminal_command(&command) {
                        Ok(_) => {}
                        Err(error) => {
                            self.output.push(format!("ERROR: {:?}", error));
                        }
                    }

                    let new_output = self.runtime.read_terminal_output();
                    self.output.extend(new_output);

                    self.command.clear();
                }
            }
        });
    }
}
