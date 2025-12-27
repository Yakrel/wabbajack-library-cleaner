// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use eframe::egui;
use egui::{Color32, RichText, Stroke, Rounding, Margin};

use crate::core::{
    calculate_library_stats, delete_old_versions, delete_orphaned_mods, detect_orphaned_mods,
    find_wabbajack_files, format_size, get_all_mod_files, get_game_folders, parse_wabbajack_file,
    scan_folder_for_duplicates, DeletionResult, LibraryStats, ModlistInfo, OldVersionScanResult,
    ScanResult,
};

/// Message types for async operations
#[derive(Debug)]
enum AsyncMessage {
    ModlistsParsed(Vec<ModlistInfo>),
    GameFoldersFound(Vec<PathBuf>),
    OrphanedScanComplete(ScanResult),
    OldVersionScanComplete(OldVersionScanResult),
    DeletionComplete(DeletionResult),
    StatsComplete(LibraryStats),
    Progress(String),
    Error(String),
}

/// Main Navigation Pages
#[derive(PartialEq, Clone, Copy)]
enum Page {
    Dashboard,
    Configuration,
    ScanResults,
    About,
}

/// UI State
pub struct WabbajackCleanerApp {
    // Core State
    page: Page,
    wabbajack_dir: Option<PathBuf>,
    downloads_dir: Option<PathBuf>,
    modlists: Vec<ModlistInfo>,
    modlist_selected: Vec<bool>,
    
    // Logic State
    game_folders: Vec<PathBuf>,
    selected_game_folder: Option<usize>,
    pending_delete_mode: bool,
    move_to_backup: bool,
    
    // Async
    tx: Sender<AsyncMessage>,
    rx: Receiver<AsyncMessage>,
    is_scanning: bool,
    current_operation: String, // "Scanning...", "Deleting...", etc.

    // Results Cache
    last_stats: Option<LibraryStats>,
    last_orphaned_result: Option<ScanResult>,
    last_old_version_result: Option<OldVersionScanResult>,
    
    // UI Helpers
    log_messages: Vec<String>,
    show_log: bool,
    show_folder_select: bool,
    notification: Option<(String, f64)>, // Message, Time
}

impl Default for WabbajackCleanerApp {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            page: Page::Configuration, // Start on config to force setup
            wabbajack_dir: None,
            downloads_dir: None,
            modlists: Vec::new(),
            modlist_selected: Vec::new(),
            game_folders: Vec::new(),
            selected_game_folder: None,
            pending_delete_mode: false,
            move_to_backup: true,
            tx,
            rx,
            is_scanning: false,
            current_operation: String::new(),
            last_stats: None,
            last_orphaned_result: None,
            last_old_version_result: None,
            log_messages: Vec::new(),
            show_log: false,
            show_folder_select: false,
            notification: None,
        }
    }
}

