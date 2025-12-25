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

use crate::core::{
    calculate_library_stats, delete_old_versions, delete_orphaned_mods,
    detect_orphaned_mods, find_wabbajack_files, format_size, get_all_mod_files,
    get_game_folders, parse_wabbajack_file, scan_folder_for_duplicates,
    DeletionResult, LibraryStats, ModlistInfo, OldVersionScanResult, ScanResult,
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

/// Application state
pub struct WabbajackCleanerApp {
    // Directory paths
    wabbajack_dir: Option<PathBuf>,
    downloads_dir: Option<PathBuf>,
    
    // Modlist management
    modlists: Vec<ModlistInfo>,
    modlist_selected: Vec<bool>,
    
    // Scan results
    last_orphaned_scan: Option<ScanResult>,
    last_old_version_scan: Option<OldVersionScanResult>,
    last_stats: Option<LibraryStats>,
    
    // Game folders for old version scan
    game_folders: Vec<PathBuf>,
    selected_game_folder: Option<usize>,
    pending_delete_mode: bool, // Track delete mode for folder selection dialog
    
    // Options
    move_to_backup: bool,
    
    // UI state
    output_log: String,
    status: String,
    is_scanning: bool,
    show_about: bool,
    show_folder_select: bool,
    
    // Async communication
    tx: Sender<AsyncMessage>,
    rx: Receiver<AsyncMessage>,
}

impl Default for WabbajackCleanerApp {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            wabbajack_dir: None,
            downloads_dir: None,
            modlists: Vec::new(),
            modlist_selected: Vec::new(),
            last_orphaned_scan: None,
            last_old_version_scan: None,
            last_stats: None,
            game_folders: Vec::new(),
            selected_game_folder: None,
            pending_delete_mode: false,
            move_to_backup: true,
            output_log: String::new(),
            status: "Ready".to_string(),
            is_scanning: false,
            show_about: false,
            show_folder_select: false,
            tx,
            rx,
        }
    }
}

