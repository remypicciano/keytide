use eframe::egui;

use crate::app::KeyTideApp;
use crate::ui::theme;

pub fn show(app: &mut KeyTideApp, ctx: &egui::Context) {
    show_error_dialog(app, ctx);
}

fn show_error_dialog(app: &mut KeyTideApp, ctx: &egui::Context) {
    if !app.show_error {
        return;
    }

    let mut open = true;
    let mut dismiss = false;

    egui::Window::new("Unable to continue")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(420.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .frame(
            egui::Frame::window(&ctx.global_style())
                .fill(theme::surface())
                .stroke(egui::Stroke::new(1.0_f32, theme::border()))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(24.0),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("!")
                        .size(38.0)
                        .strong()
                        .color(theme::danger()),
                );

                ui.add_space(10.0);

                ui.heading(egui::RichText::new("Action required").color(theme::text_primary()));

                ui.add_space(8.0);

                ui.label(egui::RichText::new(&app.error_message).color(theme::text_secondary()));

                ui.add_space(22.0);

                if ui
                    .add_sized(
                        [ui.available_width(), 42.0],
                        egui::Button::new("Close").corner_radius(egui::CornerRadius::same(8)),
                    )
                    .clicked()
                {
                    dismiss = true;
                }
            });
        });

    if dismiss || !open {
        app.show_error = false;
    }
}