impl WabbajackCleanerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Modern Dark Theme Tweaks
        style.visuals.window_rounding = Rounding::same(10.0);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
        style.visuals.widgets.active.rounding = Rounding::same(6.0);
        
        // Colors (Subtle Blue/Grey tint)
        // style.visuals.widgets.active.bg_fill = Color32::from_rgb(60, 100, 180);
        
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = Margin::same(0.0); // We will handle margins in panels

        cc.egui_ctx.set_style(style);
        Self::default()
    }

    // --- Helpers ---

    fn log(&mut self, msg: &str) {
        let time = chrono::Local::now().format("%H:%M:%S");
        self.log_messages.push(format!("[{}] {}", time, msg));
        // Auto-scroll logic handled in UI
    }

    fn notify(&mut self, msg: &str) {
        self.log(msg);
        self.notification = Some((msg.to_string(), 5.0)); // Show for 5 seconds (mock time)
    }
    
    fn is_ready(&self) -> bool {
        self.wabbajack_dir.is_some() && self.downloads_dir.is_some()
    }

    fn get_backup_path(&self) -> Option<PathBuf> {
        if !self.move_to_backup { return None; }
        self.downloads_dir.as_ref().map(|dir| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
            dir.join("WLC_Deleted").join(timestamp.to_string())
        })
    }

    // --- Logic Actions ---

    fn select_wabbajack_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_title("Select Wabbajack Folder").pick_folder() {
            self.wabbajack_dir = Some(path.clone());
            self.notify("Scanning Wabbajack folder...");
            self.is_scanning = true;
            self.current_operation = "Indexing Modlists...".to_string();
            let tx = self.tx.clone();
            thread::spawn(move || scan_wabbajack_dir(path, tx));
        }
    }

    fn select_downloads_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_title("Select Downloads Folder").pick_folder() {
            self.downloads_dir = Some(path.clone());
            self.notify("Indexing downloads folder...");
            let tx = self.tx.clone();
            thread::spawn(move || match get_game_folders(&path) {
                Ok(folders) => { let _ = tx.send(AsyncMessage::GameFoldersFound(folders)); }
                Err(e) => { let _ = tx.send(AsyncMessage::Error(e.to_string())); }
            });
        }
    }
    
    fn run_analysis(&mut self) {
         if !self.is_ready() { return; }
         self.is_scanning = true;
         self.current_operation = "Calculating Library Stats...".to_string();
         let folders = self.game_folders.clone();
         let tx = self.tx.clone();
         thread::spawn(move || {
            let stats = calculate_library_stats(&folders);
            let _ = tx.send(AsyncMessage::StatsComplete(stats));
         });
    }

    fn run_orphaned_scan(&mut self, delete: bool) {
        let selected_modlists: Vec<ModlistInfo> = self.modlists.iter().enumerate()
            .filter(|(i, _)| self.modlist_selected.get(*i).copied().unwrap_or(false))
            .map(|(_, ml)| ml.clone()).collect();
            
        if selected_modlists.is_empty() {
            self.notify("⚠️ No modlists selected! Cannot determine orphaned files.");
            return;
        }
        
        self.is_scanning = true;
        self.current_operation = if delete { "Deleting Orphaned Mods..." } else { "Scanning for Orphaned Mods..." }.to_string();
        
        let path = self.downloads_dir.clone().unwrap();
        let backup = if delete { self.get_backup_path() } else { None };
        let tx = self.tx.clone();
        
        thread::spawn(move || {
            scan_orphaned_mods_async(path, selected_modlists, delete, backup, tx);
        });
    }
    
    fn run_old_version_scan(&mut self, delete: bool) {
        if self.game_folders.is_empty() {
             self.notify("No game folders found.");
             return;
        }
        self.pending_delete_mode = delete;
        self.show_folder_select = true;
    }

    fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMessage::ModlistsParsed(list) => {
                    self.modlists = list;
                    self.modlist_selected = vec![true; self.modlists.len()];
                    self.is_scanning = false;
                    self.notify(&format!("Found {} modlists", self.modlists.len()));
                    // If we have both paths, go to dashboard
                    if self.downloads_dir.is_some() {
                        self.page = Page::Dashboard;
                        self.run_analysis();
                    }
                },
                AsyncMessage::GameFoldersFound(folders) => {
                    self.game_folders = folders;
                    self.notify(&format!("Found {} game folders", self.game_folders.len()));
                    if self.wabbajack_dir.is_some() {
                         self.page = Page::Dashboard;
                         self.run_analysis();
                    }
                },
                AsyncMessage::StatsComplete(stats) => {
                    self.last_stats = Some(stats);
                    self.is_scanning = false;
                },
                AsyncMessage::OrphanedScanComplete(res) => {
                    self.last_orphaned_result = Some(res);
                    self.is_scanning = false;
                    self.page = Page::ScanResults; // Auto switch to results
                },
                AsyncMessage::OldVersionScanComplete(res) => {
                    self.last_old_version_result = Some(res);
                    self.is_scanning = false;
                    self.page = Page::ScanResults;
                },
                AsyncMessage::DeletionComplete(res) => {
                    self.notify(&format!("Cleaned {} files, freed {}", res.deleted_count, format_size(res.space_freed)));
                    self.is_scanning = false;
                    // Refresh stats
                    self.run_analysis();
                },
                AsyncMessage::Progress(s) => self.current_operation = s,
                AsyncMessage::Error(e) => {
                    self.log(&format!("ERROR: {}", e));
                    self.is_scanning = false;
                }
            }
        }
    }
}