impl WabbajackCleanerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    
    fn append_output(&mut self, text: &str) {
        if !self.output_log.is_empty() {
            self.output_log.push('\n');
        }
        self.output_log.push_str(text);
    }
    
    fn get_selected_modlists(&self) -> Vec<ModlistInfo> {
        self.modlists
            .iter()
            .enumerate()
            .filter(|(i, _)| self.modlist_selected.get(*i).copied().unwrap_or(false))
            .map(|(_, ml)| ml.clone())
            .collect()
    }
    
    fn get_backup_path(&self) -> Option<PathBuf> {
        if !self.move_to_backup {
            return None;
        }
        
        self.downloads_dir.as_ref().map(|dir| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
            dir.join("WLC_Deleted").join(timestamp.to_string())
        })
    }
    
    fn select_wabbajack_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Wabbajack Folder (where Wabbajack.exe is)")
            .pick_folder()
        {
            self.wabbajack_dir = Some(path.clone());
            self.append_output("\n=== Scanning Wabbajack Installation ===");
            self.append_output(&format!("Selected Wabbajack folder: {:?}", path));
            self.status = "Scanning version folders...".to_string();
            
            // Scan for modlists in background
            let tx = self.tx.clone();
            thread::spawn(move || {
                scan_wabbajack_dir(path, tx);
            });
            
            self.is_scanning = true;
        }
    }
    
    fn select_downloads_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Downloads Folder")
            .pick_folder()
        {
            self.downloads_dir = Some(path.clone());
            self.append_output(&format!("Selected downloads directory: {:?}", path));
            
            // Get game folders for old version scan
            let tx = self.tx.clone();
            thread::spawn(move || {
                match get_game_folders(&path) {
                    Ok(folders) => {
                        let _ = tx.send(AsyncMessage::GameFoldersFound(folders));
                    }
                    Err(e) => {
                        let _ = tx.send(AsyncMessage::Error(format!("Failed to get game folders: {}", e)));
                    }
                }
            });
        }
    }
    
    fn scan_orphaned_mods(&mut self, delete_mode: bool) {
        if self.wabbajack_dir.is_none() {
            self.append_output("❌ Please select the Wabbajack Folder first (Step 1)");
            return;
        }
        
        if self.downloads_dir.is_none() {
            self.append_output("❌ Please select the Downloads Folder first (Step 2)");
            return;
        }
        
        let selected = self.get_selected_modlists();
        if selected.is_empty() {
            self.append_output("❌ No modlists selected. Please check at least one modlist.");
            return;
        }
        
        self.append_output("\n=== Scanning for Orphaned Mods ===");
        self.append_output(&format!("Using {} selected modlist(s):", selected.len()));
        for ml in &selected {
            self.append_output(&format!("  ✓ {}", ml.name));
        }
        
        self.status = "Scanning for orphaned mods...".to_string();
        self.is_scanning = true;
        
        let downloads_dir = self.downloads_dir.clone().unwrap();
        let backup_path = if delete_mode { self.get_backup_path() } else { None };
        let tx = self.tx.clone();
        
        thread::spawn(move || {
            scan_orphaned_mods_async(downloads_dir, selected, delete_mode, backup_path, tx);
        });
    }
    
    fn scan_old_versions(&mut self, delete_mode: bool) {
        if self.downloads_dir.is_none() {
            self.append_output("❌ Please select the Downloads Folder first");
            return;
        }
        
        if self.game_folders.is_empty() {
            self.append_output("❌ No game folders found");
            return;
        }
        
        // Store delete_mode for when folder is selected
        self.pending_delete_mode = delete_mode;
        // Show folder selection dialog
        self.show_folder_select = true;
    }
    
    fn perform_old_version_scan(&mut self, folder_idx: usize, delete_mode: bool) {
        if folder_idx >= self.game_folders.len() {
            return;
        }
        
        let folder = self.game_folders[folder_idx].clone();
        self.append_output(&format!("\nScanning: {:?}", folder.file_name().unwrap_or_default()));
        self.status = "Scanning for old versions...".to_string();
        self.is_scanning = true;
        
        let backup_path = if delete_mode { self.get_backup_path() } else { None };
        let tx = self.tx.clone();
        
        thread::spawn(move || {
            scan_old_versions_async(folder, delete_mode, backup_path, tx);
        });
    }
    
    fn view_stats(&mut self) {
        if self.downloads_dir.is_none() {
            self.append_output("❌ Please select the Downloads Folder first");
            return;
        }
        
        self.status = "Calculating statistics...".to_string();
        self.is_scanning = true;
        
        let game_folders = self.game_folders.clone();
        let tx = self.tx.clone();
        
        thread::spawn(move || {
            let stats = calculate_library_stats(&game_folders);
            let _ = tx.send(AsyncMessage::StatsComplete(stats));
        });
    }
    
    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMessage::ModlistsParsed(modlists) => {
                    self.append_output(&format!("\n📊 Total unique modlists found: {}", modlists.len()));
                    for ml in &modlists {
                        self.append_output(&format!("  ✓ {} ({} mods)", ml.name, ml.mod_count));
                    }
                    self.modlist_selected = vec![true; modlists.len()];
                    self.modlists = modlists;
                    self.status = format!("Found {} modlists", self.modlists.len());
                    self.is_scanning = false;
                }
                AsyncMessage::GameFoldersFound(folders) => {
                    self.append_output(&format!("Found {} game folder(s)", folders.len()));
                    for f in &folders {
                        self.append_output(&format!("  - {:?}", f.file_name().unwrap_or_default()));
                    }
                    self.game_folders = folders;
                }
                AsyncMessage::OrphanedScanComplete(result) => {
                    self.append_output("\n=== RESULTS ===");
                    self.append_output(&format!("✓ USED MODS: {} files ({})", 
                        result.used_mods.len(), format_size(result.used_size)));
                    self.append_output(&format!("✗ ORPHANED MODS: {} files ({})",
                        result.orphaned_mods.len(), format_size(result.orphaned_size)));
                    
                    if !result.orphaned_mods.is_empty() {
                        self.append_output("\nExamples:");
                        for (i, om) in result.orphaned_mods.iter().take(10).enumerate() {
                            self.append_output(&format!("  • {} ({})", 
                                om.file.file_name, format_size(om.file.size)));
                            if i >= 9 && result.orphaned_mods.len() > 10 {
                                self.append_output(&format!("  ... and {} more", result.orphaned_mods.len() - 10));
                                break;
                            }
                        }
                    }
                    
                    self.last_orphaned_scan = Some(result);
                    self.status = "Scan complete".to_string();
                    self.is_scanning = false;
                }
                AsyncMessage::OldVersionScanComplete(result) => {
                    self.append_output(&format!("\nFound {} groups with old versions", result.duplicates.len()));
                    self.append_output(&format!("Total: {} old files, {} to free", 
                        result.total_files, format_size(result.total_space)));
                    
                    // Show some examples
                    for (i, group) in result.duplicates.iter().take(5).enumerate() {
                        let newest = &group.files[group.newest_idx];
                        self.append_output(&format!("\n{} - {}", group.mod_key, newest.file_name));
                        self.append_output(&format!("  Will delete {} old version(s), saving {}",
                            group.files.len() - 1, format_size(group.space_to_free)));
                        
                        if i >= 4 && result.duplicates.len() > 5 {
                            self.append_output(&format!("... and {} more groups", result.duplicates.len() - 5));
                            break;
                        }
                    }
                    
                    self.last_old_version_scan = Some(result);
                    self.status = "Scan complete".to_string();
                    self.is_scanning = false;
                }
                AsyncMessage::DeletionComplete(result) => {
                    self.append_output(&format!("\nDeleted: {} files", result.deleted_count));
                    self.append_output(&format!("Space freed: {}", format_size(result.space_freed)));
                    
                    if !result.skipped.is_empty() {
                        self.append_output(&format!("Skipped: {} files", result.skipped.len()));
                    }
                    
                    self.status = format!("Completed: {} files deleted", result.deleted_count);
                    self.is_scanning = false;
                }
                AsyncMessage::StatsComplete(stats) => {
                    self.append_output("\n=== Library Statistics ===");
                    self.append_output(&format!("\nTotal Files: {}", stats.total_files));
                    self.append_output(&format!("Total Size: {}", format_size(stats.total_size)));
                    self.append_output("\nBy Game:");
                    
                    for (game, files, size) in &stats.by_game {
                        self.append_output(&format!("  {}: {} files ({})", game, files, format_size(*size)));
                    }
                    
                    self.last_stats = Some(stats);
                    self.status = "Statistics calculated".to_string();
                    self.is_scanning = false;
                }
                AsyncMessage::Progress(msg) => {
                    self.status = msg;
                }
                AsyncMessage::Error(msg) => {
                    self.append_output(&format!("❌ Error: {}", msg));
                    self.status = "Error".to_string();
                    self.is_scanning = false;
                }
            }
        }
    }
}

