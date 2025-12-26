// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;

use crate::core::parser::{
    extract_part_indicator, is_full_or_main_file, is_wabbajack_file, normalize_mod_name,
    parse_mod_filename,
};
use crate::core::types::{
    LibraryStats, ModFile, ModGroup, ModlistInfo, OldVersionScanResult, OrphanedMod, ScanResult,
};

/// Get game folders from a base directory
pub fn get_game_folders(base_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut folders = Vec::new();

    let entries = fs::read_dir(base_dir)
        .with_context(|| format!("Failed to read directory: {:?}", base_dir))?;

    // Check if this directory itself contains mod files
    let mut has_mod_files = false;
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() && is_wabbajack_file(&entry.file_name().to_string_lossy()) {
            has_mod_files = true;
            break;
        }
    }

    // If the selected directory contains mod files, include it
    if has_mod_files {
        log::info!(
            "Selected directory contains mod files, including it: {:?}",
            base_dir
        );
        folders.push(base_dir.to_path_buf());
    }

    // Also scan for subdirectories (game folders)
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if entry.file_type()?.is_dir() && !name_str.starts_with('.') && !name_str.starts_with("__")
        {
            folders.push(entry.path());
        }
    }

    folders.sort();
    Ok(folders)
}

/// Find all .wabbajack files in a directory
pub fn find_wabbajack_files(base_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut wabbajack_files = Vec::new();

    let entries = fs::read_dir(base_dir)
        .with_context(|| format!("Failed to read directory: {:?}", base_dir))?;

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }

        let name = entry.file_name();
        if name
            .to_string_lossy()
            .to_lowercase()
            .ends_with(".wabbajack")
        {
            wabbajack_files.push(entry.path());
        }
    }

    Ok(wabbajack_files)
}

/// Collect all mod files from game folders
pub fn get_all_mod_files(game_folders: &[std::path::PathBuf]) -> Result<Vec<ModFile>> {
    // Process game folders in parallel
    let all_files: Vec<ModFile> = game_folders
        .par_iter()
        .flat_map(|folder| {
            let entries = match fs::read_dir(folder) {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("Failed to read folder {:?}: {}", folder, e);
                    return Vec::new();
                }
            };

            // Collect valid entries first to avoid holding I/O locks
            let valid_entries: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| !e.file_type().map(|t| t.is_dir()).unwrap_or(true))
                .collect();

            // Process entries in parallel within each folder
            valid_entries
                .par_iter()
                .filter_map(|entry| {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    if !is_wabbajack_file(&filename) {
                        return None;
                    }

                    if let Some(mut mod_file) = parse_mod_filename(&filename) {
                        let full_path = entry.path();
                        if let Ok(metadata) = fs::metadata(&full_path) {
                            mod_file.full_path = full_path;
                            mod_file.size = metadata.len();
                            return Some(mod_file);
                        }
                    }
                    None
                })
                .collect::<Vec<ModFile>>()
        })
        .collect();

    Ok(all_files)
}

/// Detect orphaned mods by comparing mod files with active modlists
pub fn detect_orphaned_mods(mod_files: &[ModFile], active_modlists: &[ModlistInfo]) -> ScanResult {
    // Build combined sets of all used ModIDs and ModID+FileID pairs
    let mut used_mod_ids = std::collections::HashSet::new();
    let mut used_mod_file_ids = std::collections::HashSet::new();

    for modlist in active_modlists {
        for mod_key in &modlist.used_mod_keys {
            used_mod_ids.insert(mod_key.clone());
        }
        for mod_file_key in &modlist.used_mod_file_ids {
            used_mod_file_ids.insert(mod_file_key.clone());
        }
    }

    log::info!(
        "Total unique ModIDs in active modlists: {}",
        used_mod_ids.len()
    );
    log::info!(
        "Total unique ModID+FileID pairs: {}",
        used_mod_file_ids.len()
    );

    // Use thread-safe sets for parallel lookup
    // Since we only read, standard HashSet is fine if wrapped or cloned,
    // but here we just pass references to the closure.

    let (used_mods, orphaned_mods): (Vec<ModFile>, Vec<OrphanedMod>) =
        mod_files.par_iter().partition_map(|mod_file| {
            let mut is_used = false;

            // First, try precise matching with ModID+FileID if available
            if let Some(ref file_id) = mod_file.file_id {
                let mod_file_key = format!("{}-{}", mod_file.mod_id, file_id);
                if used_mod_file_ids.contains(&mod_file_key) {
                    is_used = true;
                }
            }

            // Fall back to ModID-only matching
            if !is_used && used_mod_ids.contains(&mod_file.mod_id) {
                is_used = true;
            }

            if is_used {
                rayon::iter::Either::Left(mod_file.clone())
            } else {
                rayon::iter::Either::Right(OrphanedMod {
                    file: mod_file.clone(),
                })
            }
        });

    let used_size: u64 = used_mods.par_iter().map(|m| m.size).sum();
    let orphaned_size: u64 = orphaned_mods.par_iter().map(|m| m.file.size).sum();

    log::info!(
        "Classification complete: {} used, {} orphaned",
        used_mods.len(),
        orphaned_mods.len()
    );

    ScanResult {
        used_mods,
        orphaned_mods,
        used_size,
        orphaned_size,
    }
}

