use eframe::egui;

use crate::app::KeyTideApp;
use crate::ui::theme;

const VIEWER_ID: &str = "coffer_secure_viewer";

pub fn show(app: &mut KeyTideApp, ctx: &egui::Context) {
    if !app.show_viewer {
        app.viewer_was_open = false;
        return;
    }

    let just_opened = !app.viewer_was_open;

    if just_opened {
        app.main_window_size_before_viewer =
            ctx.input(|input| input.viewport().inner_rect.map(|rect| rect.size()));

        app.viewer_was_open = true;
    }

    let initial_size = app
        .main_window_size_before_viewer
        .unwrap_or_else(|| egui::Vec2::new(960.0, 720.0));

    let mut close_and_wipe = false;

    let mut builder = egui::ViewportBuilder::default()
        .with_title("KeyTide Secure Viewer")
        .with_min_inner_size([620.0, 480.0])
        .with_resizable(true);

    // Only request a size when the native viewer is first created.
    // Reapplying it every frame can continually resize a macOS tab group.
    if just_opened {
        builder = builder.with_inner_size(initial_size);
    }

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of(VIEWER_ID),
        builder,
        |viewer_ui, _class| {
            if viewer_ui.input(|input| input.viewport().close_requested()) {
                close_and_wipe = true;
            }

            egui::Panel::top("secure_viewer_header")
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(theme::background())
                        .inner_margin(egui::Margin::symmetric(24, 20)),
                )
                .show_inside(viewer_ui, |ui| {
                    viewer_header(ui);
                });

            egui::Panel::bottom("secure_viewer_footer")
                .resizable(false)
                .exact_size(112.0)
                .frame(
                    egui::Frame::new()
                        .fill(theme::background())
                        .inner_margin(egui::Margin::symmetric(24, 14)),
                )
                .show_inside(viewer_ui, |ui| {
                    ui.label(
                        egui::RichText::new("Closing clears KeyTide’s in-memory preview buffer.")
                            .small()
                            .color(theme::text_secondary()),
                    );

                    ui.add_space(10.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 46.0],
                            egui::Button::new(
                                egui::RichText::new("Close preview")
                                    .strong()
                                    .color(theme::on_primary()),
                            )
                            .fill(theme::danger())
                            .corner_radius(egui::CornerRadius::same(8)),
                        )
                        .clicked()
                    {
                        close_and_wipe = true;
                    }
                });

            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(theme::background())
                        .inner_margin(egui::Margin::symmetric(24, 12)),
                )
                .show_inside(viewer_ui, |ui| {
                    egui::Frame::new()
                        .fill(theme::surface())
                        .stroke(egui::Stroke::new(1.0_f32, theme::border()))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.set_min_size(ui.available_size());

                            egui::ScrollArea::vertical()
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                )
                                .auto_shrink([false, false])
                                .show(ui, |ui| match app.decrypted_text.as_deref() {
                                    Some(text) => {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(text)
                                                    .monospace()
                                                    .size(15.0)
                                                    .color(theme::text_primary()),
                                            )
                                            .wrap(),
                                        );
                                    }

                                    None => {
                                        empty_viewer(ui);
                                    }
                                });
                        });
                });
        },
    );

    if close_and_wipe {
        if let Some(size) = app.main_window_size_before_viewer {
            ctx.send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                egui::ViewportCommand::InnerSize(size),
            );
        }

        app.close_secure_viewer();
    }
}

fn viewer_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(theme::primary())
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("C")
                        .size(20.0)
                        .strong()
                        .color(theme::on_primary()),
                );
            });

        ui.add_space(8.0);

        ui.vertical(|ui| {
            ui.heading(
                egui::RichText::new("Secure Viewer")
                    .size(22.0)
                    .strong()
                    .color(theme::text_primary()),
            );

            ui.label(
                egui::RichText::new("Read-only sample preview")
                    .small()
                    .color(theme::text_secondary()),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            status_chip(ui);
        });
    });
}

fn status_chip(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(theme::surface_raised())
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(11, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(theme::warning(), "!");

                ui.label(
                    egui::RichText::new("Temporary preview")
                        .small()
                        .strong()
                        .color(theme::text_primary()),
                );
            });
        });
}

fn empty_viewer(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);

        ui.heading(egui::RichText::new("Nothing to display").color(theme::text_primary()));

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("The decrypted session no longer contains readable text.")
                .color(theme::text_secondary()),
        );
    });
}
