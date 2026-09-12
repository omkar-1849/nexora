use eframe::egui::{self, Key, Modifiers, Rect, Stroke, Vec2};

use crate::filesystem::FileSystem;

use super::{
    details,
    file_grid,
    sidebar,
    state::ExplorerState,
    theme::Theme,
    toolbar,
};

pub struct ExplorerView {
    pub state: ExplorerState,
    theme_applied: bool,
}

impl ExplorerView {
    pub fn new() -> Self {
        Self {
            state: ExplorerState::new(),
            theme_applied: false,
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        filesystem: &mut FileSystem,
    ) {
        if !self.theme_applied {
            Theme::apply(ui.ctx());
            self.theme_applied = true;
        }

        self.handle_input(ui, filesystem);

        let available_size = ui.available_size();
        let total_rect = Rect::from_min_size(ui.cursor().min, available_size);

        // Render Sidebar
        let sidebar_width = Theme::SIDEBAR_WIDTH;
        let sidebar_rect = Rect::from_min_size(
            total_rect.min,
            Vec2::new(sidebar_width, total_rect.height()),
        );

        ui.painter().rect_filled(sidebar_rect, egui::CornerRadius::ZERO, Theme::BG_PANEL);
        ui.painter().line_segment(
            [sidebar_rect.right_top(), sidebar_rect.right_bottom()],
            Stroke::new(1.0, Theme::BORDER_DARK),
        );

        let mut sidebar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(sidebar_rect.shrink(8.0))
        );
        sidebar::render(&mut sidebar_ui, &mut self.state);

        // Calculate Inspector animated width
        let is_inspector_open = self.state.show_details && self.state.selected.is_some();
        let inspector_anim_id = ui.make_persistent_id("inspector_drawer_anim");
        let inspector_factor = ui.ctx().animate_bool_with_time(inspector_anim_id, is_inspector_open, 0.14);
        let inspector_width = Theme::INSPECTOR_WIDTH * inspector_factor;

        // Render Inspector on Right (if width > 0)
        if inspector_width > 1.0 {
            let inspector_rect = Rect::from_min_size(
                egui::pos2(total_rect.right() - inspector_width, total_rect.top()),
                Vec2::new(inspector_width, total_rect.height()),
            );

            ui.painter().rect_filled(inspector_rect, egui::CornerRadius::ZERO, Theme::BG_PANEL);
            ui.painter().line_segment(
                [inspector_rect.left_top(), inspector_rect.left_bottom()],
                Stroke::new(1.0, Theme::BORDER_DARK),
            );

            if inspector_factor > 0.6 {
                let mut inspector_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(inspector_rect.shrink(10.0))
                );
                details::render(&mut inspector_ui, filesystem, &mut self.state);
            }
        }

        // Center Area (Toolbar + Playground)
        let main_left = sidebar_rect.right() + 1.0;
        let main_right = total_rect.right() - inspector_width;
        let main_width = (main_right - main_left).max(100.0);

        let main_rect = Rect::from_min_size(
            egui::pos2(main_left, total_rect.top()),
            Vec2::new(main_width, total_rect.height()),
        );

        // Top Toolbar
        let toolbar_rect = Rect::from_min_size(
            main_rect.min,
            Vec2::new(main_rect.width(), Theme::TOPBAR_HEIGHT),
        );
        ui.painter().line_segment(
            [toolbar_rect.left_bottom(), toolbar_rect.right_bottom()],
            Stroke::new(1.0, Theme::BORDER_DARK),
        );

        let mut toolbar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(toolbar_rect.shrink2(Vec2::new(12.0, 8.0)))
        );
        toolbar::render(&mut toolbar_ui, &mut self.state);

        // Playground Stage
        let playground_rect = Rect::from_min_size(
            toolbar_rect.left_bottom() + Vec2::new(0.0, 1.0),
            Vec2::new(main_rect.width(), main_rect.height() - Theme::TOPBAR_HEIGHT - 1.0),
        );

        let mut playground_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(playground_rect.shrink(12.0))
        );
        file_grid::render(&mut playground_ui, filesystem, &mut self.state);
    }

    fn handle_input(&mut self, ui: &egui::Ui, filesystem: &mut FileSystem) {
        let input = ui.input(|i| i.clone());

        // Ctrl + F: Search
        if input.modifiers.matches_exact(Modifiers::COMMAND) && input.key_pressed(Key::F) {
            self.state.focus_search_requested = true;
        }

        // Escape: Clear Search or Selection
        if input.key_pressed(Key::Escape) {
            if !self.state.search_query.is_empty() {
                self.state.search_query.clear();
            } else {
                self.state.clear_selection();
            }
        }

        // Backspace or Alt + Left: Navigate Back
        if (input.key_pressed(Key::Backspace)
            || (input.modifiers.alt && input.key_pressed(Key::ArrowLeft)))
            && self.state.search_query.is_empty()
        {
            self.state.go_back();
        }

        // Alt + Right: Navigate Forward
        if input.modifiers.alt && input.key_pressed(Key::ArrowRight) {
            self.state.go_forward();
        }

        // Alt + Up: Navigate Up
        if input.modifiers.alt && input.key_pressed(Key::ArrowUp) {
            self.state.go_up();
        }

        // Enter on Selected Item
        if input.key_pressed(Key::Enter) {
            if let Some(name) = &self.state.selected {
                if let Ok(path) = self.state.current_path.join(name) {
                    if let Some(node) = filesystem.node(&path) {
                        if node.is_directory() {
                            self.state.navigate_to(path);
                        }
                    }
                }
            }
        }
    }
}

impl Default for ExplorerView {
    fn default() -> Self {
        Self::new()
    }
}