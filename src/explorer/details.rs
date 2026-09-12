use std::time::SystemTime;
use eframe::egui::{self, Align, Layout, Vec2};

use super::state::ExplorerState;
use super::theme::Theme;
use crate::filesystem::FileSystem;

pub fn render(
    ui: &mut egui::Ui,
    filesystem: &mut FileSystem,
    state: &mut ExplorerState,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Header with Close Button
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("INSPECTOR")
                    .size(10.5)
                    .strong()
                    .color(Theme::TEXT_MUTED),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(
                    egui::Button::new(egui::RichText::new("✕").size(11.0).color(Theme::TEXT_MUTED))
                        .frame(false)
                ).clicked() {
                    state.clear_selection();
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        let Some(name) = &state.selected else {
            render_empty_inspector(ui);
            return;
        };

        let Ok(path) = state.current_path.join(name) else {
            render_empty_inspector(ui);
            return;
        };

        let Some(node) = filesystem.node(&path) else {
            ui.label(
                egui::RichText::new("Metadata unavailable for this node")
                    .size(11.5)
                    .color(Theme::TEXT_MUTED),
            );
            return;
        };

        let is_dir = node.is_directory();
        let (tag_type, tag_color) = if is_dir {
            ("DIRECTORY", Theme::FOLDER_BODY)
        } else {
            Theme::file_type_info(name)
        };

        // File/Folder Entity Card
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let badge = ui.add(
                    egui::Button::new(
                        egui::RichText::new(tag_type)
                            .size(9.0)
                            .monospace()
                            .strong()
                            .color(egui::Color32::BLACK),
                    )
                    .fill(tag_color)
                    .corner_radius(egui::CornerRadius::same(3)),
                );
                let _ = badge;

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(name)
                            .size(13.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                });
            });

            ui.add_space(4.0);

            // Copyable Virtual Path
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Path:")
                        .size(10.0)
                        .color(Theme::TEXT_MUTED),
                );
                let path_str = path.to_string();
                if ui.add(
                    egui::Label::new(
                        egui::RichText::new(&path_str)
                            .size(10.0)
                            .monospace()
                            .color(Theme::TEXT_SECONDARY),
                    )
                    .sense(egui::Sense::click()),
                ).on_hover_text("Click to copy virtual path").clicked() {
                    ui.ctx().copy_text(path_str);
                }
            });
        });

        ui.add_space(10.0);

        // Core Storage Attributes Card
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new("STORAGE & METADATA")
                    .size(10.0)
                    .strong()
                    .color(Theme::TEXT_MUTED),
            );
            ui.add_space(6.0);

            render_meta_row(ui, "Type", if is_dir { "Directory" } else { "Regular File" });
            render_meta_row(ui, "Size", &format!("{} ({} bytes)", format_bytes(node.metadata.size), node.metadata.size));
            render_meta_row(ui, "Owner UID", &format!("{}", node.metadata.owner_id));

            let octal_perm = format!("{:04o}", node.metadata.permissions);
            let symbolic_perm = format_symbolic_permissions(node.metadata.permissions, is_dir);
            render_meta_row(ui, "Mode", &format!("{} ({})", octal_perm, symbolic_perm));
        });

        ui.add_space(10.0);

        // Timestamps Card
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new("TIMESTAMPS")
                    .size(10.0)
                    .strong()
                    .color(Theme::TEXT_MUTED),
            );
            ui.add_space(6.0);

            render_meta_row(ui, "Created", &format_system_time(node.metadata.created_at));
            render_meta_row(ui, "Modified", &format_system_time(node.metadata.modified_at));
            render_meta_row(ui, "Accessed", &format_system_time(node.metadata.accessed_at));
        });

        ui.add_space(10.0);

        // AI-Native Status & Integrity Card
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(6.0, 6.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.0, Theme::FILE_MODEL);
                ui.label(
                    egui::RichText::new("AI RUNTIME STATUS")
                        .size(10.0)
                        .strong()
                        .color(Theme::TEXT_MUTED),
                );
            });
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("Unlocked • Available for Agent Access")
                    .size(10.5)
                    .color(Theme::TEXT_SECONDARY),
            );
        });
    });
}

fn render_meta_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(
            egui::RichText::new(label)
                .size(10.5)
                .color(Theme::TEXT_MUTED),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .size(10.5)
                    .monospace()
                    .color(Theme::TEXT_PRIMARY),
            );
        });
    });
    ui.add_space(2.0);
}

fn render_empty_inspector(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        ui.label(
            egui::RichText::new("ℹ")
                .size(24.0)
                .color(Theme::TEXT_MUTED),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("No Item Selected")
                .size(12.0)
                .strong()
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Select a file or folder in the playground to view detailed POSIX metadata.")
                .size(10.5)
                .color(Theme::TEXT_MUTED),
        );
    });
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

fn format_symbolic_permissions(perm: u16, is_dir: bool) -> String {
    let d = if is_dir { 'd' } else { '-' };
    let u_r = if (perm & 0o400) != 0 { 'r' } else { '-' };
    let u_w = if (perm & 0o200) != 0 { 'w' } else { '-' };
    let u_x = if (perm & 0o100) != 0 { 'x' } else { '-' };
    let g_r = if (perm & 0o040) != 0 { 'r' } else { '-' };
    let g_w = if (perm & 0o020) != 0 { 'w' } else { '-' };
    let g_x = if (perm & 0o010) != 0 { 'x' } else { '-' };
    let o_r = if (perm & 0o004) != 0 { 'r' } else { '-' };
    let o_w = if (perm & 0o002) != 0 { 'w' } else { '-' };
    let o_x = if (perm & 0o001) != 0 { 'x' } else { '-' };

    format!("{}{}{}{}{}{}{}{}{}{}", d, u_r, u_w, u_x, g_r, g_w, g_x, o_r, o_w, o_x)
}

fn format_system_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let days_since_epoch = secs / 86400;
            let time_of_day_secs = secs % 86400;
            let hours = time_of_day_secs / 3600;
            let minutes = (time_of_day_secs % 3600) / 60;
            let seconds = time_of_day_secs % 60;

            // Simplified date calculation
            let year = 1970 + days_since_epoch / 365;
            let day_of_year = days_since_epoch % 365;
            let month = (day_of_year / 30) + 1;
            let day = (day_of_year % 30) + 1;

            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month.min(12), day.min(31), hours, minutes, seconds)
        }
        Err(_) => "Unknown".to_string(),
    }
}