use eframe::egui::{self, Color32, CornerRadius, Stroke};

pub struct Theme;

impl Theme {
    // Canvas & Panels
    pub const BG_CANVAS: Color32 = Color32::from_rgb(10, 10, 10);
    pub const BG_PANEL: Color32 = Color32::from_rgb(17, 17, 17);
    pub const BG_CARD: Color32 = Color32::from_rgb(23, 23, 23);
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(34, 34, 38);
    pub const BG_CARD_SELECTED: Color32 = Color32::from_rgb(40, 42, 50);
    pub const BG_INPUT: Color32 = Color32::from_rgb(15, 15, 18);

    // Borders & Dividers
    pub const BORDER_DARK: Color32 = Color32::from_rgb(40, 40, 40);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 18);
    pub const BORDER_FOCUS: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 90);
    pub const BORDER_SELECTED: Color32 = Color32::from_rgba_premultiplied(229, 169, 60, 120);

    // Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 245, 247);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 165);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(102, 102, 108);

    // Folder Amber / Mustard Yellow Identity
    pub const FOLDER_TAB: Color32 = Color32::from_rgb(212, 148, 40);
    pub const FOLDER_BODY: Color32 = Color32::from_rgb(229, 169, 60);
    pub const FOLDER_FRONT: Color32 = Color32::from_rgb(242, 184, 75);

    // Natural File Type Identities
    pub const FILE_PYTHON: Color32 = Color32::from_rgb(75, 139, 190);
    pub const FILE_RUST: Color32 = Color32::from_rgb(206, 66, 43);
    pub const FILE_C_CPP: Color32 = Color32::from_rgb(101, 154, 210);
    pub const FILE_CONFIG: Color32 = Color32::from_rgb(229, 192, 123);
    pub const FILE_MODEL: Color32 = Color32::from_rgb(152, 195, 121);
    pub const FILE_IMAGE: Color32 = Color32::from_rgb(86, 182, 194);
    pub const FILE_SCRIPT: Color32 = Color32::from_rgb(224, 108, 117);
    pub const FILE_DOC: Color32 = Color32::from_rgb(171, 178, 191);
    pub const FILE_DEFAULT: Color32 = Color32::from_rgb(130, 137, 151);

    // Spacing & Dimensions
    pub const SIDEBAR_WIDTH: f32 = 190.0;
    pub const INSPECTOR_WIDTH: f32 = 270.0;
    pub const TOPBAR_HEIGHT: f32 = 42.0;
    pub const GRID_ITEM_WIDTH: f32 = 112.0;
    pub const GRID_ITEM_HEIGHT: f32 = 118.0;

    // Corner Radius
    pub const RADIUS_CARD: CornerRadius = CornerRadius::same(6);
    pub const RADIUS_PILL: CornerRadius = CornerRadius::same(12);
    pub const RADIUS_INPUT: CornerRadius = CornerRadius::same(5);

    // Apply global egui visuals
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Self::BG_CANVAS;
        visuals.window_fill = Self::BG_PANEL;
        visuals.faint_bg_color = Self::BG_CARD;
        visuals.extreme_bg_color = Self::BG_INPUT;
        
        visuals.widgets.noninteractive.bg_fill = Self::BG_PANEL;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_DARK);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_SUBTLE);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_SECONDARY);

        visuals.widgets.hovered.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::BORDER_FOCUS);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        visuals.widgets.active.bg_fill = Self::BG_CARD_SELECTED;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Self::BORDER_SELECTED);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        visuals.selection.bg_fill = Self::BG_CARD_SELECTED;
        visuals.selection.stroke = Stroke::new(1.0, Self::BORDER_SELECTED);

        ctx.set_visuals(visuals);
    }

    /// Determine file identity based on extension
    pub fn file_type_info(name: &str) -> (&'static str, Color32) {
        let lower = name.to_lowercase();
        if lower.ends_with(".py") {
            ("PY", Self::FILE_PYTHON)
        } else if lower.ends_with(".rs") {
            ("RS", Self::FILE_RUST)
        } else if lower.ends_with(".c") || lower.ends_with(".cpp") || lower.ends_with(".h") || lower.ends_with(".hpp") {
            ("C/C++", Self::FILE_C_CPP)
        } else if lower.ends_with(".json") || lower.ends_with(".toml") || lower.ends_with(".yaml") || lower.ends_with(".yml") || lower.ends_with(".xml") {
            ("CFG", Self::FILE_CONFIG)
        } else if lower.ends_with(".safetensors") || lower.ends_with(".bin") || lower.ends_with(".pt") || lower.ends_with(".onnx") || lower.ends_with(".ckpt") {
            ("ML", Self::FILE_MODEL)
        } else if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".svg") || lower.ends_with(".gif") || lower.ends_with(".webp") {
            ("IMG", Self::FILE_IMAGE)
        } else if lower.ends_with(".sh") || lower.ends_with(".bash") || lower.ends_with(".bat") || lower.ends_with(".ps1") {
            ("SH", Self::FILE_SCRIPT)
        } else if lower.ends_with(".md") || lower.ends_with(".txt") || lower.ends_with(".pdf") || lower.ends_with(".doc") {
            ("DOC", Self::FILE_DOC)
        } else {
            ("FILE", Self::FILE_DEFAULT)
        }
    }
}
