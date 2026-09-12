use ai_native_env::config::EnvironmentConfig;
use ai_native_env::explorer::ExplorerView;
use ai_native_env::runtime::Runtime;
use eframe::egui;

struct ExplorerApp {
    runtime: Runtime,
    explorer: ExplorerView,
}

impl ExplorerApp {
    fn new() -> Self {
        let config = EnvironmentConfig::new();
        let mut runtime = Runtime::new(config);

        if let Err(error) = runtime.start() {
            eprintln!("Runtime error: {:?}", error);
        }

        Self {
            runtime,
            explorer: ExplorerView::new(),
        }
    }
}

impl eframe::App for ExplorerApp {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        self.explorer
            .render(ui, self.runtime.filesystem_mut());
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 680.0])
            .with_min_inner_size([760.0, 480.0])
            .with_title("NEXORA Core Explorer"),
        ..Default::default()
    };

    eframe::run_native(
        "NEXORA Core Explorer",
        options,
        Box::new(|_cc| Ok(Box::new(ExplorerApp::new()))),
    )
}