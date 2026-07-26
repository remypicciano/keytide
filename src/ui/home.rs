use eframe::egui;

use crate::app::{KeyTideApp, NoticeKind, OpenStage, ProtectStage, ThemeMode, Workflow};
use crate::ui::{theme, widgets};

pub fn show_splash(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let now = ctx.input(|input| input.time);
    let opacity = app.splash_opacity(now);
    let tint = |color: egui::Color32| color.gamma_multiply(opacity);

    let progress = app.splash_progress(now);
    let scale = 0.82 + 0.18 * smooth_out(progress);
    let glow = (opacity * (1.0 - (progress - 0.55).max(0.0) / 0.45)).clamp(0.0, 1.0);

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(egui::Color32::from_rgb(6, 9, 17)))
        .show_inside(ui, |ui| {
            let response = ui.interact(
                ui.max_rect(),
                ui.id().with("dismiss_splash"),
                egui::Sense::click(),
            );
            if response.clicked() {
                app.dismiss_splash();
                return;
            }

            let center = ui.max_rect().center() - egui::vec2(0.0, 72.0);
            ui.painter().circle_filled(
                center,
                124.0 * scale,
                egui::Color32::from_rgba_unmultiplied(123, 105, 255, (24.0 * glow) as u8),
            );
            ui.painter().circle_filled(
                center,
                78.0 * scale,
                egui::Color32::from_rgba_unmultiplied(123, 105, 255, (82.0 * opacity) as u8),
            );
            ui.painter().circle_stroke(
                center,
                78.0 * scale,
                egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgba_unmultiplied(213, 211, 255, (120.0 * opacity) as u8),
                ),
            );
            ui.painter().circle_filled(
                center + egui::vec2(55.0, -51.0),
                9.0 * scale,
                egui::Color32::from_rgba_unmultiplied(197, 255, 0, (220.0 * opacity) as u8),
            );

            let logo_size = 104.0 * scale;
            let logo_rect = egui::Rect::from_center_size(center, egui::Vec2::splat(logo_size));
            ui.put(
                logo_rect,
                egui::Image::new(egui::include_image!("../../assets/Pastelito_img.webp"))
                    .fit_to_exact_size(egui::Vec2::splat(logo_size))
                    .corner_radius(24.0)
                    .tint(egui::Color32::from_white_alpha((255.0 * opacity) as u8)),
            );

            ui.painter().text(
                center + egui::vec2(0.0, 116.0),
                egui::Align2::CENTER_CENTER,
                "KeyTide",
                egui::FontId::proportional(38.0),
                tint(egui::Color32::WHITE),
            );
            ui.painter().text(
                center + egui::vec2(0.0, 154.0),
                egui::Align2::CENTER_CENTER,
                "Private file protection on your device",
                egui::FontId::proportional(16.0),
                tint(egui::Color32::from_rgb(213, 211, 255)),
            );
            ui.painter().text(
                egui::pos2(ui.max_rect().center().x, ui.max_rect().bottom() - 30.0),
                egui::Align2::CENTER_CENTER,
                "Created by Rémy Picciano",
                egui::FontId::proportional(12.0),
                tint(egui::Color32::from_rgb(145, 143, 163)),
            );
        });
}

fn smooth_out(value: f32) -> f32 {
    1.0 - (1.0 - value.clamp(0.0, 1.0)).powi(3)
}

pub fn show(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let compact = ctx.input(|input| input.content_rect().width()) < 980.0;

    egui::Panel::top("workspace_header")
        .exact_size(68.0)
        .frame(
            egui::Frame::new()
                .fill(theme::surface())
                .stroke(egui::Stroke::new(1.0_f32, theme::border()))
                .inner_margin(egui::Margin::symmetric(if compact { 16 } else { 28 }, 10)),
        )
        .show_inside(ui, |ui| header(app, ui, compact));

    egui::Panel::bottom("workspace_actions")
        .exact_size(70.0)
        .frame(
            egui::Frame::new()
                .fill(theme::surface())
                .stroke(egui::Stroke::new(1.0_f32, theme::border()))
                .inner_margin(egui::Margin::symmetric(if compact { 16 } else { 28 }, 12)),
        )
        .show_inside(ui, |ui| action_bar(app, ui));

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::background()))
        .show_inside(ui, |ui| content(app, ui, compact));
}

