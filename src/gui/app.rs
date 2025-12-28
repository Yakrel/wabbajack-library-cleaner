// Copyright (C) 2025 Berkay Yetgin
// GPL-3.0 License

//! Single-page GUI for Wabbajack Library Cleaner

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use eframe::egui;
use egui::{Color32, RichText, Rounding, Vec2};

use crate::core::{
    calculate_library_stats, delete_old_versions, delete_orphaned_mods, detect_orphaned_mods,
    find_wabbajack_files, format_size, get_all_mod_files, get_game_folders, parse_wabbajack_file,
    scan_folder_for_duplicates, DeletionResult, LibraryStats, ModlistInfo, OldVersionScanResult,
    ScanResult,
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Colors
const COLOR_BG_MAIN: Color32 = Color32::from_rgb(30, 30, 35);
const COLOR_BG_CARD: Color32 = Color32::from_rgb(42, 42, 50);
const COLOR_BG_HEADER: Color32 = Color32::from_rgb(25, 25, 30);
const COLOR_ACCENT: Color32 = Color32::from_rgb(99, 102, 241);
const COLOR_SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
const COLOR_WARNING: Color32 = Color32::from_rgb(249, 115, 22); // Orange
const COLOR_DANGER: Color32 = Color32::from_rgb(239, 68, 68);
const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 245, 250);
const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175);
const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);

#[derive(Debug)]
enum AsyncMessage {
    ModlistsParsed(Vec<ModlistInfo>),
    GameFoldersFound(Vec<PathBuf>),
    OrphanedScanComplete(ScanResult),
    OldVersionScanComplete(OldVersionScanResult),
    DeletionComplete(DeletionResult),
    StatsComplete(LibraryStats),
    Progress(String, Option<(usize, usize)>),
    Error(String),
}

#[derive(PartialEq, Clone, Copy)]
enum Modal {
    None,
    About,
    FolderSelect,
}

#[derive(Clone, Copy, PartialEq)]
enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct WabbajackCleanerApp {
    wabbajack_dir: Option<PathBuf>,
    downloads_dir: Option<PathBuf>,
    modlists: Vec<ModlistInfo>,
    modlist_selected: Vec<bool>,
    game_folders: Vec<PathBuf>,
    selected_game_folder: Option<usize>,
    move_to_backup: bool,
    pending_delete_mode: bool,
    tx: Sender<AsyncMessage>,
    rx: Receiver<AsyncMessage>,
    is_loading: bool,
    current_operation: String,
    progress: Option<(usize, usize)>,
    stats: Option<LibraryStats>,
    orphaned_result: Option<ScanResult>,
    old_version_result: Option<OldVersionScanResult>,
    log_messages: Vec<(String, LogLevel)>,
    modal: Modal,
}

impl Default for WabbajackCleanerApp {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            wabbajack_dir: None,
            downloads_dir: None,
            modlists: Vec::new(),
            modlist_selected: Vec::new(),
            game_folders: Vec::new(),
            selected_game_folder: None,
            move_to_backup: true,
            pending_delete_mode: false,
            tx,
            rx,
            is_loading: false,
            current_operation: String::new(),
            progress: None,
            stats: None,
            orphaned_result: None,
            old_version_result: None,
            log_messages: Vec::new(),
            modal: Modal::None,
        }
    }
}

