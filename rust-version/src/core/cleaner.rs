// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::fs;
use std::path::Path;

use crate::core::types::{DeletionResult, ModFile, ModGroup, OrphanedMod};

/// Check if a file is locked (being used by another process)
pub fn is_file_locked(path: &Path) -> bool {
    // Try to open the file for writing
    match fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
    {
        Ok(_) => false,
        Err(_) => true,
    }
}

/// Delete a single mod file and its associated .meta file
fn delete_mod_file(file: &ModFile, backup_dir: Option<&Path>) -> Result<u64, String> {
    let path = &file.full_path;
    
    if !path.exists() {
        return Err(format!("File no longer exists: {:?}", path));
    }
    
    if is_file_locked(path) {
        return Err(format!("File is locked: {:?}", path));
    }
    
    if let Some(backup) = backup_dir {
        // Move to backup folder
        let backup_path = backup.join(&file.file_name);
        fs::rename(path, &backup_path)
            .map_err(|e| format!("Failed to move file: {}", e))?;
        
        // Also move .meta file if exists
        let meta_full = format!("{}.meta", path.display());
        let meta_path = Path::new(&meta_full);
        
        if meta_path.exists() {
            let backup_meta = backup.join(format!("{}.meta", file.file_name));
            let _ = fs::rename(meta_path, backup_meta);
        }
        
        log::info!("Moved to backup: {} ({})", file.file_name, format_size(file.size));
    } else {
        // Permanently delete
        fs::remove_file(path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
        
        // Also delete .meta file if exists
        let meta_full = format!("{}.meta", path.display());
        let meta_path = Path::new(&meta_full);
        if meta_path.exists() {
            let _ = fs::remove_file(meta_path);
        }
        
        log::info!("Deleted: {} ({})", file.file_name, format_size(file.size));
    }
    
    Ok(file.size)
}

/// Delete orphaned mods
pub fn delete_orphaned_mods(
    orphaned_mods: &[OrphanedMod],
    backup_dir: Option<&Path>,
    progress_callback: Option<&dyn Fn(usize, usize)>,
) -> DeletionResult {
    let mut result = DeletionResult::default();
    let total = orphaned_mods.len();
    
    // Create backup directory if specified
    if let Some(backup) = backup_dir {
        if let Err(e) = fs::create_dir_all(backup) {
            result.errors.push(format!("Failed to create backup folder: {}", e));
            return result;
        }
        log::info!("Created backup folder: {:?}", backup);
    }
    
    for (i, orphaned) in orphaned_mods.iter().enumerate() {
        if let Some(cb) = progress_callback {
            cb(i + 1, total);
        }
        
        match delete_mod_file(&orphaned.file, backup_dir) {
            Ok(size) => {
                result.deleted_count += 1;
                result.space_freed += size;
            }
            Err(e) => {
                result.skipped.push(orphaned.file.file_name.clone());
                result.errors.push(e);
            }
        }
    }
    
    result
}

/// Delete old versions from mod groups
pub fn delete_old_versions(
    duplicates: &[ModGroup],
    backup_dir: Option<&Path>,
    progress_callback: Option<&dyn Fn(usize, usize)>,
) -> DeletionResult {
    let mut result = DeletionResult::default();
    
    // Collect all files to delete
    let files_to_delete: Vec<&ModFile> = duplicates
        .iter()
        .flat_map(|group| group.files[..group.newest_idx].iter())
        .collect();
    
    let total = files_to_delete.len();
    
    // Create backup directory if specified
    if let Some(backup) = backup_dir {
        if let Err(e) = fs::create_dir_all(backup) {
            result.errors.push(format!("Failed to create backup folder: {}", e));
            return result;
        }
        log::info!("Created backup folder: {:?}", backup);
    }
    
    for (i, file) in files_to_delete.iter().enumerate() {
        if let Some(cb) = progress_callback {
            cb(i + 1, total);
        }
        
        // Validate before deletion
        if !validate_deletion_safety(duplicates, file) {
            result.skipped.push(file.file_name.clone());
            result.errors.push(format!("Safety check failed for: {}", file.file_name));
            continue;
        }
        
        match delete_mod_file(file, backup_dir) {
            Ok(size) => {
                result.deleted_count += 1;
                result.space_freed += size;
            }
            Err(e) => {
                result.skipped.push(file.file_name.clone());
                result.errors.push(e);
            }
        }
    }
    
    result
}

/// Validate that we're not deleting the newest file in a group
fn validate_deletion_safety(duplicates: &[ModGroup], file: &ModFile) -> bool {
    for group in duplicates {
        if group.files.len() <= 1 {
            continue;
        }
        
        // Find if this file is in the group
        let file_idx = group.files.iter().position(|f| f.full_path == file.full_path);
        
        if let Some(idx) = file_idx {
            // Make sure we're not deleting the newest
            if idx >= group.newest_idx {
                log::error!("Safety check failed: Attempting to delete newest file in group {}", group.mod_key);
                return false;
            }
            
            // Verify newest still exists
            let newest = &group.files[group.newest_idx];
            if !newest.full_path.exists() {
                log::error!("Newest file doesn't exist: {:?}", newest.full_path);
                return false;
            }
        }
    }
    
    true
}

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Convert timestamp to human-readable date
#[allow(dead_code)]
pub fn timestamp_to_date(timestamp: &str) -> String {
    timestamp
        .parse::<i64>()
        .ok()
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1536 * 1024), "1.50 MB");
    }

    #[test]
    fn test_timestamp_to_date() {
        assert_eq!(timestamp_to_date("1234567890"), "2009-02-13 23:31");
        assert_eq!(timestamp_to_date("invalid"), "Unknown");
    }

    #[test]
    fn test_is_file_locked() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test").unwrap();
        drop(file);
        
        // File should not be locked after closing
        assert!(!is_file_locked(&file_path));
    }

    #[test]
    fn test_delete_mod_file_permanent() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test-123-1-0-1234567890.7z");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        let mod_file = ModFile {
            file_name: "test-123-1-0-1234567890.7z".to_string(),
            full_path: file_path.clone(),
            mod_name: "test".to_string(),
            mod_id: "123".to_string(),
            file_id: None,
            version: "1-0".to_string(),
            timestamp: "1234567890".to_string(),
            size: 12,
            is_patch: false,
        };
        
        let result = delete_mod_file(&mod_file, None);
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_mod_file_with_backup() {
        let dir = tempdir().unwrap();
        let backup_dir = dir.path().join("backup");
        let file_path = dir.path().join("test-123-1-0-1234567890.7z");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        fs::create_dir(&backup_dir).unwrap();
        
        let mod_file = ModFile {
            file_name: "test-123-1-0-1234567890.7z".to_string(),
            full_path: file_path.clone(),
            mod_name: "test".to_string(),
            mod_id: "123".to_string(),
            file_id: None,
            version: "1-0".to_string(),
            timestamp: "1234567890".to_string(),
            size: 12,
            is_patch: false,
        };
        
        let result = delete_mod_file(&mod_file, Some(&backup_dir));
        assert!(result.is_ok());
        assert!(!file_path.exists());
        assert!(backup_dir.join("test-123-1-0-1234567890.7z").exists());
    }
}