impl eframe::App for WabbajackCleanerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();
        if self.is_scanning { ctx.request_repaint(); }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Main Layout: Sidebar (Left) + Content (Right)
            ui.columns(2, |cols| {
                // Adjust column widths: Sidebar is fixed width
                // Hacky way in `columns`: use `StripBuilder` for precise control usually, 
                // but for simple UI, let's just make the sidebar separate visually.
                
                // Actually, `SidePanel` is better for this.
            });
        });
        
        // Re-doing layout with SidePanel
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Wabbajack");
                    ui.label("Library Cleaner");
                });
                ui.add_space(20.0);
                
                let btn = |ui: &mut egui::Ui, label: &str, active: bool| {
                    let text = RichText::new(label).size(16.0).strong();
                    let btn = egui::Button::new(text)
                        .frame(false)
                        .min_size(egui::vec2(180.0, 40.0));
                        
                    if active {
                         ui.add(btn.fill(ui.visuals().widgets.active.bg_fill));
                    } else {
                         if ui.add(btn).clicked() { return true; }
                    }
                    false
                };

                if btn(ui, "📊 Dashboard", self.page == Page::Dashboard) { self.page = Page::Dashboard; }
                if btn(ui, "⚙️ Configuration", self.page == Page::Configuration) { self.page = Page::Configuration; }
                if btn(ui, "📋 Results", self.page == Page::ScanResults) { self.page = Page::ScanResults; }
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                     ui.add_space(20.0);
                     if btn(ui, "ℹ️ About", self.page == Page::About) { self.page = Page::About; }
                });
            });
            
        // Bottom Panel for Status/Log
        if self.show_log {
            egui::TopBottomPanel::bottom("log_panel")
                .resizable(true)
                .min_height(100.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Terminal");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() { self.show_log = false; }
                            if ui.button("Clear").clicked() { self.log_messages.clear(); }
                        });
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 2.0);
                        for msg in &self.log_messages {
                            ui.label(RichText::new(msg).monospace().size(12.0));
                        }
                    });
                });
        } else {
            // Mini status bar
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.is_scanning {
                        ui.spinner();
                        ui.label(&self.current_operation);
                    } else {
                        ui.label("Ready");
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Show Log").clicked() { self.show_log = true; }
                        ui.label("v2.1.0");
                    });
                });
            });
        }

        // Main Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Apply margins
            egui::Frame::none()
                .inner_margin(20.0)
                .show(ui, |ui| {
                    match self.page {
                        Page::Dashboard => self.render_dashboard(ui),
                        Page::Configuration => self.render_config(ui),
                        Page::ScanResults => self.render_results(ui),
                        Page::About => self.render_about(ui),
                    }
                });
        });

        // Modals
        if self.show_folder_select {
            egui::Window::new("Select Game Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Select a folder to scan for duplicates:");
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                         for (i, folder) in self.game_folders.iter().enumerate() {
                            let name = folder.file_name().unwrap_or_default().to_string_lossy();
                            if ui.selectable_label(self.selected_game_folder == Some(i), name).clicked() {
                                self.selected_game_folder = Some(i);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Start Scan").clicked() {
                             if let Some(i) = self.selected_game_folder {
                                 self.show_folder_select = false;
                                 let f = self.game_folders[i].clone();
                                 let del = self.pending_delete_mode;
                                 let tx = self.tx.clone();
                                 let backup = if del { self.get_backup_path() } else { None };
                                 self.is_scanning = true;
                                 self.current_operation = "Scanning for old versions...".to_string();
                                 thread::spawn(move || scan_old_versions_async(f, del, backup, tx));
                             }
                        }
                        if ui.button("Cancel").clicked() { self.show_folder_select = false; }
                    });
                });
        }
    }
}

// UI Render Implementations
impl WabbajackCleanerApp {
    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dashboard");
        ui.add_space(10.0);

        if !self.is_ready() {
            ui.label(RichText::new("⚠️ Setup Incomplete").color(Color32::YELLOW).size(20.0));
            ui.label("Please go to the Configuration tab and select your Wabbajack and Downloads folders.");
            if ui.button("Go to Configuration").clicked() { self.page = Page::Configuration; }
            return;
        }