impl eframe::App for WabbajackCleanerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process async messages
        self.process_messages();
        
        // Request repaint if scanning
        if self.is_scanning {
            ctx.request_repaint();
        }
        
        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Title
                ui.vertical_centered(|ui| {
                    ui.heading("Wabbajack Library Cleaner v2.0");
                    ui.label("Clean orphaned mods and old versions from your Wabbajack downloads");
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                // Step 1: Wabbajack folder selection
                ui.add_space(10.0);
                ui.heading("Step 1: Select Wabbajack Folder");
                ui.label("Select your Wabbajack installation folder (where Wabbajack.exe is located)");
                ui.label("Example: D:\\Wabbajack or D:\\Games\\Wabbajack");
                ui.label("💡 The tool will automatically scan all version folders for modlists");
                
                ui.horizontal(|ui| {
                    if ui.button("📁 Select Wabbajack Folder").clicked() {
                        self.select_wabbajack_dir();
                    }
                    if let Some(ref path) = self.wabbajack_dir {
                        ui.label(format!("Selected: {:?}", path));
                    } else {
                        ui.label("(Not selected)");
                    }
                });
                
                // Modlist checkboxes
                if !self.modlists.is_empty() {
                    ui.add_space(5.0);
                    ui.separator();
                    ui.label(egui::RichText::new("Select Active Modlists:").strong());
                    ui.label("Check the modlists you are currently using:");
                    
                    for (i, ml) in self.modlists.iter().enumerate() {
                        let mut checked = self.modlist_selected.get(i).copied().unwrap_or(false);
                        if ui.checkbox(&mut checked, format!("{} ({} mods)", ml.name, ml.mod_count)).changed() {
                            if i < self.modlist_selected.len() {
                                self.modlist_selected[i] = checked;
                            }
                        }
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("Select All").clicked() {
                            self.modlist_selected.iter_mut().for_each(|c| *c = true);
                        }
                        if ui.button("Deselect All").clicked() {
                            self.modlist_selected.iter_mut().for_each(|c| *c = false);
                        }
                    });
                }
                
                ui.add_space(10.0);
                ui.separator();
                
                // Step 2: Downloads folder selection
                ui.add_space(10.0);
                ui.heading("Step 2: Select Downloads Folder");
                ui.label("Select your downloads folder (e.g., F:\\Wabbajack or F:\\Wabbajack\\Fallout 4)");
                ui.label("💡 You can select either the parent folder or a specific game folder");
                
                ui.horizontal(|ui| {
                    if ui.button("📁 Select Downloads Folder").clicked() {
                        self.select_downloads_dir();
                    }
                    if let Some(ref path) = self.downloads_dir {
                        ui.label(format!("Selected: {:?}", path));
                    } else {
                        ui.label("(Not selected)");
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                // Step 3: Options
                ui.add_space(10.0);
                ui.heading("Step 3: Deletion Options");
                
                if let Some(ref downloads) = self.downloads_dir {
                    let backup_path = downloads.join("WLC_Deleted").join("<timestamp>");
                    ui.label(format!("Deleted files will be moved to: {:?}", backup_path));
                } else {
                    ui.label("Deleted files will be moved to: (Select downloads folder first)");
                }
                
                ui.checkbox(&mut self.move_to_backup, "💾 Move to deletion folder (can be restored later)");
                
                ui.add_space(10.0);
                ui.separator();
                
                // Step 4: Actions
                ui.add_space(10.0);
                ui.heading("Step 4: Cleanup Actions");
                
                ui.label(egui::RichText::new("PRIMARY: Orphaned Mods Cleanup").strong());
                ui.label("Remove mods not used by selected modlists (major space savings)");
                
                ui.add_enabled_ui(!self.is_scanning, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("🔍 Scan for Orphaned Mods").clicked() {
                            self.scan_orphaned_mods(false);
                        }
                        if ui.button("🧹 Clean Orphaned Mods").clicked() {
                            self.scan_orphaned_mods(true);
                        }
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                ui.label(egui::RichText::new("SECONDARY: Old Versions Cleanup").strong());
                ui.label("⚠️ Warning: Some modlists may require old versions! Check carefully before cleaning.");
                
                ui.add_enabled_ui(!self.is_scanning, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("🔍 Scan for Old Versions").clicked() {
                            self.scan_old_versions(false);
                        }
                        if ui.button("🧹 Clean Old Versions").clicked() {
                            self.scan_old_versions(true);
                        }
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                ui.add_enabled_ui(!self.is_scanning, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📊 View Statistics").clicked() {
                            self.view_stats();
                        }
                        if ui.button("📖 About").clicked() {
                            self.show_about = true;
                        }
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                // Status and progress
                ui.horizontal(|ui| {
                    ui.label(&self.status);
                    if self.is_scanning {
                        ui.spinner();
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                // Output
                ui.heading("Output");
                
                egui::ScrollArea::vertical()
                    .id_salt("output_scroll")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.output_log.as_str())
                                .desired_width(f32::INFINITY)
                                .desired_rows(15)
                                .font(egui::TextStyle::Monospace)
                        );
                    });
                
                if ui.button("Clear Output").clicked() {
                    self.output_log.clear();
                }
                
                ui.add_space(10.0);
                ui.separator();
                
                // Footer
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Made by Berkay Yetgin | github.com/Yakrel/wabbajack-library-cleaner").italics());
                });
            });
        });
        
        // Folder selection dialog
        if self.show_folder_select {
            egui::Window::new("Select Folder to Scan")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Select which mod folder to scan for old versions:");
                    
                    for (i, folder) in self.game_folders.iter().enumerate() {
                        let name = folder.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        let is_selected = self.selected_game_folder == Some(i);
                        if ui.selectable_label(is_selected, &name).clicked() {
                            self.selected_game_folder = Some(i);
                        }
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("Scan").clicked() {
                            if let Some(idx) = self.selected_game_folder {
                                self.show_folder_select = false;
                                self.perform_old_version_scan(idx, false);
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_folder_select = false;
                        }
                    });
                });
        }
        
        // About dialog
        if self.show_about {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Wabbajack Library Cleaner v2.0");
                    ui.add_space(10.0);
                    ui.label("© 2025 Berkay Yetgin");
                    ui.add_space(10.0);
                    ui.hyperlink_to("GitHub", "https://github.com/Yakrel/wabbajack-library-cleaner");
                    ui.add_space(10.0);
                    ui.label("Licensed under GNU General Public License v3.0");
                    ui.label("This is free and open source software.");
                    ui.add_space(10.0);
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }
    }
}

