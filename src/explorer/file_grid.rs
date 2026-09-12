use eframe::egui::{self, Align2, Color32, Rect, Sense, Stroke, Vec2};

use super::state::{ExplorerState, ViewMode};
use super::theme::Theme;
use crate::filesystem::{FileSystem, VirtualPath};

pub fn render(
    ui: &mut egui::Ui,
    filesystem: &mut FileSystem,
    state: &mut ExplorerState,
) {
    let entries = match filesystem.list_directory(&state.current_path) {
        Ok(entries) => entries,
        Err(error) => {
            render_error_state(ui, &error.to_string());
            return;
        }
    };

    // Filter items based on search query
    let query = state.search_query.trim().to_lowercase();
    let filtered_entries: Vec<String> = entries
        .into_iter()
        .filter(|name| query.is_empty() || name.to_lowercase().contains(&query))
        .collect();

    if filtered_entries.is_empty() {
        if !query.is_empty() {
            render_no_search_results(ui, &state.search_query);
        } else {
            render_empty_state(ui);
        }
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            match state.view_mode {
                ViewMode::Grid => render_grid_view(ui, filesystem, state, &filtered_entries),
                ViewMode::List => render_list_view(ui, filesystem, state, &filtered_entries),
            }
        });
}

fn render_grid_view(
    ui: &mut egui::Ui,
    filesystem: &mut FileSystem,
    state: &mut ExplorerState,
    entries: &[String],
) {
    let available_width = ui.available_width();
    let item_w = Theme::GRID_ITEM_WIDTH;
    let item_h = Theme::GRID_ITEM_HEIGHT;
    let spacing = 14.0;

    let columns = ((available_width + spacing) / (item_w + spacing)).floor().max(1.0) as usize;

    let mut pending_navigation: Option<VirtualPath> = None;

    egui::Grid::new("playground_file_grid")
        .spacing(Vec2::new(spacing, spacing))
        .min_col_width(item_w)
        .show(ui, |ui| {
            for (index, name) in entries.iter().enumerate() {
                let Ok(item_path) = state.current_path.join(name) else {
                    continue;
                };

                let node = filesystem.node(&item_path);
                let is_directory = node.as_ref().map(|n| n.is_directory()).unwrap_or(false);
                let size_bytes = node.as_ref().map(|n| n.metadata.size).unwrap_or(0);

                let is_selected = state.selected.as_deref() == Some(name.as_str());

                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(item_w, item_h),
                    Sense::click(),
                );

                let is_hovered = response.hovered();

                // Animation factors
                let select_id = ui.make_persistent_id(format!("grid_sel_{}_{}", state.current_path, name));
                let hover_id = ui.make_persistent_id(format!("grid_hov_{}_{}", state.current_path, name));
                let select_anim = ui.ctx().animate_bool_with_time(select_id, is_selected, 0.08);
                let hover_anim = ui.ctx().animate_bool_with_time(hover_id, is_hovered, 0.06);

                // Background & border rendering
                let bg_color = if select_anim > 0.001 {
                    Color32::from_rgba_premultiplied(
                        Theme::BG_CARD_SELECTED.r(),
                        Theme::BG_CARD_SELECTED.g(),
                        Theme::BG_CARD_SELECTED.b(),
                        (select_anim * 255.0) as u8,
                    )
                } else if hover_anim > 0.001 {
                    Color32::from_rgba_premultiplied(
                        Theme::BG_CARD_HOVER.r(),
                        Theme::BG_CARD_HOVER.g(),
                        Theme::BG_CARD_HOVER.b(),
                        (hover_anim * 255.0) as u8,
                    )
                } else {
                    Theme::BG_CARD
                };

                ui.painter().rect_filled(rect, Theme::RADIUS_CARD, bg_color);

                let border_stroke = if is_selected {
                    Stroke::new(1.0, Theme::BORDER_SELECTED)
                } else if is_hovered {
                    Stroke::new(1.0, Theme::BORDER_FOCUS)
                } else {
                    Stroke::new(1.0, Theme::BORDER_SUBTLE)
                };

                ui.painter().rect_stroke(
                    rect,
                    Theme::RADIUS_CARD,
                    border_stroke,
                    egui::StrokeKind::Inside,
                );

                // Draw Icon / Badge
                let icon_area = Rect::from_center_size(
                    rect.center() - Vec2::new(0.0, 16.0),
                    Vec2::new(44.0, 44.0),
                );

                if is_directory {
                    draw_yellow_folder_icon(ui, icon_area);
                } else {
                    let (type_tag, tag_color) = Theme::file_type_info(name);
                    draw_file_badge_icon(ui, icon_area, type_tag, tag_color);
                }

                // Draw Filename
                let label_pos = rect.center() + Vec2::new(0.0, 20.0);
                let text_color = if is_selected || is_hovered {
                    Theme::TEXT_PRIMARY
                } else {
                    Theme::TEXT_SECONDARY
                };

                let truncated_name = truncate_filename(name, 14);
                ui.painter().text(
                    label_pos,
                    Align2::CENTER_CENTER,
                    truncated_name,
                    egui::FontId::proportional(11.5),
                    text_color,
                );

                // Draw Subtext (size or type)
                let subtext_pos = rect.center() + Vec2::new(0.0, 36.0);
                let subtext = if is_directory {
                    "Folder".to_string()
                } else {
                    format_bytes(size_bytes)
                };

                ui.painter().text(
                    subtext_pos,
                    Align2::CENTER_CENTER,
                    subtext,
                    egui::FontId::proportional(9.5),
                    Theme::TEXT_MUTED,
                );

                // Interaction Handling
                if response.clicked() {
                    state.select(name.clone());
                }

                if response.double_clicked() {
                    if is_directory {
                        pending_navigation = Some(item_path.clone());
                    } else {
                        state.select(name.clone());
                    }
                }

                // Context Menu
                response.context_menu(|ui| {
                    ui.label(egui::RichText::new(name).strong());
                    ui.separator();
                    if is_directory && ui.button("Open Folder").clicked() {
                        pending_navigation = Some(item_path.clone());
                        ui.close();
                    }
                    if ui.button("Inspect Details").clicked() {
                        state.select(name.clone());
                        ui.close();
                    }
                    if ui.button("Copy Virtual Path").clicked() {
                        ui.ctx().copy_text(item_path.to_string());
                        ui.close();
                    }
                });

                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
        });

    if let Some(path) = pending_navigation {
        state.navigate_to(path);
    }
}