impl WabbajackCleanerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_rounding = Rounding::same(8.0);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
        style.visuals.widgets.active.rounding = Rounding::same(6.0);
        style.visuals.window_fill = COLOR_BG_MAIN;
        style.visuals.panel_fill = COLOR_BG_MAIN;
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        cc.egui_ctx.set_style(style);
        Self::default()
    }

    fn log(&mut self, level: LogLevel, msg: &str) {
        let time = chrono::Local::now().format("%H:%M:%S");
        self.log_messages
            .push((format!("[{}] {}", time, msg), level));
        if self.log_messages.len() > 500 {
            self.log_messages.remove(0);
        }
    }

    fn is_ready(&self) -> bool {
        self.wabbajack_dir.is_some() && self.downloads_dir.is_some()
    }

    fn selected_modlist_count(&self) -> usize {
        self.modlist_selected.iter().filter(|&&x| x).count()
    }

    fn get_backup_path(&self) -> Option<PathBuf> {
        if !self.move_to_backup {
            return None;
        }
        self.downloads_dir.as_ref().map(|dir| {
            let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
            dir.join("WLC_Backup").join(ts.to_string())
        })
    }

    fn select_wabbajack_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Wabbajack Installation Folder")
            .pick_folder()
        {
            self.wabbajack_dir = Some(path.clone());
            self.log(LogLevel::Info, "Scanning Wabbajack folder...");
            self.is_loading = true;
            self.current_operation = "Scanning for modlists...".to_string();
            let tx = self.tx.clone();
            thread::spawn(move || scan_wabbajack_dir(path, tx));
        }
    }

    fn select_downloads_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Downloads Folder")
            .pick_folder()
        {
            self.downloads_dir = Some(path.clone());
            self.log(LogLevel::Info, "Indexing downloads folder...");
            let tx = self.tx.clone();
            thread::spawn(move || match get_game_folders(&path) {
                Ok(folders) => {
                    tx.send(AsyncMessage::GameFoldersFound(folders)).ok();
                }
                Err(e) => {
                    tx.send(AsyncMessage::Error(e.to_string())).ok();
                }
            });
        }
    }

    fn run_analysis(&mut self) {
        if !self.is_ready() {
            return;
        }
        self.is_loading = true;
        self.current_operation = "Calculating statistics...".to_string();
        let folders = self.game_folders.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let stats = calculate_library_stats(&folders);
            tx.send(AsyncMessage::StatsComplete(stats)).ok();
        });
    }

    fn run_orphaned_scan(&mut self, delete: bool) {
        let selected: Vec<ModlistInfo> = self
            .modlists
            .iter()
            .enumerate()
            .filter(|(i, _)| self.modlist_selected.get(*i).copied().unwrap_or(false))
            .map(|(_, ml)| ml.clone())
            .collect();

        if selected.is_empty() {
            self.log(LogLevel::Warning, "Please select at least one modlist!");
            return;
        }

        self.is_loading = true;
        self.current_operation = if delete {
            "Cleaning orphaned mods..."
        } else {
            "Scanning for orphaned mods..."
        }
        .to_string();
        let path = self.downloads_dir.clone().unwrap();
        let backup = if delete { self.get_backup_path() } else { None };
        let tx = self.tx.clone();
        thread::spawn(move || scan_orphaned_mods_async(path, selected, delete, backup, tx));
    }

    fn run_old_version_scan(&mut self, delete: bool) {
        if self.game_folders.is_empty() {
            self.log(LogLevel::Warning, "No game folders found.");
            return;
        }
        self.pending_delete_mode = delete;
        self.modal = Modal::FolderSelect;
    }

    fn start_old_version_scan(&mut self) {
        if let Some(idx) = self.selected_game_folder {
            let folder = self.game_folders[idx].clone();
            let delete = self.pending_delete_mode;
            let backup = if delete { self.get_backup_path() } else { None };
            let tx = self.tx.clone();
            self.modal = Modal::None;
            self.is_loading = true;
            self.current_operation = "Scanning for old versions...".to_string();
            thread::spawn(move || scan_old_versions_async(folder, delete, backup, tx));
        }
    }

    fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMessage::ModlistsParsed(list) => {
                    self.log(LogLevel::Info, &format!("Found {} modlists", list.len()));
                    self.modlist_selected = vec![true; list.len()];
                    self.modlists = list;
                    self.is_loading = false;
                    self.progress = None;
                    if self.downloads_dir.is_some() {
                        self.run_analysis();
                    }
                }
                AsyncMessage::GameFoldersFound(folders) => {
                    self.log(
                        LogLevel::Info,
                        &format!("Found {} game folders", folders.len()),
                    );
                    self.game_folders = folders;
                    self.progress = None;
                    if self.wabbajack_dir.is_some() {
                        self.run_analysis();
                    }
                }
                AsyncMessage::StatsComplete(stats) => {
                    self.stats = Some(stats);
                    self.is_loading = false;
                    self.progress = None;
                }
                AsyncMessage::OrphanedScanComplete(res) => {
                    self.log(
                        LogLevel::Info,
                        &format!(
                            "Found {} orphaned files ({})",
                            res.orphaned_mods.len(),
                            format_size(res.orphaned_size)
                        ),
                    );
                    self.orphaned_result = Some(res);
                    self.is_loading = false;
                    self.progress = None;
                }
                AsyncMessage::OldVersionScanComplete(res) => {
                    self.log(
                        LogLevel::Info,
                        &format!(
                            "Found {} old versions ({})",
                            res.total_files,
                            format_size(res.total_space)
                        ),
                    );
                    self.old_version_result = Some(res);
                    self.is_loading = false;
                    self.progress = None;
                }
                AsyncMessage::DeletionComplete(res) => {
                    self.log(
                        LogLevel::Info,
                        &format!(
                            "Cleaned {} files, freed {}",
                            res.deleted_count,
                            format_size(res.space_freed)
                        ),
                    );
                    self.is_loading = false;
                    self.progress = None;
                    self.run_analysis();
                }
                AsyncMessage::Progress(s, prog) => {
                    self.current_operation = s;
                    self.progress = prog;
                }
                AsyncMessage::Error(e) => {
                    self.log(LogLevel::Error, &format!("Error: {}", e));
                    self.is_loading = false;
                    self.progress = None;
                }
            }
        }
    }
}