fn header(app: &mut KeyTideApp, ui: &mut egui::Ui, compact: bool) {
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!("../../assets/Pastelito_img.webp"))
                .fit_to_exact_size(egui::Vec2::splat(40.0))
                .corner_radius(8.0),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("KeyTide")
                .size(18.0)
                .strong()
                .color(theme::text_primary()),
        );
        ui.add_space(if compact { 10.0 } else { 34.0 });
        nav_button(app, ui, Workflow::Protect, "Protect");
        nav_button(app, ui, Workflow::Open, "Open");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            nav_button(app, ui, Workflow::Settings, "Settings");
            if !compact {
                nav_button(app, ui, Workflow::Security, "Security");
            }
            theme_switch(app, ui);
        });
    });
}

fn nav_button(app: &mut KeyTideApp, ui: &mut egui::Ui, workflow: Workflow, label: &str) {
    let selected = app.workflow == workflow;
    let response = ui.add_sized(
        [82.0, 38.0],
        egui::Button::new(egui::RichText::new(label).strong().color(if selected {
            theme::primary()
        } else {
            theme::text_secondary()
        }))
        .fill(if selected {
            theme::primary().gamma_multiply(0.12)
        } else {
            egui::Color32::TRANSPARENT
        })
        .stroke(if selected {
            egui::Stroke::new(1.0_f32, theme::primary().gamma_multiply(0.35))
        } else {
            egui::Stroke::NONE
        })
        .corner_radius(8.0),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        app.navigate(workflow);
    }
}

fn theme_switch(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    let dark = app.theme_mode == ThemeMode::Dark;
    let desired = egui::Vec2::new(122.0, 36.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
    let amount = ui.ctx().animate_bool(response.id, dark);

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(8),
        theme::surface_raised(),
        egui::Stroke::new(1.0_f32, theme::border()),
        egui::StrokeKind::Inside,
    );
    let half = rect.width() * 0.5;
    let selected_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + amount * half, rect.top()),
        egui::vec2(half, rect.height()),
    )
    .shrink(3.0);
    ui.painter()
        .rect_filled(selected_rect, egui::CornerRadius::same(6), theme::primary());
    for (label, center, selected) in [
        (
            "Light",
            egui::pos2(rect.left() + half * 0.5, rect.center().y),
            !dark,
        ),
        (
            "Dark",
            egui::pos2(rect.left() + half * 1.5, rect.center().y),
            dark,
        ),
    ] {
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            if selected {
                theme::on_primary()
            } else {
                theme::text_secondary()
            },
        );
    }
    if response.clicked() {
        app.set_theme_mode(if dark {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        });
    }
    response.on_hover_text("Change application appearance");
}

fn content(app: &mut KeyTideApp, ui: &mut egui::Ui, compact: bool) {
    let inset = if compact { 18.0 } else { 32.0 };
    let width = (ui.available_width() - inset * 2.0).clamp(0.0, 1360.0);
    let offset = ((ui.available_width() - width) * 0.5).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(offset);
        ui.allocate_ui_with_layout(
            egui::Vec2::new(width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| match app.workflow {
                Workflow::Protect | Workflow::Open => workflow_shell(app, ui, compact),
                Workflow::Security => static_page(ui, "Security", security_content),
                Workflow::Settings => settings_page(app, ui),
            },
        );
    });
}

fn workflow_shell(app: &mut KeyTideApp, ui: &mut egui::Ui, compact: bool) {
    egui::ScrollArea::vertical()
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(if compact { 22.0 } else { 34.0 });
            if compact {
                compact_progress(app, ui);
                ui.add_space(22.0);
                workflow_content(app, ui);
            } else {
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| workflow_rail(app, ui),
                    );
                    ui.add_space(30.0);
                    ui.separator();
                    ui.add_space(30.0);
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| workflow_content(app, ui),
                    );
                });
            }
            ui.add_space(42.0);
        });
}

