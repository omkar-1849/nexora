use eframe::egui::{self, Align, Layout, Rect, Sense, Stroke, Vec2};

use super::state::{ExplorerState, ViewMode};
use super::theme::Theme;
use crate::filesystem::VirtualPath;

pub fn render(ui: &mut egui::Ui, state: &mut ExplorerState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

        // Back Button
        let can_back = state.can_go_back();
        let back_btn = ui.add_enabled(
            can_back,
            egui::Button::new(
                egui::RichText::new("◀")
                    .size(11.0)
                    .color(if can_back { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED }),
            )
            .min_size(Vec2::new(26.0, 26.0))
            .corner_radius(egui::CornerRadius::same(5)),
        );
        if back_btn.clicked() {
            state.go_back();
        }

        // Forward Button
        let can_fwd = state.can_go_forward();
        let fwd_btn = ui.add_enabled(
            can_fwd,
            egui::Button::new(
                egui::RichText::new("▶")
                    .size(11.0)
                    .color(if can_fwd { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED }),
            )
            .min_size(Vec2::new(26.0, 26.0))
            .corner_radius(egui::CornerRadius::same(5)),
        );
        if fwd_btn.clicked() {
            state.go_forward();
        }

        // Up to Parent Button
        let has_parent = state.current_path.parent().is_some();
        let up_btn = ui.add_enabled(
            has_parent,
            egui::Button::new(
                egui::RichText::new("▲")
                    .size(11.0)
                    .color(if has_parent { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED }),
            )
            .min_size(Vec2::new(26.0, 26.0))
            .corner_radius(egui::CornerRadius::same(5)),
        );
        if up_btn.clicked() {
            state.go_up();
        }

        ui.add_space(4.0);

        // Segmented Interactive Breadcrumbs Container
        let breadcrumb_height = 26.0;
        let available_w = (ui.available_width() - 250.0).max(180.0);
        let (breadcrumb_rect, _) = ui.allocate_exact_size(
            Vec2::new(available_w, breadcrumb_height),
            Sense::hover(),
        );

        // Draw Breadcrumb container background
        ui.painter().rect_filled(
            breadcrumb_rect,
            egui::CornerRadius::same(5),
            Theme::BG_INPUT,
        );
        ui.painter().rect_stroke(
            breadcrumb_rect,
            egui::CornerRadius::same(5),
            Stroke::new(1.0, Theme::BORDER_SUBTLE),
            egui::StrokeKind::Inside,
        );

        // Inside breadcrumb container UI
        let mut breadcrumb_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(breadcrumb_rect.shrink2(Vec2::new(4.0, 2.0)))
                .layout(Layout::left_to_right(Align::Center)),
        );

        render_breadcrumbs(&mut breadcrumb_ui, state);

        ui.add_space(8.0);

        // Search Filter Input Box
        let search_width = 170.0;
        let mut search_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    ui.cursor().min,
                    Vec2::new(search_width, 26.0),
                ))
                .layout(Layout::left_to_right(Align::Center)),
        );

        let search_response = search_ui.add_sized(
            Vec2::new(search_width - 24.0, 26.0),
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text(egui::RichText::new("🔍 Filter items...").size(11.5).color(Theme::TEXT_MUTED))
                .text_color(Theme::TEXT_PRIMARY),
        );

        if state.focus_search_requested {
            search_response.request_focus();
            state.focus_search_requested = false;
        }

        if !state.search_query.is_empty()
            && search_ui.add(
                egui::Button::new(egui::RichText::new("✕").size(10.0).color(Theme::TEXT_MUTED))
                    .min_size(Vec2::new(18.0, 22.0))
            ).clicked()
        {
            state.search_query.clear();
        }

        ui.add_space(search_width + 4.0);

        // View Mode Toggle (Grid / List)
        let mode_text = match state.view_mode {
            ViewMode::Grid => "⊞ Grid",
            ViewMode::List => "☰ List",
        };

        if ui.add(
            egui::Button::new(
                egui::RichText::new(mode_text)
                    .size(11.0)
                    .color(Theme::TEXT_SECONDARY),
            )
            .min_size(Vec2::new(56.0, 26.0))
            .corner_radius(egui::CornerRadius::same(5)),
        ).clicked() {
            state.toggle_view_mode();
        }
    });
}

fn render_breadcrumbs(ui: &mut egui::Ui, state: &mut ExplorerState) {
    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

    // Root button
    let is_root = state.current_path.is_root();
    let root_btn = ui.add(
        egui::Button::new(
            egui::RichText::new("/")
                .size(12.0)
                .monospace()
                .strong()
                .color(if is_root { Theme::FOLDER_BODY } else { Theme::TEXT_SECONDARY }),
        )
        .frame(false),
    );

    if root_btn.clicked() {
        state.navigate_to(VirtualPath::root());
    }

    let components = state.current_path.components().to_vec();
    let total_components = components.len();

    let mut accumulated = VirtualPath::root();

    for (index, segment) in components.iter().enumerate() {
        ui.label(
            egui::RichText::new("›")
                .size(12.0)
                .color(Theme::TEXT_MUTED),
        );

        if let Ok(next) = accumulated.join(segment) {
            accumulated = next;
        }

        let is_last = index + 1 == total_components;
        let segment_path = accumulated.clone();

        let btn_text = egui::RichText::new(segment)
            .size(12.0)
            .color(if is_last { Theme::TEXT_PRIMARY } else { Theme::TEXT_SECONDARY });

        let seg_btn = ui.add(
            egui::Button::new(btn_text)
                .frame(false),
        );

        if seg_btn.clicked() {
            state.navigate_to(segment_path);
        }
    }
}