impl eframe::App for WabbajackCleanerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();
        if self.is_loading {
            ctx.request_repaint();
        }

        // Header
        egui::TopBottomPanel::top("header")
            .exact_height(50.0)
            .frame(
                egui::Frame::none()
                    .fill(COLOR_BG_HEADER)
                    .inner_margin(egui::vec2(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Wabbajack Library Cleaner")
                            .size(18.0)
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(format!("v{}", APP_VERSION))
                            .size(12.0)
                            .color(COLOR_TEXT_MUTED),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("About").clicked() {
                            self.modal = Modal::About;
                        }
                        ui.add_space(16.0);
                        ui.checkbox(&mut self.move_to_backup, "Backup mode");
                    });
                });
            });

        // Log panel
        egui::TopBottomPanel::bottom("log_panel")
            .resizable(false)
            .exact_height(120.0)
            .frame(
                egui::Frame::none()
                    .fill(COLOR_BG_HEADER)
                    .inner_margin(egui::vec2(12.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    if self.is_loading {
                        ui.spinner();
                        ui.label(
                            RichText::new(&self.current_operation).color(COLOR_TEXT_SECONDARY),
                        );
                        if let Some((current, total)) = self.progress {
                            if total > 0 {
                                ui.add(
                                    egui::ProgressBar::new(current as f32 / total as f32)
                                        .desired_width(120.0)
                                        .text(format!("{}/{}", current, total)),
                                );
                            }
                        }
                    } else {
                        ui.label(RichText::new("Ready").color(COLOR_SUCCESS));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Copy Log").clicked() {
                            let log_text: String = self
                                .log_messages
                                .iter()
                                .map(|(msg, _)| msg.as_str())
                                .collect::<Vec<_>>()
                                .join("\n");
                            ui.ctx().copy_text(log_text);
                        }
                    });
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (msg, level) in &self.log_messages {
                            let color = match level {
                                LogLevel::Info => COLOR_TEXT_SECONDARY,
                                LogLevel::Warning => COLOR_WARNING,
                                LogLevel::Error => COLOR_DANGER,
                            };
                            ui.label(RichText::new(msg).monospace().size(11.0).color(color));
                        }
                    });
            });

        // Main content
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(COLOR_BG_MAIN).inner_margin(16.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.render_paths_section(ui);
                    ui.add_space(12.0);
                    self.render_modlist_section(ui);
                    ui.add_space(12.0);
                    self.render_actions_section(ui);
                    ui.add_space(12.0);
                    self.render_results_section(ui);
                });
            });

        self.render_modals(ctx);
    }
}