fn workflow_rail(app: &KeyTideApp, ui: &mut egui::Ui) {
    let (title, description) = match app.workflow {
        Workflow::Protect => (
            "Protect",
            "Create a protected copy and a separate unlock key.",
        ),
        Workflow::Open => ("Open", "Authenticate and restore a protected file."),
        Workflow::Security | Workflow::Settings => return,
    };
    ui.label(
        egui::RichText::new(format!("{} WORKFLOW", title.to_uppercase()))
            .size(11.0)
            .strong()
            .color(theme::primary()),
    );
    ui.add_space(8.0);
    ui.heading(
        egui::RichText::new("Your progress")
            .size(24.0)
            .strong()
            .color(theme::text_primary()),
    );
    ui.add_space(7.0);
    ui.label(
        egui::RichText::new("These steps update automatically and are not clickable.")
            .small()
            .color(theme::text_muted()),
    );
    ui.add_space(22.0);

    match app.workflow {
        Workflow::Protect => {
            let active = protect_stage_index(app.protect_stage);
            for (index, label) in ["Choose file", "Review", "Protect", "Complete"]
                .iter()
                .enumerate()
            {
                rail_step(ui, label, index, active);
            }
        }
        Workflow::Open => {
            let active = open_stage_index(app.open_stage);
            for (index, label) in [
                "Protected file",
                "Unlock key",
                "Review",
                "Restore",
                "Complete",
            ]
            .iter()
            .enumerate()
            {
                rail_step(ui, label, index, active);
            }
        }
        Workflow::Security | Workflow::Settings => {}
    }

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(18.0);
    ui.label(
        egui::RichText::new(description)
            .small()
            .color(theme::text_muted()),
    );
}

fn rail_step(ui: &mut egui::Ui, label: &str, index: usize, active: usize) {
    let current = index == active;
    let complete = index < active;
    let desired = egui::vec2(ui.available_width(), 48.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let marker = egui::pos2(rect.left() + 20.0, rect.center().y);
    if current {
        ui.painter()
            .rect_filled(rect, 10.0, theme::primary().gamma_multiply(0.10));
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
            2.0,
            theme::primary(),
        );
    }
    if index > 0 {
        ui.painter().line_segment(
            [
                egui::pos2(marker.x, rect.top()),
                egui::pos2(marker.x, marker.y - 13.0),
            ],
            egui::Stroke::new(1.0_f32, theme::border()),
        );
    }
    ui.painter().circle_filled(
        marker,
        11.0,
        if current || complete {
            theme::primary()
        } else {
            theme::surface_raised()
        },
    );
    ui.painter()
        .circle_stroke(marker, 11.0, egui::Stroke::new(1.0_f32, theme::border()));
    ui.painter().text(
        marker,
        egui::Align2::CENTER_CENTER,
        if complete {
            "OK".to_string()
        } else {
            (index + 1).to_string()
        },
        egui::FontId::proportional(if complete { 8.0 } else { 11.0 }),
        if current || complete {
            theme::on_primary()
        } else {
            theme::text_muted()
        },
    );
    ui.painter().text(
        egui::pos2(marker.x + 25.0, marker.y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        if current {
            theme::text_primary()
        } else {
            theme::text_muted()
        },
    );
}

fn compact_progress(app: &KeyTideApp, ui: &mut egui::Ui) {
    let (active, labels): (usize, &[&str]) = match app.workflow {
        Workflow::Protect => (
            protect_stage_index(app.protect_stage),
            &["Choose", "Review", "Protect", "Done"],
        ),
        Workflow::Open => (
            open_stage_index(app.open_stage),
            &["File", "Key", "Review", "Restore", "Done"],
        ),
        Workflow::Security | Workflow::Settings => return,
    };
    widgets::workflow_steps(ui, active, labels);
}

fn workflow_content(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    match app.workflow {
        Workflow::Protect => protect_page(app, ui),
        Workflow::Open => open_page(app, ui),
        Workflow::Security | Workflow::Settings => {}
    }
}

fn page_intro(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.heading(
        egui::RichText::new(title)
            .size(32.0)
            .strong()
            .color(theme::text_primary()),
    );
    ui.add_space(10.0);
    ui.label(
        egui::RichText::new(description)
            .size(16.0)
            .color(theme::text_secondary()),
    );
    ui.add_space(28.0);
}

fn protect_page(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    match app.protect_stage {
        ProtectStage::SelectFile => protect_select(app, ui),
        ProtectStage::Review => protect_review(app, ui),
        ProtectStage::Processing => processing(app, ui, "Preparing protected output"),
        ProtectStage::Complete => protect_complete(app, ui),
    }
}

fn protect_select(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Choose a file",
        "The original remains unchanged. KeyTide creates a protected copy.",
    );
    match app.source_file.as_ref() {
        None => {
            if widgets::drop_zone(ui, "Drop a file", "Any local file is supported.", "browse")
                .clicked()
            {
                app.select_source_file();
            }
        }
        Some(file) => {
            if widgets::file_card(ui, file) {
                app.clear_source_file();
                return;
            }
            ui.add_space(26.0);
            inline_notice(
                ui,
                "Fresh key for this file",
                "KeyTide will create a new random .cofferkey. Every protected file receives its own key so one lost or exposed key cannot unlock your other files.",
                theme::primary(),
            );
        }
    }
}

