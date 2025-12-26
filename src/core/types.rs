// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::collections::HashSet;
use std::path::PathBuf;

/// Represents a parsed mod file from the downloads folder
#[derive(Debug, Clone)]
pub struct ModFile {
    pub file_name: String,
    pub full_path: PathBuf,
    pub mod_name: String,
    pub mod_id: String,
    pub file_id: Option<String>,
    pub version: String,
    pub timestamp: String,
    pub size: u64,
    pub is_patch: bool,
}

/// Represents a group of mod versions (same mod, different versions)
#[derive(Debug, Clone)]
pub struct ModGroup {
    pub mod_key: String,
    pub files: Vec<ModFile>,
    pub newest_idx: usize,
    pub space_to_free: u64,
}

/// Information about a parsed .wabbajack modlist file
#[derive(Debug, Clone)]
pub struct ModlistInfo {
    #[allow(dead_code)]
    pub file_path: PathBuf,
    pub name: String,
    pub mod_count: usize,
    /// ModID-based keys for quick lookup (backward compatibility)
    pub used_mod_keys: HashSet<String>,
    /// ModID+FileID combination for precise matching
    pub used_mod_file_ids: HashSet<String>,
}

/// Represents a mod file that's not used by any active modlist
#[derive(Debug, Clone)]
pub struct OrphanedMod {
    pub file: ModFile,
}

/// Archive extensions supported by Wabbajack
pub const ARCHIVE_EXTENSIONS: &[&str] = &[".7z", ".zip", ".rar", ".tar", ".gz", ".exe"];

/// Result of a scan operation
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub used_mods: Vec<ModFile>,
    pub orphaned_mods: Vec<OrphanedMod>,
    pub used_size: u64,
    pub orphaned_size: u64,
}

/// Result of old version scan
#[derive(Debug, Clone)]
pub struct OldVersionScanResult {
    pub duplicates: Vec<ModGroup>,
    pub total_files: usize,
    pub total_space: u64,
}

/// Deletion result
#[derive(Debug, Clone, Default)]
pub struct DeletionResult {
    pub deleted_count: usize,
    pub space_freed: u64,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
}

/// Statistics about the mod library
#[derive(Debug, Clone, Default)]
pub struct LibraryStats {
    pub total_files: usize,
    pub total_size: u64,
    pub by_game: Vec<(String, usize, u64)>,
}