impl WabbajackCleanerApp {
    fn section_frame(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::none()
            .fill(COLOR_BG_CARD)
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(title)
                        .size(13.0)
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                add_contents(ui);
            });
    }

    fn render_paths_section(&mut self, ui: &mut egui::Ui) {
        Self::section_frame(ui, "Step 1: Select Folders", |ui| {
            ui.columns(2, |cols| {
                // Wabbajack
                cols[0].label(RichText::new("Wabbajack Installation").color(COLOR_TEXT_PRIMARY));
                cols[0].label(
                    RichText::new("Folder containing Wabbajack.exe")
                        .size(11.0)
                        .color(COLOR_TEXT_MUTED),
                );
                cols[0].add_space(4.0);
                cols[0].horizontal(|ui| {
                    if ui.button("Browse...").clicked() {
                        self.select_wabbajack_dir();
                    }
                    if let Some(p) = &self.wabbajack_dir {
                        ui.label(
                            RichText::new(p.file_name().unwrap_or_default().to_string_lossy())
                                .color(COLOR_SUCCESS),
                        );
                    } else {
                        ui.label(RichText::new("Not selected").color(COLOR_DANGER));
                    }
                });

                // Downloads
                cols[1].label(RichText::new("Downloads Folder").color(COLOR_TEXT_PRIMARY));
                cols[1].label(
                    RichText::new("Wabbajack mod downloads location")
                        .size(11.0)
                        .color(COLOR_TEXT_MUTED),
                );
                cols[1].add_space(4.0);
                cols[1].horizontal(|ui| {
                    if ui.button("Browse...").clicked() {
                        self.select_downloads_dir();
                    }
                    if let Some(p) = &self.downloads_dir {
                        ui.label(
                            RichText::new(p.file_name().unwrap_or_default().to_string_lossy())
                                .color(COLOR_SUCCESS),
                        );
                    } else {
                        ui.label(RichText::new("Not selected").color(COLOR_DANGER));
                    }
                });
            });

            if let Some(stats) = &self.stats {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{} files", stats.total_files))
                            .size(12.0)
                            .color(COLOR_TEXT_SECONDARY),
                    );
                    ui.label(RichText::new(" | ").color(COLOR_TEXT_MUTED));
                    ui.label(
                        RichText::new(format_size(stats.total_size))
                            .size(12.0)
                            .color(COLOR_ACCENT),
                    );
                    ui.label(RichText::new(" | ").color(COLOR_TEXT_MUTED));
                    ui.label(
                        RichText::new(format!("{} game folders", self.game_folders.len()))
                            .size(12.0)
                            .color(COLOR_TEXT_SECONDARY),
                    );
                });
            }
        });
    }

    fn render_modlist_section(&mut self, ui: &mut egui::Ui) {
        Self::section_frame(ui, "Step 2: Select Modlists to Protect", |ui| {
            if self.modlists.is_empty() {
                ui.label(RichText::new("Select Wabbajack folder first.").color(COLOR_TEXT_MUTED));
            } else {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{}/{} selected",
                            self.selected_modlist_count(),
                            self.modlists.len()
                        ))
                        .size(12.0)
                        .color(COLOR_TEXT_SECONDARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("None").clicked() {
                            self.modlist_selected.iter_mut().for_each(|x| *x = false);
                        }
                        if ui.small_button("All").clicked() {
                            self.modlist_selected.iter_mut().for_each(|x| *x = true);
                        }
                    });
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for (i, ml) in self.modlists.iter().enumerate() {
                            let checked = self.modlist_selected.get(i).copied().unwrap_or(false);
                            let mut new_checked = checked;
                            let color = if checked {
                                COLOR_TEXT_PRIMARY
                            } else {
                                COLOR_TEXT_MUTED
                            };
                            if ui
                                .checkbox(
                                    &mut new_checked,
                                    RichText::new(format!("{} ({} mods)", ml.name, ml.mod_count))
                                        .color(color),
                                )
                                .changed()
                            {
                                if let Some(sel) = self.modlist_selected.get_mut(i) {
                                    *sel = new_checked;
                                }
                            }
                        }
                    });
            }
        });
    }

    fn render_actions_section(&mut self, ui: &mut egui::Ui) {
        Self::section_frame(ui, "Step 3: Cleanup Actions", |ui| {
            let ready = self.is_ready() && !self.is_loading;

            ui.columns(2, |cols| {
                // Orphaned Mods
                cols[0].label(
                    RichText::new("Orphaned Mods")
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                cols[0].label(
                    RichText::new("Mods not used by selected modlists")
                        .size(11.0)
                        .color(COLOR_TEXT_MUTED),
                );
                cols[0].add_space(4.0);
                cols[0].horizontal(|ui| {
                    if ui
                        .add_enabled(ready, egui::Button::new("Analyze"))
                        .clicked()
                    {
                        self.run_orphaned_scan(false);
                    }
                    if ui
                        .add_enabled(
                            ready,
                            egui::Button::new(RichText::new("Clean").color(COLOR_TEXT_PRIMARY))
                                .fill(COLOR_DANGER),
                        )
                        .clicked()
                    {
                        self.run_orphaned_scan(true);
                    }
                });

                // Old Versions
                cols[1].label(
                    RichText::new("Old Versions")
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                cols[1].label(
                    RichText::new("Duplicate mods with newer versions")
                        .size(11.0)
                        .color(COLOR_TEXT_MUTED),
                );
                cols[1].add_space(4.0);
                cols[1].horizontal(|ui| {
                    if ui
                        .add_enabled(ready, egui::Button::new("Analyze"))
                        .clicked()
                    {
                        self.run_old_version_scan(false);
                    }
                    if ui
                        .add_enabled(
                            ready,
                            egui::Button::new(RichText::new("Clean").color(COLOR_TEXT_PRIMARY))
                                .fill(COLOR_WARNING),
                        )
                        .clicked()
                    {
                        self.run_old_version_scan(true);
                    }
                });
            });
        });
    }

    fn render_results_section(&mut self, ui: &mut egui::Ui) {
        if self.orphaned_result.is_none() && self.old_version_result.is_none() {
            return;
        }

        Self::section_frame(ui, "Results", |ui| {
            if let Some(res) = &self.orphaned_result {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Orphaned Mods:")
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(format!("{} files", res.orphaned_mods.len()))
                            .color(COLOR_TEXT_SECONDARY),
                    );
                    ui.label(RichText::new(format_size(res.orphaned_size)).color(COLOR_DANGER));
                });
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .id_salt("orphaned")
                    .show(ui, |ui| {
                        for m in &res.orphaned_mods {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&m.file.file_name)
                                        .size(11.0)
                                        .color(COLOR_TEXT_PRIMARY),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format_size(m.file.size))
                                                .size(11.0)
                                                .color(COLOR_TEXT_MUTED),
                                        );
                                    },
                                );
                            });
                        }
                    });
                ui.add_space(8.0);
            }

            if let Some(res) = &self.old_version_result {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Old Versions:")
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(format!("{} files", res.total_files))
                            .color(COLOR_TEXT_SECONDARY),
                    );
                    ui.label(RichText::new(format_size(res.total_space)).color(COLOR_WARNING));
                });
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .id_salt("oldver")
                    .show(ui, |ui| {
                        for group in &res.duplicates {
                            ui.label(
                                RichText::new(&group.mod_key)
                                    .size(11.0)
                                    .strong()
                                    .color(COLOR_ACCENT),
                            );
                            for (i, f) in group.files.iter().enumerate() {
                                let is_keep = i == group.newest_idx;
                                let (status, color) = if is_keep {
                                    ("KEEP", COLOR_SUCCESS)
                                } else {
                                    ("DELETE", COLOR_DANGER)
                                };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("  {} - {}", status, f.file_name))
                                            .size(11.0)
                                            .color(color),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(format_size(f.size))
                                                    .size(11.0)
                                                    .color(COLOR_TEXT_MUTED),
                                            );
                                        },
                                    );
                                });
                            }
                        }
                    });
            }
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context) {
        if self.modal == Modal::About {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .default_width(800.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.columns(2, |cols| {
                        // Left Column: About Info
                        cols[0].vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(
                                RichText::new("Wabbajack Library Cleaner")
                                    .size(24.0)
                                    .strong()
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.label(
                                RichText::new(format!("Version {}", APP_VERSION))
                                    .size(14.0)
                                    .color(COLOR_TEXT_SECONDARY),
                            );
                            ui.add_space(20.0);
                            ui.label(
                                RichText::new("Clean up your Wabbajack downloads folder")
                                    .size(14.0)
                                    .color(COLOR_TEXT_SECONDARY),
                            );
                            ui.label(
                                RichText::new("safely and efficiently.")
                                    .size(14.0)
                                    .color(COLOR_TEXT_SECONDARY),
                            );
                            ui.add_space(30.0);
                            ui.label(
                                RichText::new("Created by Berkay Yetgin").color(COLOR_TEXT_MUTED),
                            );
                            ui.add_space(8.0);
                            ui.hyperlink_to(
                                "GitHub Repository",
                                "https://github.com/Yakrel/wabbajack-library-cleaner",
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("License: GPL-3.0")
                                    .size(11.0)
                                    .color(COLOR_TEXT_MUTED),
                            );
                        });

                        // Right Column: Changelog
                        cols[1].vertical(|ui| {
                            ui.label(
                                RichText::new("Changelog")
                                    .strong()
                                    .size(16.0)
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.add_space(8.0);
                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    let changelog = include_str!("../../CHANGELOG.md");
                                    for line in changelog.lines() {
                                        if let Some(c) = line.strip_prefix("## ") {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new(c).strong().color(COLOR_ACCENT));
                                        } else if let Some(c) = line.strip_prefix("### ") {
                                            ui.label(
                                                RichText::new(c)
                                                    .strong()
                                                    .size(12.0)
                                                    .color(COLOR_TEXT_PRIMARY),
                                            );
                                        } else if let Some(c) = line.strip_prefix("- ") {
                                            ui.label(
                                                RichText::new(format!("• {}", c))
                                                    .size(12.0)
                                                    .color(COLOR_TEXT_SECONDARY),
                                            );
                                        }
                                    }
                                });
                        });
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        if ui.button(RichText::new("Close").size(14.0)).clicked() {
                            self.modal = Modal::None;
                        }
                    });
                });
        }

        if self.modal == Modal::FolderSelect {
            egui::Window::new("Select Game Folder")
                .collapsible(false)
                .resizable(false)
                .default_width(350.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Select a game folder to scan:");
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (i, folder) in self.game_folders.iter().enumerate() {
                                let name = folder.file_name().unwrap_or_default().to_string_lossy();
                                if ui
                                    .selectable_label(self.selected_game_folder == Some(i), &*name)
                                    .clicked()
                                {
                                    self.selected_game_folder = Some(i);
                                }
                            }
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                self.selected_game_folder.is_some(),
                                egui::Button::new("Start Scan").fill(COLOR_ACCENT),
                            )
                            .clicked()
                        {
                            self.start_old_version_scan();
                        }
                        if ui.button("Cancel").clicked() {
                            self.modal = Modal::None;
                        }
                    });
                });
        }
    }
}