fn protect_review(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Review and save",
        "Choose where the protected copy and key will be placed.",
    );
    if let Some(file) = app.source_file.as_ref() {
        widgets::compact_file_row(ui, "Source", file, || {});
    }
    ui.add_space(22.0);
    let destination = app.protect_destination.clone();
    let choose = output_editor(
        ui,
        "Protected copy",
        &mut app.protected_filename,
        destination.as_ref(),
        true,
    );
    if choose {
        app.choose_protect_destination();
    }
    app.refresh_planned_outputs();
    ui.add_space(18.0);
    inline_notice(
        ui,
        "New unlock key",
        "A unique .cofferkey will be saved beside the protected copy. Move it somewhere separate after saving.",
        theme::primary(),
    );
}

fn open_page(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    match app.open_stage {
        OpenStage::SelectContainer => open_container(app, ui),
        OpenStage::SelectKey => open_key(app, ui),
        OpenStage::Review => open_review(app, ui),
        OpenStage::Processing => processing(app, ui, "Authenticating protected file"),
        OpenStage::Complete => open_complete(app, ui),
    }
}

fn open_container(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Choose a protected file",
        "Start with the .coffer container you want to restore.",
    );
    if widgets::drop_zone(
        ui,
        "Drop a .coffer file",
        "The container is authenticated before anything is restored.",
        "browse",
    )
    .clicked()
    {
        app.select_encrypted_file();
    }
}

fn open_key(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Choose the unlock key",
        "KeyTide will verify that the key and protected file belong together.",
    );
    if let Some(file) = app.encrypted_file.as_ref() {
        widgets::compact_file_row(ui, "Protected file", file, || {});
        ui.add_space(18.0);
    }
    match app.key_file.as_ref() {
        Some(file) => {
            if widgets::file_card(ui, file) {
                app.clear_key_file();
            }
        }
        None => {
            if widgets::drop_zone(
                ui,
                "Drop the key file",
                "Wrong or changed files fail with the same safe error.",
                "browse",
            )
            .clicked()
            {
                app.select_key();
            }
        }
    }
}

fn open_review(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Review and restore",
        "Choose where the authenticated file will be written.",
    );
    if let Some(file) = app.encrypted_file.as_ref() {
        widgets::compact_file_row(ui, "Protected file", file, || {});
    }
    ui.add_space(8.0);
    if let Some(file) = app.key_file.as_ref() {
        widgets::compact_file_row(ui, "Unlock key", file, || {});
    }
    ui.add_space(22.0);
    let destination = app.restore_destination.clone();
    let choose = output_editor(
        ui,
        "Restored file",
        &mut app.restored_filename,
        destination.as_ref(),
        false,
    );
    if choose {
        app.choose_restore_destination();
    }
    app.refresh_planned_outputs();
    ui.add_space(18.0);
    inline_notice(
        ui,
        "Authentication",
        "Plaintext is not committed until the container has been authenticated.",
        theme::primary(),
    );
}

fn output_editor(
    ui: &mut egui::Ui,
    title: &str,
    filename: &mut String,
    destination: Option<&std::path::PathBuf>,
    require_coffer_extension: bool,
) -> bool {
    let mut choose = false;
    egui::Frame::new()
        .fill(theme::surface())
        .stroke(egui::Stroke::new(1.0_f32, theme::border()))
        .corner_radius(8.0)
        .inner_margin(20.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading(
                egui::RichText::new(title)
                    .size(20.0)
                    .strong()
                    .color(theme::text_primary()),
            );
            ui.add_space(18.0);
            ui.label(
                egui::RichText::new("Destination folder")
                    .small()
                    .strong()
                    .color(theme::text_secondary()),
            );
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(
                            destination
                                .map(|path| path.display().to_string())
                                .unwrap_or_else(|| "No folder selected".to_string()),
                        )
                        .color(theme::text_primary()),
                    )
                    .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Choose folder").clicked() {
                        choose = true;
                    }
                });
            });
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Filename")
                    .small()
                    .strong()
                    .color(theme::text_secondary()),
            );
            ui.add(egui::TextEdit::singleline(filename).desired_width(f32::INFINITY));
            if require_coffer_extension && !filename.to_ascii_lowercase().ends_with(".coffer") {
                ui.add_space(6.0);
                ui.colored_label(theme::warning(), "Filename must end in .coffer");
            }
        });
    choose
}