        // Stats Cards
        if let Some(stats) = &self.last_stats {
            ui.columns(3, |cols| {
                cols[0].group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Total Library Size");
                        ui.label(RichText::new(format_size(stats.total_size)).size(24.0).strong());
                    });
                });
                cols[1].group(|ui| {
                     ui.vertical_centered(|ui| {
                        ui.label("Total Files");
                        ui.label(RichText::new(format!("{}", stats.total_files)).size(24.0).strong());
                    });
                });
                cols[2].group(|ui| {
                     ui.vertical_centered(|ui| {
                        ui.label("Active Modlists");
                        ui.label(RichText::new(format!("{}", self.modlists.len())).size(24.0).strong());
                    });
                });
            });
        } else {
             if ui.button("Load Stats").clicked() { self.run_analysis(); }
        }

        ui.add_space(30.0);
        ui.heading("Actions");
        ui.separator();
        
        // Action Cards
        ui.columns(2, |cols| {
            // Orphaned Mods
            cols[0].group(|ui| {
                ui.set_min_height(150.0);
                ui.heading("🗑️ Orphaned Mods");
                ui.label("Find and remove mods that are not used by any of your currently active modlists.");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.add_enabled(!self.is_scanning, egui::Button::new("Analyze").min_size(egui::vec2(80.0, 30.0))).clicked() {
                        self.run_orphaned_scan(false);
                    }
                    if ui.add_enabled(!self.is_scanning, egui::Button::new(RichText::new("CLEAN").color(Color32::LIGHT_RED)).min_size(egui::vec2(80.0, 30.0))).clicked() {
                        self.run_orphaned_scan(true);
                    }
                });
            });

            // Old Versions
            cols[1].group(|ui| {
                ui.set_min_height(150.0);
                ui.heading("🕰️ Old Versions");
                ui.label("Find duplicate mods where a newer version exists (e.g., Mod-1.0.zip vs Mod-1.1.zip).");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.add_enabled(!self.is_scanning, egui::Button::new("Analyze").min_size(egui::vec2(80.0, 30.0))).clicked() {
                        self.run_old_version_scan(false);
                    }
                    if ui.add_enabled(!self.is_scanning, egui::Button::new(RichText::new("CLEAN").color(Color32::LIGHT_RED)).min_size(egui::vec2(80.0, 30.0))).clicked() {
                        self.run_old_version_scan(true);
                    }
                });
            });
        });
        
        ui.add_space(20.0);
        ui.checkbox(&mut self.move_to_backup, "Safety Mode: Move deleted files to '_backup' folder instead of permanent deletion.");
    }

    fn render_config(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        ui.add_space(10.0);
        
        // Paths Group
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Paths");
            
            ui.label("Wabbajack Installation:");
            ui.horizontal(|ui| {
                if ui.button("📂 Browse...").clicked() { self.select_wabbajack_dir(); }
                if let Some(p) = &self.wabbajack_dir {
                    ui.label(RichText::new(p.to_string_lossy()).monospace().color(Color32::LIGHT_GREEN));
                } else {
                    ui.label(RichText::new("Not selected").italics().color(Color32::LIGHT_RED));
                }
            });
            
            ui.add_space(10.0);
            
            ui.label("Downloads Folder:");
            ui.horizontal(|ui| {
                if ui.button("📂 Browse...").clicked() { self.select_downloads_dir(); }
                if let Some(p) = &self.downloads_dir {
                    ui.label(RichText::new(p.to_string_lossy()).monospace().color(Color32::LIGHT_GREEN));
                } else {
                    ui.label(RichText::new("Not selected").italics().color(Color32::LIGHT_RED));
                }
            });
        });

        ui.add_space(20.0);
        
        // Modlists Group
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Active Modlists");
            ui.label("Select the modlists you want to PROTECT. Files used by these lists will NOT be deleted.");
            ui.add_space(5.0);
            
            if self.modlists.is_empty() {
                ui.label("No modlists found. Please select Wabbajack folder.");
            } else {
                ui.horizontal(|ui| {
                    if ui.button("Select All").clicked() { self.modlist_selected.iter_mut().for_each(|x| *x = true); }
                    if ui.button("Select None").clicked() { self.modlist_selected.iter_mut().for_each(|x| *x = false); }
                });
                
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for (i, ml) in self.modlists.iter().enumerate() {
                        let mut checked = self.modlist_selected[i];
                        if ui.checkbox(&mut checked, format!("{} ({} mods)", ml.name, ml.mod_count)).changed() {
                            self.modlist_selected[i] = checked;
                        }
                    }
                });
            }
        });
    }

    fn render_results(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scan Results");
        ui.separator();

        if self.last_orphaned_result.is_none() && self.last_old_version_result.is_none() {
            ui.label("No scan results yet. Go to Dashboard and run an analysis.");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(res) = &self.last_orphaned_result {
                ui.group(|ui| {
                    ui.heading("Orphaned Mods Results");
                    ui.label(format!("Found {} orphaned files totaling {}", res.orphaned_mods.len(), format_size(res.orphaned_size)));
                    ui.collapsing("View Files", |ui| {
                         for m in &res.orphaned_mods {
                             ui.label(format!("{} ({})", m.file.file_name, format_size(m.file.size)));
                         }
                    });
                });
                ui.add_space(10.0);
            }
            
            if let Some(res) = &self.last_old_version_result {
                ui.group(|ui| {
                    ui.heading("Old Version Results");
                    ui.label(format!("Found {} old versions totaling {}", res.total_files, format_size(res.total_space)));
                    ui.collapsing("View Details", |ui| {
                         for group in &res.duplicates {
                             ui.label(RichText::new(&group.mod_key).strong());
                             for (i, f) in group.files.iter().enumerate() {
                                 let txt = format!(" - {} ({})", f.file_name, format_size(f.size));
                                 if i == group.newest_idx {
                                     ui.label(RichText::new(txt + " [KEEP]").color(Color32::GREEN));
                                 } else {
                                     ui.label(RichText::new(txt + " [DELETE]").color(Color32::RED));
                                 }
                             }
                         }
                    });
                });
            }
        });
    }

    fn render_about(&mut self, ui: &mut egui::Ui) {
        ui.heading("About Wabbajack Library Cleaner");
        ui.add_space(10.0);
        ui.label("This tool helps manage disk space by removing unused mods from your Wabbajack download library.");
        ui.add_space(10.0);
        ui.label("Version: 2.1.0");
        ui.hyperlink("https://github.com/Yakrel/wabbajack-library-cleaner");
        ui.add_space(20.0);
        ui.label("License: GPL-3.0");
    }
}