// Async helpers
fn scan_wabbajack_dir(path: PathBuf, tx: Sender<AsyncMessage>) {
    tx.send(AsyncMessage::Progress("Scanning...".to_string(), None))
        .ok();
    let entries = match std::fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            tx.send(AsyncMessage::Error(e.to_string())).ok();
            return;
        }
    };

    let mut modlist_map: std::collections::HashMap<String, (PathBuf, String)> =
        std::collections::HashMap::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let version_name = entry.file_name().to_string_lossy().to_string();
        let modlists_path = entry.path().join("downloaded_mod_lists");
        if modlists_path.exists() {
            if let Ok(files) = find_wabbajack_files(&modlists_path) {
                for wbfile in files {
                    let basename = wbfile
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let key = basename.split("@@").next().unwrap_or(&basename).to_string();
                    if modlist_map
                        .get(&key)
                        .map(|(_, v)| &version_name > v)
                        .unwrap_or(true)
                    {
                        modlist_map.insert(key, (wbfile, version_name.clone()));
                    }
                }
            }
        }
    }

    if modlist_map.is_empty() {
        tx.send(AsyncMessage::Error("No modlists found.".to_string()))
            .ok();
        return;
    }

    let total = modlist_map.len();
    let mut modlists = Vec::new();
    for (i, (_, (p, _))) in modlist_map.into_iter().enumerate() {
        tx.send(AsyncMessage::Progress(
            "Parsing modlists...".to_string(),
            Some((i + 1, total)),
        ))
        .ok();
        if let Ok(info) = parse_wabbajack_file(&p) {
            modlists.push(info);
        }
    }
    tx.send(AsyncMessage::ModlistsParsed(modlists)).ok();
}