fn inline_notice(ui: &mut egui::Ui, title: &str, body: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.07))
        .stroke(egui::Stroke::new(1.0_f32, color.gamma_multiply(0.5)))
        .corner_radius(8.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(title).strong().color(color));
            ui.add_space(5.0);
            ui.label(egui::RichText::new(body).color(theme::text_secondary()));
        });
}

fn processing(app: &mut KeyTideApp, ui: &mut egui::Ui, title: &str) {
    page_intro(ui, title, "Keep KeyTide open while this operation finishes.");
    egui::Frame::new()
        .fill(theme::surface())
        .stroke(egui::Stroke::new(1.0_f32, theme::border()))
        .corner_radius(8.0)
        .inner_margin(24.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::ProgressBar::new(app.progress)
                    .show_percentage()
                    .animate(true),
            );
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("KeyTide is working locally. Keep the application open.")
                    .color(theme::text_secondary()),
            );
            ui.add_space(16.0);
            if ui.button("Cancel").clicked() {
                app.cancel_processing();
            }
        });
}

fn protect_complete(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Protection complete",
        "Your protected file is ready. Keep its unlock key somewhere separate.",
    );
    result_paths(
        app,
        ui,
        app.encryption_output.clone(),
        app.key_output.clone(),
        true,
    );
}

fn open_complete(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    page_intro(
        ui,
        "Restoration complete",
        "The file was authenticated completely before KeyTide restored it.",
    );
    result_paths(app, ui, app.decryption_output.clone(), None, false);
    if app.offer_text_preview && app.decrypted_text.is_some() {
        ui.add_space(14.0);
        if ui.button("Preview sample text").clicked() {
            app.open_secure_viewer();
        }
    }
}

fn result_paths(
    app: &mut KeyTideApp,
    ui: &mut egui::Ui,
    output: Option<std::path::PathBuf>,
    key: Option<std::path::PathBuf>,
    protect: bool,
) {
    egui::Frame::new()
        .fill(theme::surface())
        .stroke(egui::Stroke::new(1.0_f32, theme::border()))
        .corner_radius(8.0)
        .inner_margin(20.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            path_row(ui, "Output", output.as_ref());
            if let Some(key) = key.as_ref() {
                ui.add_space(12.0);
                path_row(ui, "Key", Some(key));
            }
            ui.add_space(20.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Show destination").clicked() {
                    app.reveal_planned_output(output.clone());
                }
                if ui.button("Copy path").clicked()
                    && let Some(path) = output.as_ref()
                {
                    ui.ctx().copy_text(path.display().to_string());
                }
                if ui
                    .button(if protect {
                        "Protect another"
                    } else {
                        "Open another"
                    })
                    .clicked()
                {
                    if protect {
                        app.reset_protect();
                    } else {
                        app.reset_open();
                    }
                }
            });
        });
}

fn path_row(ui: &mut egui::Ui, label: &str, path: Option<&std::path::PathBuf>) {
    ui.label(
        egui::RichText::new(label)
            .small()
            .strong()
            .color(theme::text_muted()),
    );
    ui.add(
        egui::Label::new(
            egui::RichText::new(
                path.map(|path| path.display().to_string())
                    .unwrap_or_else(|| "Path unavailable".to_string()),
            )
            .strong()
            .color(theme::text_primary()),
        )
        .truncate(),
    );
}

fn static_page(ui: &mut egui::Ui, title: &str, body: fn(&mut egui::Ui)) {
    egui::ScrollArea::vertical()
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.add_space(36.0);
            ui.heading(
                egui::RichText::new(title)
                    .size(32.0)
                    .strong()
                    .color(theme::text_primary()),
            );
            ui.add_space(28.0);
            body(ui);
            ui.add_space(40.0);
        });
}