// Async helper functions
fn scan_wabbajack_dir(path: PathBuf, tx: Sender<AsyncMessage>) {
    let _ = tx.send(AsyncMessage::Progress("Scanning version folders...".to_string()));
    
    // Find all version folders
    let entries = match std::fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            let _ = tx.send(AsyncMessage::Error(format!("Failed to read directory: {}", e)));
            return;
        }
    };
    
    let mut modlist_map: std::collections::HashMap<String, (PathBuf, String)> = std::collections::HashMap::new();
    
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        
        let version_name = entry.file_name().to_string_lossy().to_string();
        let modlists_path = entry.path().join("downloaded_mod_lists");
        
        if modlists_path.exists() {
            if let Ok(wabbajack_files) = find_wabbajack_files(&modlists_path) {
                for wbfile in wabbajack_files {
                    let basename = wbfile.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    
                    // Extract modlist key (before @@)
                    let modlist_key = basename.split("@@").next().unwrap_or(&basename).to_string();
                    
                    // Keep the latest version (alphabetically last = newest)
                    let should_update = modlist_map.get(&modlist_key)
                        .map(|(_, v)| &version_name > v)
                        .unwrap_or(true);
                    
                    if should_update {
                        modlist_map.insert(modlist_key, (wbfile, version_name.clone()));
                    }
                }
            }
        }
    }
    
    if modlist_map.is_empty() {
        let _ = tx.send(AsyncMessage::Error(
            "No version folders with downloaded_mod_lists found. Make sure you selected the Wabbajack root folder.".to_string()
        ));
        return;
    }
    
    // Parse the modlists
    let mut modlists = Vec::new();
    for (_, (path, _)) in modlist_map {
        match parse_wabbajack_file(&path) {
            Ok(info) => modlists.push(info),
            Err(e) => {
                log::warn!("Failed to parse {:?}: {}", path, e);
            }
        }
    }
    
    if modlists.is_empty() {
        let _ = tx.send(AsyncMessage::Error("Failed to parse any modlist files".to_string()));
        return;
    }
    
    let _ = tx.send(AsyncMessage::ModlistsParsed(modlists));
}

