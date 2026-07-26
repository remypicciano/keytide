use eframe::egui;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, TryRecvError},
};
use std::time::Duration;

use crate::coffer::{
    KeyTideError, ProtectRequest, ProtectResult, RestoreRequest, RestoreResult, protect_file,
    restore_file,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Workflow {
    Protect,
    #[default]
    Open,
    Security,
    Settings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProtectStage {
    #[default]
    SelectFile,
    Review,
    Processing,
    Complete,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpenStage {
    #[default]
    SelectContainer,
    SelectKey,
    Review,
    Processing,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Notice {
    pub kind: NoticeKind,
    pub message: String,
}

impl Notice {
    fn new(kind: NoticeKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectedFile {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub size_bytes: u64,
}

impl SelectedFile {
    pub fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown file")
            .to_string();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("No extension")
            .to_string();
        let size_bytes = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        Self {
            path,
            name,
            extension,
            size_bytes,
        }
    }

    pub fn readable_size(&self) -> String {
        format_file_size(self.size_bytes)
    }
}

pub struct KeyTideApp {
    pub theme_mode: ThemeMode,
    pub show_splash: bool,
    pub splash_started_at: Option<f64>,
    pub workflow: Workflow,
    pub protect_stage: ProtectStage,
    pub open_stage: OpenStage,
    pub source_file: Option<SelectedFile>,
    pub encrypted_file: Option<SelectedFile>,
    pub key_file: Option<SelectedFile>,
    pub notice: Notice,
    pub progress: f32,
    pub show_error: bool,
    pub error_message: String,
    pub show_viewer: bool,
    pub decrypted_text: Option<String>,
    pub main_window_size_before_viewer: Option<egui::Vec2>,
    pub viewer_was_open: bool,
    pub encryption_output: Option<PathBuf>,
    pub key_output: Option<PathBuf>,
    pub decryption_output: Option<PathBuf>,
    pub protect_destination: Option<PathBuf>,
    pub restore_destination: Option<PathBuf>,
    pub protected_filename: String,
    pub restored_filename: String,
    pub processing_started_at: Option<f64>,
    pub offer_text_preview: bool,
    operation_receiver: Option<Receiver<OperationOutcome>>,
    operation_cancel: Option<OperationControl>,
}

enum OperationOutcome {
    Protected(Result<ProtectResult, KeyTideError>),
    Restored(Result<RestoreResult, KeyTideError>),
}

struct OperationControl {
    cancelled: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Drop for OperationControl {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Default for KeyTideApp {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Light,
            show_splash: true,
            splash_started_at: None,
            workflow: Workflow::Open,
            protect_stage: ProtectStage::SelectFile,
            open_stage: OpenStage::SelectContainer,
            source_file: None,
            encrypted_file: None,
            key_file: None,
            notice: Notice::new(NoticeKind::Info, "Ready"),
            progress: 0.0,
            show_error: false,
            error_message: String::new(),
            show_viewer: false,
            decrypted_text: None,
            main_window_size_before_viewer: None,
            viewer_was_open: false,
            encryption_output: None,
            key_output: None,
            decryption_output: None,
            protect_destination: None,
            restore_destination: None,
            protected_filename: String::new(),
            restored_filename: String::new(),
            processing_started_at: None,
            offer_text_preview: true,
            operation_receiver: None,
            operation_cancel: None,
        }
    }
}

impl KeyTideApp {
    pub fn dismiss_splash(&mut self) {
        self.show_splash = false;
        self.splash_started_at = None;
    }

    pub fn splash_opacity(&self, now: f64) -> f32 {
        let Some(started) = self.splash_started_at else {
            return 0.0;
        };
        let elapsed = now - started;
        if elapsed < 1.8 {
            smoothstep((elapsed / 1.8) as f32)
        } else if elapsed < 3.0 {
            1.0
        } else {
            1.0 - smoothstep(((elapsed - 3.0) / 0.8) as f32)
        }
    }

    pub fn splash_progress(&self, now: f64) -> f32 {
        self.splash_started_at
            .map(|started| ((now - started) / 3.8) as f32)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0)
    }

    fn update_splash(&mut self, ctx: &egui::Context) {
        if !self.show_splash {
            return;
        }
        let now = ctx.input(|input| input.time);
        let started = *self.splash_started_at.get_or_insert(now);
        if now - started >= 3.8 {
            self.dismiss_splash();
        } else {
            ctx.request_repaint();
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
        self.set_notice(
            NoticeKind::Success,
            match mode {
                ThemeMode::Light => "Light appearance enabled",
                ThemeMode::Dark => "Dark appearance enabled",
            },
        );
    }

    pub fn navigate(&mut self, workflow: Workflow) {
        self.workflow = workflow;
        let kind = match workflow {
            Workflow::Security | Workflow::Settings => NoticeKind::Success,
            Workflow::Protect | Workflow::Open => NoticeKind::Info,
        };
        self.set_notice(
            kind,
            match workflow {
                Workflow::Protect => "Choose a file to protect",
                Workflow::Open => "Choose a protected file",
                Workflow::Security | Workflow::Settings => "Local | Offline | Private",
            },
        );
    }

    pub fn reset_protect(&mut self) {
        self.protect_stage = ProtectStage::SelectFile;
        self.source_file = None;
        self.encryption_output = None;
        self.key_output = None;
        self.progress = 0.0;
        self.protect_destination = None;
        self.protected_filename.clear();
        self.processing_started_at = None;
        self.set_notice(NoticeKind::Info, "Choose a file to protect");
    }

    pub fn reset_open(&mut self) {
        self.open_stage = OpenStage::SelectContainer;
        self.encrypted_file = None;
        self.key_file = None;
        self.decryption_output = None;
        self.decrypted_text = None;
        self.progress = 0.0;
        self.restore_destination = None;
        self.restored_filename.clear();
        self.processing_started_at = None;
        self.set_notice(NoticeKind::Info, "Choose a protected file");
    }

    pub fn select_source_file(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() {
            self.source_file = Some(SelectedFile::from_path(path));
            self.prepare_protect_output();
            self.protect_stage = ProtectStage::SelectFile;
            self.set_notice(NoticeKind::Info, "File selected - continue to review");
        }
    }

    pub fn select_encrypted_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("KeyTide protected files", &["coffer"])
            .pick_file()
        {
            self.encrypted_file = Some(SelectedFile::from_path(path));
            self.prepare_restore_output();
            self.open_stage = OpenStage::SelectKey;
            self.set_notice(NoticeKind::Info, "Now choose the matching key");
        }
    }

    pub fn select_key(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("KeyTide keys", &["cofferkey"])
            .pick_file()
        {
            self.key_file = Some(SelectedFile::from_path(path));
            self.open_stage = OpenStage::SelectKey;
            self.set_notice(NoticeKind::Info, "Key selected - continue to review");
        }
    }

    pub fn clear_source_file(&mut self) {
        self.source_file = None;
        self.encryption_output = None;
        self.key_output = None;
        self.protect_destination = None;
        self.protected_filename.clear();
        self.protect_stage = ProtectStage::SelectFile;
        self.set_notice(NoticeKind::Info, "Choose a file to protect");
    }

    pub fn clear_encrypted_file(&mut self) {
        self.encrypted_file = None;
        self.decryption_output = None;
        self.restore_destination = None;
        self.restored_filename.clear();
        self.open_stage = OpenStage::SelectContainer;
        self.set_notice(NoticeKind::Info, "Choose a protected file");
    }

    pub fn clear_key_file(&mut self) {
        self.key_file = None;
        self.decryption_output = None;
        self.open_stage = if self.encrypted_file.is_some() {
            OpenStage::SelectKey
        } else {
            OpenStage::SelectContainer
        };
        self.set_notice(NoticeKind::Info, "Choose the matching key");
    }

    pub fn review_protect(&mut self) {
        if self.can_protect() {
            self.prepare_protect_output();
            self.protect_stage = ProtectStage::Review;
            self.set_notice(
                NoticeKind::Info,
                "Review where your protected files will be saved",
            );
        }
    }

    pub fn review_open(&mut self) {
        if self.can_open() {
            self.prepare_restore_output();
            self.open_stage = OpenStage::Review;
            self.set_notice(NoticeKind::Info, "Review before restoring the file");
        }
    }

    pub fn back(&mut self) {
        match self.workflow {
            Workflow::Protect => self.protect_stage = ProtectStage::SelectFile,
            Workflow::Open => {
                self.open_stage = if self.encrypted_file.is_some() {
                    OpenStage::SelectKey
                } else {
                    OpenStage::SelectContainer
                }
            }
            Workflow::Security | Workflow::Settings => {}
        }
        self.set_notice(NoticeKind::Info, "Selections preserved");
    }

    pub fn can_protect(&self) -> bool {
        self.source_file.is_some()
    }

    pub fn can_open(&self) -> bool {
        self.encrypted_file.is_some() && self.key_file.is_some()
    }

    pub fn can_run_protect(&self) -> bool {
        self.can_protect()
            && self.protect_destination.is_some()
            && valid_output_name(&self.protected_filename, "coffer")
    }

    pub fn can_run_open(&self) -> bool {
        self.can_open()
            && self.restore_destination.is_some()
            && valid_output_name(&self.restored_filename, "")
    }

    pub fn choose_protect_destination(&mut self) {
        if let Some(path) = FileDialog::new().pick_folder() {
            self.protect_destination = Some(path);
            self.refresh_planned_outputs();
            self.set_notice(NoticeKind::Info, "Protected output destination updated");
        }
    }

    pub fn choose_restore_destination(&mut self) {
        if let Some(path) = FileDialog::new().pick_folder() {
            self.restore_destination = Some(path);
            self.refresh_planned_outputs();
            self.set_notice(NoticeKind::Info, "Restore destination updated");
        }
    }

    pub fn reveal_planned_output(&mut self, output: Option<PathBuf>) {
        let Some(output) = output else {
            self.show_error("Choose an output destination first.");
            return;
        };
        if let Err(error) = reveal_in_file_manager(&output) {
            self.show_error(format!("Could not open the destination folder: {error}"));
        }
    }

    pub fn run_protect(&mut self) {
        if self.operation_receiver.is_some() {
            self.show_error("Wait for the current file operation to finish or cancel it first.");
            return;
        }
        self.prepare_protect_output();
        if !self.can_run_protect() {
            self.show_error("Choose a destination and a valid .coffer filename before continuing.");
            return;
        }

        let Some(source) = self.source_file.as_ref().map(|file| file.path.clone()) else {
            return;
        };
        let Some(container_output) = self.encryption_output.clone() else {
            return;
        };
        let Some(key_output) = self.key_output.clone() else {
            return;
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let result = protect_file(ProtectRequest {
                source: &source,
                container_output: &container_output,
                key_output: &key_output,
                cancelled: Some(&worker_cancelled),
            });
            let _ = sender.send(OperationOutcome::Protected(result));
        });

        self.operation_receiver = Some(receiver);
        self.operation_cancel = Some(OperationControl {
            cancelled,
            worker: Some(worker),
        });
        self.protect_stage = ProtectStage::Processing;
        self.progress = 0.0;
        self.processing_started_at = None;
        self.refresh_planned_outputs();
        self.set_notice(NoticeKind::Info, "Protecting file locally");
    }

    pub fn run_open(&mut self) {
        if self.operation_receiver.is_some() {
            self.show_error("Wait for the current file operation to finish or cancel it first.");
            return;
        }
        self.prepare_restore_output();
        if !self.can_run_open() {
            self.show_error("Choose both files, a destination, and a valid restored filename.");
            return;
        }

        let Some(container) = self.encrypted_file.as_ref().map(|file| file.path.clone()) else {
            return;
        };
        let Some(key) = self.key_file.as_ref().map(|file| file.path.clone()) else {
            return;
        };
        let Some(output) = self.decryption_output.clone() else {
            return;
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let result = restore_file(RestoreRequest {
                container: &container,
                key: &key,
                output: &output,
                cancelled: Some(&worker_cancelled),
            });
            let _ = sender.send(OperationOutcome::Restored(result));
        });

        self.operation_receiver = Some(receiver);
        self.operation_cancel = Some(OperationControl {
            cancelled,
            worker: Some(worker),
        });
        self.open_stage = OpenStage::Processing;
        self.progress = 0.0;
        self.processing_started_at = None;
        self.refresh_planned_outputs();
        self.set_notice(NoticeKind::Info, "Authenticating before restoration");
    }

    pub fn cancel_processing(&mut self) {
        self.operation_cancel.take();
        self.operation_receiver = None;
        self.processing_started_at = None;
        self.progress = 0.0;
        if self.protect_stage == ProtectStage::Processing {
            self.protect_stage = ProtectStage::Review;
        }
        if self.open_stage == OpenStage::Processing {
            self.open_stage = OpenStage::Review;
        }
        self.set_notice(
            NoticeKind::Warning,
            "Cancelling and removing incomplete output",
        );
    }

    fn prepare_protect_output(&mut self) {
        let Some(file) = self.source_file.as_ref() else {
            return;
        };
        if self.protect_destination.is_none() {
            self.protect_destination = file.path.parent().map(PathBuf::from);
        }
        if self.protected_filename.is_empty() {
            self.protected_filename = format!("{}.coffer", file.name);
        }
        self.refresh_planned_outputs();
    }

    fn prepare_restore_output(&mut self) {
        let Some(file) = self.encrypted_file.as_ref() else {
            return;
        };
        if self.restore_destination.is_none() {
            self.restore_destination = file.path.parent().map(PathBuf::from);
        }
        if self.restored_filename.is_empty() {
            self.restored_filename = file
                .name
                .strip_suffix(".coffer")
                .unwrap_or("Restored file")
                .to_string();
        }
        self.refresh_planned_outputs();
    }

    pub fn refresh_planned_outputs(&mut self) {
        self.encryption_output = self
            .protect_destination
            .as_ref()
            .map(|path| path.join(&self.protected_filename));
        self.key_output = self.protect_destination.as_ref().map(|path| {
            let stem = self
                .protected_filename
                .strip_suffix(".coffer")
                .unwrap_or(&self.protected_filename);
            path.join(format!("{stem}.cofferkey"))
        });
        self.decryption_output = self
            .restore_destination
            .as_ref()
            .map(|path| path.join(&self.restored_filename));
    }

    fn update_processing(&mut self, ctx: &egui::Context) {
        let processing = self.protect_stage == ProtectStage::Processing
            || self.open_stage == OpenStage::Processing;
        if !processing {
            return;
        }
        let outcome =
            self.operation_receiver
                .as_ref()
                .and_then(|receiver| match receiver.try_recv() {
                    Ok(outcome) => Some(Ok(outcome)),
                    Err(TryRecvError::Disconnected) => Some(Err(())),
                    Err(TryRecvError::Empty) => None,
                });
        match outcome {
            Some(Ok(outcome)) => {
                self.operation_receiver = None;
                self.operation_cancel = None;
                self.processing_started_at = None;
                self.progress = 1.0;
                self.finish_operation(outcome);
            }
            Some(Err(())) => {
                self.operation_receiver = None;
                self.operation_cancel = None;
                self.processing_started_at = None;
                self.progress = 0.0;
                if self.protect_stage == ProtectStage::Processing {
                    self.protect_stage = ProtectStage::Review;
                }
                if self.open_stage == OpenStage::Processing {
                    self.open_stage = OpenStage::Review;
                }
                self.show_error(
                    "The background operation stopped unexpectedly. No incomplete output was kept.",
                );
            }
            None => {
                let now = ctx.input(|input| input.time);
                let started = *self.processing_started_at.get_or_insert(now);
                self.progress = (0.15 + ((now - started) as f32 * 0.08)).min(0.9);
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }
    }

    fn finish_operation(&mut self, outcome: OperationOutcome) {
        match outcome {
            OperationOutcome::Protected(Ok(result)) => {
                self.encryption_output = Some(result.container);
                self.key_output = Some(result.key);
                self.protect_stage = ProtectStage::Complete;
                self.set_notice(NoticeKind::Success, "File protected successfully");
            }
            OperationOutcome::Restored(Ok(result)) => {
                self.decryption_output = Some(result.output.clone());
                self.decrypted_text = if self.offer_text_preview {
                    fs::metadata(&result.output)
                        .ok()
                        .filter(|metadata| metadata.len() <= 1_048_576)
                        .and_then(|_| fs::read_to_string(&result.output).ok())
                } else {
                    None
                };
                self.open_stage = OpenStage::Complete;
                self.set_notice(
                    NoticeKind::Success,
                    format!("Restored {} successfully", result.original_filename),
                );
            }
            OperationOutcome::Protected(Err(error)) => {
                self.protect_stage = ProtectStage::Review;
                self.show_error(error.user_message());
            }
            OperationOutcome::Restored(Err(error)) => {
                self.open_stage = OpenStage::Review;
                self.show_error(error.user_message());
            }
        }
    }

    pub fn open_secure_viewer(&mut self) {
        if self.decrypted_text.is_some() {
            self.show_viewer = true;
        }
    }

    pub fn close_secure_viewer(&mut self) {
        self.show_viewer = false;
        self.viewer_was_open = false;
        self.decrypted_text = None;
    }

    pub fn show_error(&mut self, message: impl Into<String>) {
        self.error_message = message.into();
        self.show_error = true;
        self.set_notice(NoticeKind::Error, "Action required");
    }

    pub fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped_files = ctx.input(|input| input.raw.dropped_files.clone());
        for dropped in dropped_files {
            let Some(path) = dropped.path else { continue };
            match self.workflow {
                Workflow::Protect => {
                    self.source_file = Some(SelectedFile::from_path(path));
                    self.prepare_protect_output();
                    self.protect_stage = ProtectStage::SelectFile;
                    self.set_notice(NoticeKind::Info, "File selected - continue to review");
                }
                Workflow::Open => assign_open_drop(self, path),
                Workflow::Security | Workflow::Settings => {}
            }
        }
    }

    fn set_notice(&mut self, kind: NoticeKind, message: impl Into<String>) {
        self.notice = Notice::new(kind, message);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let command = egui::Modifiers::COMMAND;
        if ctx.input_mut(|input| input.consume_key(command, egui::Key::E)) {
            self.navigate(Workflow::Protect);
        }
        if ctx.input_mut(|input| input.consume_key(command, egui::Key::D)) {
            self.navigate(Workflow::Open);
        }
    }
}

impl eframe::App for KeyTideApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        crate::ui::theme::apply_visuals(&ctx, self.theme_mode == ThemeMode::Light);
        self.update_splash(&ctx);
        if self.show_splash {
            crate::ui::home::show_splash(self, ui);
            return;
        }
        self.handle_shortcuts(&ctx);
        self.handle_dropped_files(&ctx);
        self.update_processing(&ctx);
        crate::ui::home::show(self, ui);
        crate::ui::dialogs::show(self, &ctx);
        crate::ui::secure_viewer::show(self, &ctx);
    }
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn assign_open_drop(app: &mut KeyTideApp, path: PathBuf) {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "coffer" => {
            app.encrypted_file = Some(SelectedFile::from_path(path));
            app.prepare_restore_output();
            app.open_stage = OpenStage::SelectKey;
            app.set_notice(NoticeKind::Info, "Protected file selected");
        }
        "cofferkey" => {
            app.key_file = Some(SelectedFile::from_path(path));
            app.open_stage = if app.encrypted_file.is_some() {
                OpenStage::SelectKey
            } else {
                OpenStage::SelectContainer
            };
            app.set_notice(NoticeKind::Info, "Key selected");
        }
        _ => app.show_error("Drop a .coffer file or a supported .cofferkey file."),
    }
}