fn security_content(ui: &mut egui::Ui) {
    ui.set_max_width(820.0);
    ui.label(
        egui::RichText::new("What KeyTide protects")
            .size(20.0)
            .strong()
            .color(theme::text_primary()),
    );
    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(
            "KeyTide is designed to encrypt file contents and filename metadata locally. The separate key is required to restore the original.",
        )
        .color(theme::text_secondary()),
    );
    ui.add_space(28.0);
    security_group(
        ui,
        "Container",
        "Versioned format with authenticated metadata and strict parsing.",
    );
    security_group(
        ui,
        "Authentication",
        "Wrong keys, changed files, and corruption fail before plaintext is committed.",
    );
    security_group(
        ui,
        "Key separation",
        "The key file contains the capability to restore content. Store it apart from the container.",
    );
    security_group(
        ui,
        "Limits",
        "KeyTide cannot protect plaintext from malware already controlling your user session.",
    );
    ui.add_space(26.0);
    inline_notice(
        ui,
        "No recovery backdoor",
        "A lost key cannot be recreated by KeyTide.",
        theme::warning(),
    );
    ui.add_space(16.0);
    inline_notice(
        ui,
        "Future key carriers must be attachments",
        "For the planned image and file carrier feature, choose Attach file or Send as document. Do not paste an image inline or send it with a normal photo button. Those methods often compress or rewrite the file, so the received copy will not work as a key.",
        theme::primary(),
    );
}

fn security_group(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.separator();
    ui.add_space(14.0);
    ui.label(
        egui::RichText::new(title)
            .strong()
            .color(theme::text_primary()),
    );
    ui.add_space(5.0);
    ui.label(egui::RichText::new(body).color(theme::text_secondary()));
    ui.add_space(14.0);
}

fn settings_page(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.add_space(36.0);
            ui.label(
                egui::RichText::new("PREFERENCES")
                    .size(11.0)
                    .strong()
                    .color(theme::primary()),
            );
            ui.add_space(8.0);
            ui.heading(
                egui::RichText::new("Settings")
                    .size(32.0)
                    .strong()
                    .color(theme::text_primary()),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Control how KeyTide handles files and local history.")
                    .size(16.0)
                    .color(theme::text_secondary()),
            );
            ui.add_space(28.0);

            let wide = ui.available_width() >= 900.0;
            ui.horizontal_top(|ui| {
                let main_width = if wide {
                    ui.available_width() * 0.64
                } else {
                    ui.available_width()
                };
                ui.allocate_ui_with_layout(
                    egui::vec2(main_width, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| settings_controls(app, ui),
                );
                if wide {
                    ui.add_space(24.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        settings_summary,
                    );
                }
            });
            ui.add_space(32.0);
            ui.separator();
            ui.add_space(18.0);
            ui.label(
                egui::RichText::new("About KeyTide")
                    .size(18.0)
                    .strong()
                    .color(theme::text_primary()),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Created by Rémy Picciano").color(theme::text_secondary()),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Copyright © 2026 Rémy Picciano. All rights reserved.")
                    .small()
                    .color(theme::text_muted()),
            );
        });
}

fn settings_controls(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(theme::surface())
        .stroke(egui::Stroke::new(1.0_f32, theme::border()))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(22, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("OPTIONAL BEHAVIOR")
                    .size(11.0)
                    .strong()
                    .color(theme::primary()),
            );
            setting_row(
                ui,
                &mut app.offer_text_preview,
                "Offer text preview",
                "Show an optional in-memory preview for supported text files.",
            );
            ui.add_space(18.0);
            ui.label(
                egui::RichText::new("BUILT-IN SAFEGUARDS")
                    .size(11.0)
                    .strong()
                    .color(theme::primary()),
            );
            setting_info_row(
                ui,
                "Explicit destinations",
                "Every output location is shown for review before an operation starts.",
            );
            setting_info_row(
                ui,
                "Existing files are protected",
                "KeyTide stops instead of replacing an existing container, key, or restored file.",
            );
            setting_info_row(
                ui,
                "No location history",
                "Recently used folders are not persisted after KeyTide closes.",
            );
        });
}

fn setting_info_row(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(16.0)
                    .strong()
                    .color(theme::text_primary()),
            );
            ui.add_space(7.0);
            ui.label(
                egui::RichText::new(body)
                    .size(13.0)
                    .color(theme::text_muted()),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("Always")
                    .small()
                    .strong()
                    .color(theme::text_muted()),
            );
        });
    });
    ui.add_space(18.0);
    ui.separator();
}

