use eframe::egui::{self, Align, Align2, Color32, Layout, Rect, Sense, Vec2};

use super::state::ExplorerState;
use super::theme::Theme;
use crate::filesystem::VirtualPath;

struct LocationItem {
    label: &'static str,
    path: &'static str,
    icon: &'static str,
}

pub fn render(ui: &mut egui::Ui, state: &mut ExplorerState) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        // Sidebar Header
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("VIRTUAL ROOTS")
                    .size(10.5)
                    .strong()
                    .color(Theme::TEXT_MUTED),
            );
        });

        ui.add_space(10.0);

        let locations = [
            LocationItem {
                label: "/home",
                path: "/home",
                icon: "⌂",
            },
            LocationItem {
                label: "/workspace",
                path: "/workspace",
                icon: "⌘",
            },
            LocationItem {
                label: "/projects",
                path: "/projects",
                icon: "◈",
            },
            LocationItem {
                label: "/data",
                path: "/data",
                icon: "▤",
            },
            LocationItem {
                label: "/models",
                path: "/models",
                icon: "⌬",
            },
            LocationItem {
                label: "/tmp",
                path: "/tmp",
                icon: "⌁",
            },
        ];

        for item in &locations {
            let is_active = state.current_path.to_string() == item.path
                || state.current_path.to_string().starts_with(&format!("{}/", item.path));

            let desired_size = Vec2::new(ui.available_width(), 32.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

            let is_hovered = response.hovered();

            // Animated active factor
            let active_id = ui.make_persistent_id(format!("side_active_{}", item.path));
            let hover_id = ui.make_persistent_id(format!("side_hover_{}", item.path));
            let active_anim = ui.ctx().animate_bool_with_time(active_id, is_active, 0.12);
            let hover_anim = ui.ctx().animate_bool_with_time(hover_id, is_hovered, 0.08);

            // Draw background
            if active_anim > 0.001 {
                let bg = Color32::from_rgba_premultiplied(
                    Theme::BG_CARD_SELECTED.r(),
                    Theme::BG_CARD_SELECTED.g(),
                    Theme::BG_CARD_SELECTED.b(),
                    (active_anim * 255.0) as u8,
                );
                ui.painter().rect_filled(rect, egui::CornerRadius::same(6), bg);

                // Active marker bar on left
                let bar_rect = Rect::from_min_size(
                    rect.min + Vec2::new(2.0, 6.0),
                    Vec2::new(3.0, rect.height() - 12.0),
                );
                ui.painter().rect_filled(bar_rect, egui::CornerRadius::same(2), Theme::FOLDER_BODY);
            } else if hover_anim > 0.001 {
                let bg = Color32::from_rgba_premultiplied(
                    Theme::BG_CARD_HOVER.r(),
                    Theme::BG_CARD_HOVER.g(),
                    Theme::BG_CARD_HOVER.b(),
                    (hover_anim * 255.0) as u8,
                );
                ui.painter().rect_filled(rect, egui::CornerRadius::same(6), bg);
            }

            // Draw Icon and Text
            let icon_pos = rect.min + Vec2::new(14.0, rect.height() / 2.0);
            let text_pos = rect.min + Vec2::new(34.0, rect.height() / 2.0);

            let text_color = if is_active || is_hovered {
                Theme::TEXT_PRIMARY
            } else {
                Theme::TEXT_SECONDARY
            };

            let icon_color = if is_active {
                Theme::FOLDER_BODY
            } else {
                Theme::TEXT_MUTED
            };

            ui.painter().text(
                icon_pos,
                Align2::CENTER_CENTER,
                item.icon,
                egui::FontId::monospace(13.0),
                icon_color,
            );

            ui.painter().text(
                text_pos,
                Align2::LEFT_CENTER,
                item.label,
                egui::FontId::proportional(13.0),
                text_color,
            );

            if response.clicked() {
                if let Ok(path) = VirtualPath::parse(item.path) {
                    state.navigate_to(path);
                }
            }

            ui.add_space(2.0);
        }

        // Bottom section: System Info / Host Isolation marker
        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("NEXORA CORE FS v1.0")
                        .size(9.5)
                        .color(Theme::TEXT_MUTED),
                );
            });

            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(6.0, 6.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.0, Color32::from_rgb(152, 195, 121));
                ui.label(
                    egui::RichText::new("Virtual Isolation Active")
                        .size(10.0)
                        .color(Theme::TEXT_SECONDARY),
                );
            });

            ui.separator();
        });
    });
}