/// Check if files have conflicting descriptors (different content variants)
fn has_conflicting_descriptors(filename1: &str, filename2: &str) -> bool {
    let lower1 = filename1.to_lowercase();
    let lower2 = filename2.to_lowercase();

    let all_descriptors = [
        // Texture quality
        " 1k",
        " 2k",
        " 4k",
        " 8k",
        "-1k",
        "-2k",
        "-4k",
        "-8k",
        // Body types
        "cbbe",
        "uunp",
        "bhunp",
        "vanilla body",
        "bodyslide",
        // Mod components
        " armor",
        " weapon",
        " clothes",
        " clothing",
        " hair",
        " gloves",
        " boots",
        " helmet",
        " meshes",
        " textures",
        "-armor",
        "-weapon",
        "-clothes",
        "-hair",
        "-gloves",
        // File types
        " esp ",
        " esm ",
        " esl ",
        "esp-fe",
        "esp only",
        "esm only",
        "loose files",
        " bsa",
        // Compatibility
        " compat",
        "compatibility",
        " aslal",
        "no worldspace",
        "worldspace edit",
        " performance",
        // Edition types
        " lite",
        " light",
        " full",
        " extended",
        " complete",
        " basic",
        " standard",
        " deluxe",
        // Clean variants
        " clean",
        " dirty",
        " gross",
        // Optional content
        " optional",
        " addon",
        " add-on",
        " expansion",
    ];

    let descriptors1: Vec<_> = all_descriptors
        .iter()
        .filter(|d| lower1.contains(*d))
        .collect();
    let descriptors2: Vec<_> = all_descriptors
        .iter()
        .filter(|d| lower2.contains(*d))
        .collect();

    // If one file has descriptors but the other doesn't
    if (descriptors1.is_empty() != descriptors2.is_empty())
        && (!descriptors1.is_empty() || !descriptors2.is_empty())
    {
        return true;
    }

    // If both have descriptors but they don't share any
    if !descriptors1.is_empty() && !descriptors2.is_empty() {
        let has_common = descriptors1.iter().any(|d1| descriptors2.contains(d1));
        if !has_common {
            return true;
        }
    }

    false
}

/// Check if a mod group has suspicious version patterns
fn has_suspicious_version_pattern(group: &ModGroup) -> bool {
    if group.files.len() < 2 {
        return false;
    }

    for i in 0..group.files.len() - 1 {
        for j in i + 1..group.files.len() {
            let file1 = &group.files[i];
            let file2 = &group.files[j];

            // If versions are identical
            if file1.version == file2.version {
                // Check size ratio (>10x difference)
                let size_ratio = file1.size as f64 / file2.size as f64;
                if !(0.1..=10.0).contains(&size_ratio) {
                    log::warn!(
                        "Group {}: Same version '{}' but size diff >10x",
                        group.mod_key,
                        file1.version
                    );
                    return true;
                }

                // Check timestamp difference (< 1 hour apart)
                if let (Ok(ts1), Ok(ts2)) = (
                    file1.timestamp.parse::<i64>(),
                    file2.timestamp.parse::<i64>(),
                ) {
                    let time_diff = (ts2 - ts1).abs();
                    if time_diff < 3600 {
                        log::warn!(
                            "Group {}: Same version '{}' uploaded within 1 hour",
                            group.mod_key,
                            file1.version
                        );
                        return true;
                    }
                }
            }

            // Check for conflicting descriptors
            if has_conflicting_descriptors(&file1.file_name, &file2.file_name) {
                log::warn!(
                    "Group {}: Files have conflicting descriptors",
                    group.mod_key
                );
                return true;
            }
        }
    }

    false
}