fn render_list_view(
    ui: &mut egui::Ui,
    filesystem: &mut FileSystem,
    state: &mut ExplorerState,
    entries: &[String],
) {
    let mut pending_navigation: Option<VirtualPath> = None;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Name").size(11.0).color(Theme::TEXT_MUTED).strong());
        ui.add_space(ui.available_width() - 240.0);
        ui.label(egui::RichText::new("Permissions").size(11.0).color(Theme::TEXT_MUTED).strong());
        ui.add_space(30.0);
        ui.label(egui::RichText::new("Size").size(11.0).color(Theme::TEXT_MUTED).strong());
    });
    ui.separator();

    for name in entries {
        let Ok(item_path) = state.current_path.join(name) else {
            continue;
        };

        let node = filesystem.node(&item_path);
        let is_directory = node.as_ref().map(|n| n.is_directory()).unwrap_or(false);
        let size_bytes = node.as_ref().map(|n| n.metadata.size).unwrap_or(0);
        let permissions = node.as_ref().map(|n| n.metadata.permissions).unwrap_or(0);

        let is_selected = state.selected.as_deref() == Some(name.as_str());

        let row_height = 28.0;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), row_height),
            Sense::click(),
        );

        let is_hovered = response.hovered();

        if is_selected {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(4), Theme::BG_CARD_SELECTED);
            ui.painter().rect_stroke(rect, egui::CornerRadius::same(4), Stroke::new(1.0, Theme::BORDER_SELECTED), egui::StrokeKind::Inside);
        } else if is_hovered {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(4), Theme::BG_CARD_HOVER);
        }

        let icon_str = if is_directory { "🗀" } else { "📄" };
        let icon_color = if is_directory { Theme::FOLDER_BODY } else { Theme::TEXT_SECONDARY };

        // Draw Row items
        let icon_pos = rect.min + Vec2::new(10.0, rect.height() / 2.0);
        ui.painter().text(icon_pos, Align2::CENTER_CENTER, icon_str, egui::FontId::proportional(13.0), icon_color);

        let name_pos = rect.min + Vec2::new(26.0, rect.height() / 2.0);
        let text_color = if is_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_SECONDARY };
        ui.painter().text(name_pos, Align2::LEFT_CENTER, name, egui::FontId::proportional(12.0), text_color);

        let perm_str = format!("{:o}", permissions);
        let perm_pos = rect.max - Vec2::new(130.0, rect.height() / 2.0);
        ui.painter().text(perm_pos, Align2::CENTER_CENTER, perm_str, egui::FontId::monospace(11.0), Theme::TEXT_MUTED);

        let size_str = if is_directory { "—".to_string() } else { format_bytes(size_bytes) };
        let size_pos = rect.max - Vec2::new(40.0, rect.height() / 2.0);
        ui.painter().text(size_pos, Align2::RIGHT_CENTER, size_str, egui::FontId::monospace(11.0), Theme::TEXT_SECONDARY);

        if response.clicked() {
            state.select(name.clone());
        }

        if response.double_clicked() {
            if is_directory {
                pending_navigation = Some(item_path.clone());
            } else {
                state.select(name.clone());
            }
        }

        ui.add_space(2.0);
    }

    if let Some(path) = pending_navigation {
        state.navigate_to(path);
    }
}