fn scan_orphaned_mods_async(
    downloads_dir: PathBuf,
    selected_modlists: Vec<ModlistInfo>,
    delete_mode: bool,
    backup_path: Option<PathBuf>,
    tx: Sender<AsyncMessage>,
) {
    let _ = tx.send(AsyncMessage::Progress("Getting game folders...".to_string()));
    
    let game_folders = match get_game_folders(&downloads_dir) {
        Ok(f) => f,
        Err(e) => {
            let _ = tx.send(AsyncMessage::Error(format!("Failed to get game folders: {}", e)));
            return;
        }
    };
    
    let _ = tx.send(AsyncMessage::Progress("Collecting mod files...".to_string()));
    
    let all_mod_files = match get_all_mod_files(&game_folders) {
        Ok(f) => f,
        Err(e) => {
            let _ = tx.send(AsyncMessage::Error(format!("Failed to collect mod files: {}", e)));
            return;
        }
    };
    
    let _ = tx.send(AsyncMessage::Progress("Analyzing mod usage...".to_string()));
    
    let result = detect_orphaned_mods(&all_mod_files, &selected_modlists);
    
    if delete_mode && !result.orphaned_mods.is_empty() {
        let _ = tx.send(AsyncMessage::Progress("Deleting orphaned mods...".to_string()));
        
        let deletion_result = delete_orphaned_mods(
            &result.orphaned_mods,
            backup_path.as_deref(),
            None,
        );
        
        let _ = tx.send(AsyncMessage::DeletionComplete(deletion_result));
    } else {
        let _ = tx.send(AsyncMessage::OrphanedScanComplete(result));
    }
}

fn scan_old_versions_async(
    folder: PathBuf,
    delete_mode: bool,
    backup_path: Option<PathBuf>,
    tx: Sender<AsyncMessage>,
) {
    let _ = tx.send(AsyncMessage::Progress("Scanning for duplicates...".to_string()));
    
    let result = match scan_folder_for_duplicates(&folder) {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(AsyncMessage::Error(format!("Scan failed: {}", e)));
            return;
        }
    };
    
    if delete_mode && !result.duplicates.is_empty() {
        let _ = tx.send(AsyncMessage::Progress("Deleting old versions...".to_string()));
        
        let deletion_result = delete_old_versions(
            &result.duplicates,
            backup_path.as_deref(),
            None,
        );
        
        let _ = tx.send(AsyncMessage::DeletionComplete(deletion_result));
    } else {
        let _ = tx.send(AsyncMessage::OldVersionScanComplete(result));
    }
}
