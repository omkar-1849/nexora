mod gui;

use gui::EnvironmentApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "AI-Native Computing Environment",
        options,
        Box::new(|_cc| Ok(Box::new(EnvironmentApp::new()))),
    )
}