fn settings_summary(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(theme::primary().gamma_multiply(0.08))
        .stroke(egui::Stroke::new(
            1.0_f32,
            theme::primary().gamma_multiply(0.35),
        ))
        .corner_radius(12.0)
        .inner_margin(22.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("PRIVACY BASELINE").size(11.0).strong().color(theme::primary()));
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Secure by design").size(21.0).strong().color(theme::text_primary()));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Cryptographic choices are fixed so files cannot be protected with an unsafe configuration.").color(theme::text_secondary()));
            ui.add_space(20.0);
            for (title, body) in [
                ("Local processing", "Files remain on this device."),
                ("No recovery key", "Only your separate key can restore a file."),
                ("Private defaults", "Replacement and history controls stay explicit."),
            ] {
                ui.label(egui::RichText::new(title).strong().color(theme::text_primary()));
                ui.add_space(3.0);
                ui.label(egui::RichText::new(body).small().color(theme::text_muted()));
                ui.add_space(14.0);
            }
        });
}

fn setting_row(ui: &mut egui::Ui, value: &mut bool, title: &str, body: &str) {
    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(16.0)
                    .strong()
                    .color(theme::text_primary()),
            );
            ui.add_space(7.0);
            ui.label(
                egui::RichText::new(body)
                    .size(13.0)
                    .color(theme::text_muted()),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if settings_toggle(ui, *value).clicked() {
                *value = !*value;
            }
        });
    });
    ui.add_space(18.0);
    ui.separator();
}

fn settings_toggle(ui: &mut egui::Ui, on: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(54.0, 30.0), egui::Sense::click());
    let amount = ui.ctx().animate_bool(response.id, on);
    let fill = if on {
        theme::primary()
    } else {
        theme::surface_raised()
    };
    ui.painter().rect(
        rect,
        15.0,
        fill,
        egui::Stroke::new(
            1.0_f32,
            if on {
                theme::primary()
            } else {
                theme::border()
            },
        ),
        egui::StrokeKind::Inside,
    );
    let x = egui::lerp((rect.left() + 15.0)..=(rect.right() - 15.0), amount);
    ui.painter().circle_filled(
        egui::pos2(x, rect.center().y),
        10.0,
        if on {
            theme::on_primary()
        } else {
            theme::text_muted()
        },
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text(if on { "Turn off" } else { "Turn on" })
}

fn action_bar(app: &mut KeyTideApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let notice_color = match app.notice.kind {
            NoticeKind::Info => theme::text_secondary(),
            NoticeKind::Success => theme::success(),
            NoticeKind::Warning => theme::warning(),
            NoticeKind::Error => theme::danger(),
        };
        ui.add(
            egui::Label::new(
                egui::RichText::new(&app.notice.message)
                    .small()
                    .color(notice_color),
            )
            .truncate(),
        );
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| match app.workflow {
                Workflow::Protect => match app.protect_stage {
                    ProtectStage::SelectFile => {
                        if widgets::primary_button(ui, "Review", app.can_protect()).clicked() {
                            app.review_protect();
                        }
                    }
                    ProtectStage::Review => {
                        if widgets::primary_button(ui, "Protect", app.can_run_protect()).clicked() {
                            app.run_protect();
                        }
                        if ui.button("Back").clicked() {
                            app.back();
                        }
                    }
                    ProtectStage::Processing | ProtectStage::Complete => {}
                },
                Workflow::Open => match app.open_stage {
                    OpenStage::SelectContainer => {}
                    OpenStage::SelectKey => {
                        if widgets::primary_button(ui, "Review", app.can_open()).clicked() {
                            app.review_open();
                        }
                        if ui.button("Change file").clicked() {
                            app.clear_encrypted_file();
                        }
                    }
                    OpenStage::Review => {
                        if widgets::primary_button(ui, "Restore", app.can_run_open()).clicked() {
                            app.run_open();
                        }
                        if ui.button("Back").clicked() {
                            app.back();
                        }
                    }
                    OpenStage::Processing | OpenStage::Complete => {}
                },
                Workflow::Security | Workflow::Settings => {}
            },
        );
    });
}

fn protect_stage_index(stage: ProtectStage) -> usize {
    match stage {
        ProtectStage::SelectFile => 0,
        ProtectStage::Review => 1,
        ProtectStage::Processing => 2,
        ProtectStage::Complete => 3,
    }
}

fn open_stage_index(stage: OpenStage) -> usize {
    match stage {
        OpenStage::SelectContainer => 0,
        OpenStage::SelectKey => 1,
        OpenStage::Review => 2,
        OpenStage::Processing => 3,
        OpenStage::Complete => 4,
    }
}
