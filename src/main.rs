mod app;
mod coffer;
mod ui;

fn main() {
    init_logging();
    tracing::info!("KeyTide starting");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 820.0])
            .with_min_inner_size([760.0, 600.0])
            .with_resizable(true),

        ..Default::default()
    };

    let result = eframe::run_native(
        "KeyTide",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;
            egui_extras::install_image_loaders(ctx);

            ui::theme::apply_visuals(ctx, true);

            let mut style = (*ctx.global_style()).clone();

            style.spacing.item_spacing = egui::Vec2::new(10.0, 10.0);

            style.spacing.button_padding = egui::Vec2::new(16.0, 10.0);

            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(15.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
            );

            ctx.set_global_style(style);

            Ok(Box::new(app::KeyTideApp::default()))
        }),
    );
    match result {
        Ok(()) => tracing::info!("KeyTide closed normally"),
        Err(error) => {
            tracing::error!(error = %error, "KeyTide stopped unexpectedly");
            std::process::exit(1);
        }
    }
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("keytide=info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(false)
        .try_init();
}