fn valid_output_name(name: &str, required_extension: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && trimmed != "."
        && trimmed != ".."
        && !trimmed.contains(['/', '\\', '\0'])
        && (required_extension.is_empty()
            || trimmed
                .to_ascii_lowercase()
                .ends_with(&format!(".{required_extension}")))
}

fn reveal_in_file_manager(path: &std::path::Path) -> std::io::Result<()> {
    let destination = if path.exists() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        if path.exists() && path.is_file() {
            command.arg("-R").arg(path);
        } else {
            command.arg(destination);
        }
        command.spawn()?.wait()?;
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");
        if path.exists() && path.is_file() {
            command.arg(format!("/select,{}", path.display()));
        } else {
            command.arg(destination);
        }
        command.spawn()?.wait()?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(destination).spawn()?.wait()?;
    }

    Ok(())
}

fn format_file_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let size = size as f64;
    if size >= GB {
        format!("{:.2} GB", size / GB)
    } else if size >= MB {
        format!("{:.2} MB", size / MB)
    } else if size >= KB {
        format!("{:.1} KB", size / KB)
    } else {
        format!("{} bytes", size as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn selected(name: &str) -> SelectedFile {
        SelectedFile {
            path: PathBuf::from(name),
            name: name.to_string(),
            extension: name
                .rsplit_once('.')
                .map(|(_, ext)| ext)
                .unwrap_or("")
                .to_string(),
            size_bytes: 42,
        }
    }

    fn wait_for_operation(app: &mut KeyTideApp) {
        let context = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.operation_receiver.is_some() && Instant::now() < deadline {
            app.update_processing(&context);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            app.operation_receiver.is_none(),
            "background operation timed out"
        );
    }

    #[test]
    fn navigation_preserves_selected_files() {
        let mut app = KeyTideApp {
            source_file: Some(selected("notes.txt")),
            ..Default::default()
        };
        app.navigate(Workflow::Settings);
        app.navigate(Workflow::Protect);
        assert!(app.source_file.is_some());
    }

    #[test]
    fn protect_requires_explicit_review_transition() {
        let mut app = KeyTideApp {
            source_file: Some(selected("notes.txt")),
            ..Default::default()
        };
        assert_eq!(app.protect_stage, ProtectStage::SelectFile);
        app.review_protect();
        assert_eq!(app.protect_stage, ProtectStage::Review);
    }

    #[test]
    fn open_requires_both_files_before_review() {
        let mut app = KeyTideApp {
            encrypted_file: Some(selected("notes.coffer")),
            ..Default::default()
        };
        app.review_open();
        assert_ne!(app.open_stage, OpenStage::Review);
        app.key_file = Some(selected("notes.cofferkey"));
        app.review_open();
        assert_eq!(app.open_stage, OpenStage::Review);
    }

    #[test]
    fn notices_have_explicit_severity() {
        let notice = Notice::new(NoticeKind::Success, "Complete");
        assert_eq!(notice.kind, NoticeKind::Success);
    }

    #[test]
    fn protect_output_requires_safe_coffer_filename() {
        let mut app = KeyTideApp {
            source_file: Some(selected("notes.txt")),
            ..Default::default()
        };
        app.review_protect();
        assert!(app.can_run_protect());

        app.protected_filename = "../notes.coffer".to_string();
        assert!(!app.can_run_protect());
        app.protected_filename = "notes.txt".to_string();
        assert!(!app.can_run_protect());
    }

    #[test]
    fn cancelling_processing_returns_to_review() {
        let mut app = KeyTideApp {
            workflow: Workflow::Protect,
            source_file: Some(selected("notes.txt")),
            ..Default::default()
        };
        app.run_protect();
        assert_eq!(app.protect_stage, ProtectStage::Processing);
        app.cancel_processing();
        assert_eq!(app.protect_stage, ProtectStage::Review);
        assert_eq!(app.progress, 0.0);
    }

    #[test]
    fn splash_fades_in_holds_and_fades_out() {
        let app = KeyTideApp {
            splash_started_at: Some(10.0),
            ..Default::default()
        };
        assert_eq!(app.splash_opacity(10.0), 0.0);
        assert!(app.splash_opacity(10.9) > 0.0);
        assert_eq!(app.splash_opacity(11.8), 1.0);
        assert_eq!(app.splash_opacity(13.0), 1.0);
        assert!(app.splash_opacity(13.4) < 1.0);
    }

    #[test]
    fn interface_workflow_runs_real_protection_and_restoration() {
        let directory = tempfile::tempdir().unwrap();
        let source_path = directory.path().join("notes.txt");
        fs::write(&source_path, "private notes").unwrap();
        let mut app = KeyTideApp {
            workflow: Workflow::Protect,
            source_file: Some(SelectedFile::from_path(source_path)),
            protect_destination: Some(directory.path().to_path_buf()),
            protected_filename: "notes.coffer".to_string(),
            ..Default::default()
        };
        app.run_protect();
        wait_for_operation(&mut app);
        assert_eq!(app.protect_stage, ProtectStage::Complete);
        let container = app.encryption_output.clone().unwrap();
        let key = app.key_output.clone().unwrap();
        assert_eq!(key.file_name().unwrap(), "notes.cofferkey");

        let restored_path = directory.path().join("restored.txt");
        app.workflow = Workflow::Open;
        app.encrypted_file = Some(SelectedFile::from_path(container));
        app.key_file = Some(SelectedFile::from_path(key));
        app.restore_destination = Some(directory.path().to_path_buf());
        app.restored_filename = "restored.txt".to_string();
        app.run_open();
        wait_for_operation(&mut app);
        assert_eq!(app.open_stage, OpenStage::Complete);
        assert_eq!(fs::read_to_string(restored_path).unwrap(), "private notes");
    }
}