fn scan_orphaned_mods_async(
    path: PathBuf,
    modlists: Vec<ModlistInfo>,
    delete: bool,
    backup: Option<PathBuf>,
    tx: Sender<AsyncMessage>,
) {
    tx.send(AsyncMessage::Progress(
        "Indexing files...".to_string(),
        None,
    ))
    .ok();
    let folders = match get_game_folders(&path) {
        Ok(f) => f,
        Err(e) => {
            tx.send(AsyncMessage::Error(e.to_string())).ok();
            return;
        }
    };
    let files = match get_all_mod_files(&folders) {
        Ok(f) => f,
        Err(e) => {
            tx.send(AsyncMessage::Error(e.to_string())).ok();
            return;
        }
    };
    tx.send(AsyncMessage::Progress(
        format!("Analyzing {} files...", files.len()),
        None,
    ))
    .ok();
    let result = detect_orphaned_mods(&files, &modlists);
    if delete && !result.orphaned_mods.is_empty() {
        tx.send(AsyncMessage::Progress(
            "Cleaning...".to_string(),
            Some((0, result.orphaned_mods.len())),
        ))
        .ok();
        let del = delete_orphaned_mods(&result.orphaned_mods, backup.as_deref(), None);
        tx.send(AsyncMessage::DeletionComplete(del)).ok();
    } else {
        tx.send(AsyncMessage::OrphanedScanComplete(result)).ok();
    }
}

fn scan_old_versions_async(
    path: PathBuf,
    delete: bool,
    backup: Option<PathBuf>,
    tx: Sender<AsyncMessage>,
) {
    tx.send(AsyncMessage::Progress("Scanning...".to_string(), None))
        .ok();
    let result = match scan_folder_for_duplicates(&path) {
        Ok(r) => r,
        Err(e) => {
            tx.send(AsyncMessage::Error(e.to_string())).ok();
            return;
        }
    };
    if delete && !result.duplicates.is_empty() {
        tx.send(AsyncMessage::Progress(
            "Cleaning...".to_string(),
            Some((0, result.total_files)),
        ))
        .ok();
        let del = delete_old_versions(&result.duplicates, backup.as_deref(), None);
        tx.send(AsyncMessage::DeletionComplete(del)).ok();
    } else {
        tx.send(AsyncMessage::OldVersionScanComplete(result)).ok();
    }
}