fn draw_yellow_folder_icon(ui: &egui::Ui, rect: Rect) {
    let painter = ui.painter();

    // Folder Back Tab
    let tab_rect = Rect::from_min_size(
        rect.min + Vec2::new(2.0, 4.0),
        Vec2::new(18.0, 10.0),
    );
    painter.rect_filled(tab_rect, egui::CornerRadius::same(3), Theme::FOLDER_TAB);

    // Folder Body
    let body_rect = Rect::from_min_size(
        rect.min + Vec2::new(2.0, 10.0),
        Vec2::new(rect.width() - 4.0, rect.height() - 14.0),
    );
    painter.rect_filled(body_rect, egui::CornerRadius::same(4), Theme::FOLDER_BODY);

    // Front flap subtle highlight
    let front_rect = Rect::from_min_size(
        rect.min + Vec2::new(2.0, 16.0),
        Vec2::new(rect.width() - 4.0, rect.height() - 20.0),
    );
    painter.rect_filled(front_rect, egui::CornerRadius::same(3), Theme::FOLDER_FRONT);
}

fn draw_file_badge_icon(ui: &egui::Ui, rect: Rect, type_tag: &str, tag_color: Color32) {
    let painter = ui.painter();

    // File paper background
    let paper_rect = Rect::from_min_size(
        rect.min + Vec2::new(6.0, 2.0),
        Vec2::new(rect.width() - 12.0, rect.height() - 4.0),
    );
    painter.rect_filled(paper_rect, egui::CornerRadius::same(3), Color32::from_rgb(32, 34, 38));
    painter.rect_stroke(
        paper_rect,
        egui::CornerRadius::same(3),
        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 30)),
        egui::StrokeKind::Inside,
    );

    // Type Badge pill at bottom
    let badge_rect = Rect::from_min_size(
        paper_rect.min + Vec2::new(2.0, paper_rect.height() - 14.0),
        Vec2::new(paper_rect.width() - 4.0, 12.0),
    );
    painter.rect_filled(badge_rect, egui::CornerRadius::same(2), tag_color);

    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        type_tag,
        egui::FontId::monospace(8.0),
        Color32::from_rgb(10, 10, 10),
    );

    // File inner symbol
    let glyph = match type_tag {
        "PY" => "🐍",
        "RS" => "⚙",
        "ML" => "⌬",
        "IMG" => "🖼",
        "CFG" => "{ }",
        _ => "≡",
    };

    painter.text(
        paper_rect.center() - Vec2::new(0.0, 4.0),
        Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(11.0),
        Theme::TEXT_PRIMARY,
    );
}

fn render_empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        ui.label(
            egui::RichText::new("🗀")
                .size(36.0)
                .color(Theme::TEXT_MUTED),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Empty Virtual Directory")
                .size(14.0)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("This folder contains no files or subdirectories.")
                .size(11.5)
                .color(Theme::TEXT_MUTED),
        );
    });
}

fn render_no_search_results(ui: &mut egui::Ui, query: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        ui.label(
            egui::RichText::new("🔍")
                .size(32.0)
                .color(Theme::TEXT_MUTED),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("No matches found for \"{}\"", query))
                .size(13.5)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Try a different search term or clear the filter.")
                .size(11.5)
                .color(Theme::TEXT_MUTED),
        );
    });
}

fn render_error_state(ui: &mut egui::Ui, error_msg: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        ui.label(
            egui::RichText::new("⚠")
                .size(32.0)
                .color(Theme::FILE_SCRIPT),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Unable to Read Virtual Directory")
                .size(13.5)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(error_msg)
                .size(11.5)
                .color(Theme::TEXT_MUTED),
        );
    });
}

fn truncate_filename(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        name.to_string()
    } else {
        let prefix: String = name.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", prefix)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        "0 B".to_string()
    } else if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}