/// Scan folder for old versions (duplicates)
pub fn scan_folder_for_duplicates(folder_path: &Path) -> Result<OldVersionScanResult> {
    log::info!("Scanning folder: {:?}", folder_path);

    let mut mod_groups: HashMap<String, ModGroup> = HashMap::new();
    let mut skipped = 0;

    let entries = fs::read_dir(folder_path)
        .with_context(|| format!("Failed to read directory: {:?}", folder_path))?;

    for entry in entries {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();

        if !is_wabbajack_file(&filename) {
            skipped += 1;
            continue;
        }

        let mut mod_file = match parse_mod_filename(&filename) {
            Some(mf) => mf,
            None => {
                skipped += 1;
                continue;
            }
        };

        let full_path = entry.path();
        let metadata = fs::metadata(&full_path)?;
        mod_file.full_path = full_path;
        mod_file.size = metadata.len();

        // Create mod key: ModID + normalized ModName + part indicator
        let normalized_name = normalize_mod_name(&mod_file.mod_name);
        let part_indicator = extract_part_indicator(&mod_file.file_name)
            .or_else(|| extract_part_indicator(&mod_file.mod_name))
            .unwrap_or_default();
        let mod_key = format!("{}:{}{}", mod_file.mod_id, normalized_name, part_indicator);

        mod_groups
            .entry(mod_key.clone())
            .or_insert_with(|| ModGroup {
                mod_key,
                files: Vec::new(),
                newest_idx: 0,
                space_to_free: 0,
            })
            .files
            .push(mod_file);
    }

    if skipped > 0 {
        log::info!("Skipped {} files in {:?}", skipped, folder_path);
    }

    // Find duplicates and calculate space
    let mut duplicates = Vec::new();

    for (_, mut group) in mod_groups {
        if group.files.len() <= 1 {
            continue;
        }

        // Check for unique timestamps
        let unique_timestamps: std::collections::HashSet<_> =
            group.files.iter().map(|f| &f.timestamp).collect();

        if unique_timestamps.len() <= 1 {
            log::info!(
                "Skipped group {}: all files have same timestamp",
                group.mod_key
            );
            continue;
        }

        // Sort by timestamp, then version
        group
            .files
            .sort_by(|a, b| match a.timestamp.cmp(&b.timestamp) {
                std::cmp::Ordering::Equal => a.version.cmp(&b.version),
                other => other,
            });

        // Check for suspicious patterns
        if has_suspicious_version_pattern(&group) {
            log::warn!(
                "Skipped group {}: suspicious version pattern",
                group.mod_key
            );
            continue;
        }

        // Check for patch/main file combinations
        let has_patch = group.files.iter().any(|f| f.is_patch);
        let has_main = group
            .files
            .iter()
            .any(|f| is_full_or_main_file(&f.file_name));

        if has_patch && has_main {
            log::warn!(
                "Skipped group {}: contains both PATCH and MAIN files",
                group.mod_key
            );
            continue;
        }

        // Check if newest is a small patch
        let newest = group.files.last().unwrap();
        let mut skip_patch = false;
        if newest.is_patch && group.files.len() > 1 {
            for i in 0..group.files.len() - 1 {
                let old_file = &group.files[i];
                let size_ratio = newest.size as f64 / old_file.size as f64;
                if size_ratio < 0.1 {
                    log::warn!(
                        "Skipped group {}: newest file is likely a patch",
                        group.mod_key
                    );
                    skip_patch = true;
                    break;
                }
            }
        }

        if skip_patch {
            continue;
        }

        // Set newest index and calculate space to free
        group.newest_idx = group.files.len() - 1;
        group.space_to_free = group.files[..group.newest_idx].iter().map(|f| f.size).sum();

        duplicates.push(group);
    }

    let total_files: usize = duplicates.iter().map(|g| g.files.len() - 1).sum();
    let total_space: u64 = duplicates.iter().map(|g| g.space_to_free).sum();

    log::info!("Found {} mod groups with duplicates", duplicates.len());

    Ok(OldVersionScanResult {
        duplicates,
        total_files,
        total_space,
    })
}