// --- Async Helpers (Copied from previous implementation) ---
// Note: In a real refactor, these should be in the `core` module or a separate file,
// but to keep this single-file replacement working, I'll include them here.

fn scan_wabbajack_dir(path: PathBuf, tx: Sender<AsyncMessage>) {
    tx.send(AsyncMessage::Progress("Scanning version folders...".to_string())).ok();
    // (Logic identical to previous, abbreviated for brevity)
    let entries = match std::fs::read_dir(&path) { Ok(e) => e, Err(e) => { tx.send(AsyncMessage::Error(e.to_string())).ok(); return; } };
    let mut modlist_map: std::collections::HashMap<String, (PathBuf, String)> = std::collections::HashMap::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
        let version_name = entry.file_name().to_string_lossy().to_string();
        let modlists_path = entry.path().join("downloaded_mod_lists");
        if modlists_path.exists() {
            if let Ok(wabbajack_files) = find_wabbajack_files(&modlists_path) {
                for wbfile in wabbajack_files {
                    let basename = wbfile.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    let modlist_key = basename.split("@@").next().unwrap_or(&basename).to_string();
                    let should_update = modlist_map.get(&modlist_key).map(|(_, v)| &version_name > v).unwrap_or(true);
                    if should_update { modlist_map.insert(modlist_key, (wbfile, version_name.clone())); }
                }
            }
        }
    }
    if modlist_map.is_empty() { tx.send(AsyncMessage::Error("No modlists found.".to_string())).ok(); return; }
    let mut modlists = Vec::new();
    for (_, (path, _)) in modlist_map {
        if let Ok(info) = parse_wabbajack_file(&path) { modlists.push(info); }
    }
    tx.send(AsyncMessage::ModlistsParsed(modlists)).ok();
}

fn scan_orphaned_mods_async(path: PathBuf, ml: Vec<ModlistInfo>, del: bool, bak: Option<PathBuf>, tx: Sender<AsyncMessage>) {
    tx.send(AsyncMessage::Progress("Indexing files...".to_string())).ok();
    let folders = match get_game_folders(&path) { Ok(f) => f, Err(e) => { tx.send(AsyncMessage::Error(e.to_string())).ok(); return; } };
    let files = match get_all_mod_files(&folders) { Ok(f) => f, Err(e) => { tx.send(AsyncMessage::Error(e.to_string())).ok(); return; } };
    tx.send(AsyncMessage::Progress("Analyzing...".to_string())).ok();
    let res = detect_orphaned_mods(&files, &ml);
    if del && !res.orphaned_mods.is_empty() {
        tx.send(AsyncMessage::Progress("Cleaning...".to_string())).ok();
        let del_res = delete_orphaned_mods(&res.orphaned_mods, bak.as_deref(), None);
        tx.send(AsyncMessage::DeletionComplete(del_res)).ok();
    } else {
        tx.send(AsyncMessage::OrphanedScanComplete(res)).ok();
    }
}

fn scan_old_versions_async(path: PathBuf, del: bool, bak: Option<PathBuf>, tx: Sender<AsyncMessage>) {
    tx.send(AsyncMessage::Progress("Scanning duplicates...".to_string())).ok();
    let res = match scan_folder_for_duplicates(&path) { Ok(r) => r, Err(e) => { tx.send(AsyncMessage::Error(e.to_string())).ok(); return; } };
    if del && !res.duplicates.is_empty() {
        tx.send(AsyncMessage::Progress("Cleaning duplicates...".to_string())).ok();
        let del_res = delete_old_versions(&res.duplicates, bak.as_deref(), None);
        tx.send(AsyncMessage::DeletionComplete(del_res)).ok();
    } else {
        tx.send(AsyncMessage::OldVersionScanComplete(res)).ok();
    }
}