/// Calculate library statistics
pub fn calculate_library_stats(game_folders: &[std::path::PathBuf]) -> LibraryStats {
    let results: Vec<(String, usize, u64)> = game_folders
        .par_iter()
        .map(|folder| {
            let entries = match fs::read_dir(folder) {
                Ok(e) => e,
                Err(_) => return ("Unknown".to_string(), 0, 0),
            };

            let mut game_files = 0;
            let mut game_size = 0u64;

            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if entry.file_type().map(|t| t.is_dir()).unwrap_or(true) {
                    continue;
                }

                let filename = entry.file_name().to_string_lossy().to_string();
                if !is_wabbajack_file(&filename) {
                    continue;
                }

                if let Ok(metadata) = entry.metadata() {
                    game_files += 1;
                    game_size += metadata.len();
                }
            }

            let game_name = folder
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            (game_name, game_files, game_size)
        })
        .collect();

    let mut stats = LibraryStats::default();
    for (name, files, size) in results {
        if files > 0 {
            stats.by_game.push((name, files, size));
            stats.total_files += files;
            stats.total_size += size;
        }
    }

    // Sort by game name for consistent display
    stats.by_game.sort_by(|a, b| a.0.cmp(&b.0));

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_detect_orphaned_mods() {
        let mod_files = vec![
            ModFile {
                file_name: "mod1.7z".to_string(),
                full_path: std::path::PathBuf::new(),
                mod_name: "Mod1".to_string(),
                mod_id: "123".to_string(),
                file_id: Some("456".to_string()),
                version: "1.0".to_string(),
                timestamp: "1234567890".to_string(),
                size: 1000,
                is_patch: false,
            },
            ModFile {
                file_name: "mod2.7z".to_string(),
                full_path: std::path::PathBuf::new(),
                mod_name: "Mod2".to_string(),
                mod_id: "123".to_string(),
                file_id: Some("789".to_string()),
                version: "2.0".to_string(),
                timestamp: "1234567891".to_string(),
                size: 2000,
                is_patch: false,
            },
            ModFile {
                file_name: "mod3.7z".to_string(),
                full_path: std::path::PathBuf::new(),
                mod_name: "Mod3".to_string(),
                mod_id: "999".to_string(),
                file_id: None,
                version: "1.0".to_string(),
                timestamp: "1234567892".to_string(),
                size: 3000,
                is_patch: false,
            },
            ModFile {
                file_name: "mod4.7z".to_string(),
                full_path: std::path::PathBuf::new(),
                mod_name: "Mod4".to_string(),
                mod_id: "888".to_string(),
                file_id: Some("111".to_string()),
                version: "1.0".to_string(),
                timestamp: "1234567893".to_string(),
                size: 4000,
                is_patch: false,
            },
        ];

        let mut used_mod_keys = std::collections::HashSet::new();
        used_mod_keys.insert("123".to_string());
        used_mod_keys.insert("999".to_string());

        let mut used_mod_file_ids = std::collections::HashSet::new();
        used_mod_file_ids.insert("123-456".to_string());

        let modlist = ModlistInfo {
            file_path: std::path::PathBuf::new(),
            name: "Test Modlist".to_string(),
            mod_count: 2,
            used_mod_keys,
            used_mod_file_ids,
        };

        let result = detect_orphaned_mods(&mod_files, &[modlist]);

        // mod1, mod2 (both have ModID 123), and mod3 (ModID 999) should be used
        // mod4 (ModID 888) should be orphaned
        assert_eq!(result.orphaned_mods.len(), 1);
        assert_eq!(result.used_mods.len(), 3);
        assert_eq!(result.orphaned_mods[0].file.file_name, "mod4.7z");
    }

    #[test]
    fn test_find_wabbajack_files() {
        let dir = tempdir().unwrap();

        // Create test files
        File::create(dir.path().join("modlist1.wabbajack")).unwrap();
        File::create(dir.path().join("modlist2.wabbajack")).unwrap();
        File::create(dir.path().join("readme.txt")).unwrap();

        let files = find_wabbajack_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_get_all_mod_files() {
        let dir = tempdir().unwrap();
        let game_dir = dir.path().join("Skyrim");
        fs::create_dir(&game_dir).unwrap();

        // Create valid mod files
        let mut file1 = File::create(game_dir.join("SkyUI-12345-5-0-1234567890.7z")).unwrap();
        file1.write_all(b"test content").unwrap();

        let mut file2 = File::create(game_dir.join("SKSE-54321-1-0-9876543210.zip")).unwrap();
        file2.write_all(b"test content 2").unwrap();

        // Create invalid file
        File::create(game_dir.join("readme.txt")).unwrap();

        let files = get_all_mod_files(&[game_dir]).unwrap();
        assert_eq!(files.len(), 2);
    }
}
