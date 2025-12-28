// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use zip::ZipArchive;

use crate::core::types::{ModFile, ModlistInfo, ARCHIVE_EXTENSIONS};

/// JSON structures for parsing .wabbajack files
#[derive(Debug, Deserialize)]
struct Modlist {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    #[allow(dead_code)]
    version: Option<String>,
    #[serde(rename = "Author")]
    #[allow(dead_code)]
    author: Option<String>,
    #[serde(rename = "Archives")]
    archives: Vec<ModlistArchive>,
}

#[derive(Debug, Deserialize)]
struct ModlistArchive {
    #[serde(rename = "Hash")]
    #[allow(dead_code)]
    hash: Option<String>,
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(rename = "Size")]
    #[allow(dead_code)]
    size: Option<i64>,
    #[serde(rename = "State")]
    state: ModlistModState,
}

#[derive(Debug, Deserialize)]
struct ModlistModState {
    #[serde(rename = "$type")]
    #[allow(dead_code)]
    type_name: Option<String>,
    #[serde(rename = "ModID")]
    mod_id: Option<i64>,
    #[serde(rename = "FileID")]
    file_id: Option<i64>,
    #[serde(rename = "GameName")]
    #[allow(dead_code)]
    game_name: Option<String>,
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(rename = "Version")]
    #[allow(dead_code)]
    version: Option<String>,
}

/// Check if a string contains only digits (optionally with leading minus)
pub fn is_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let s = s.strip_prefix('-').unwrap_or(s);
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Check if a string looks like a version pattern (e.g., "1.2.3", "v1.0")
pub fn is_version_pattern(s: &str) -> bool {
    let s = s.to_lowercase();
    let s = s.strip_prefix('v').unwrap_or(&s);

    let mut has_digit = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            has_digit = true;
        } else if c != '.' && c != '-' && c != '_' {
            return false;
        }
    }
    has_digit
}

/// Normalize mod name by removing trailing version patterns
pub fn normalize_mod_name(mod_name: &str) -> String {
    let parts: Vec<&str> = mod_name.split(' ').collect();
    let mut clean_parts = Vec::new();

    for part in parts {
        if is_version_pattern(part) {
            break;
        }
        clean_parts.push(part);
    }

    if clean_parts.is_empty() {
        mod_name.to_string()
    } else {
        clean_parts.join(" ")
    }
}

/// Detect if a filename indicates a patch/hotfix/update file
pub fn is_patch_or_hotfix(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    let patch_keywords = [
        "patch", "hotfix", "update", "fix", "- patch", "-patch", " patch", "- hotfix", "-hotfix",
        " hotfix", "- update", "-update", " update", "- fix", "-fix", " fix",
    ];

    patch_keywords.iter().any(|kw| lower.contains(kw))
}

/// Detect if a filename indicates a full/main file
pub fn is_full_or_main_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    let full_keywords = ["main", "full", "complete", "- main", "-main", " main"];

    full_keywords.iter().any(|kw| lower.contains(kw))
}

/// Extract part indicator from filename (e.g., "-1-", "Part 1")
pub fn extract_part_indicator(filename: &str) -> Option<String> {
    let lower = filename.to_lowercase();

    // Pattern 1: "-1-", "-2-", etc.
    for i in 1..=20 {
        let pattern = format!("-{}-", i);
        if let Some(idx) = lower.find(&pattern) {
            // Check what comes before
            if idx > 0 {
                let prev_char = lower.chars().nth(idx - 1).unwrap_or(' ');
                if prev_char.is_alphabetic() || prev_char.is_ascii_digit() {
                    continue;
                }
            }

            // Check what comes after
            let after_idx = idx + pattern.len();
            if after_idx < lower.len() {
                let next_char = lower.chars().nth(after_idx).unwrap_or(' ');
                if !next_char.is_ascii_digit() {
                    return Some(pattern);
                }
            } else {
                return Some(pattern);
            }
        }
    }

    // Pattern 2: "part 1", "part1", etc.
    for i in (1..=20).rev() {
        let patterns = [
            format!("part {}", i),
            format!("part{}", i),
            format!("(part {})", i),
            format!("pt{}", i),
            format!("pt {}", i),
        ];

        for pattern in patterns {
            if lower.contains(&pattern) {
                return Some(format!(":part{}", i));
            }
        }
    }

    None
}

/// Check if a file has a valid archive extension
pub fn has_valid_archive_extension(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    ARCHIVE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Check if a file is a valid Wabbajack mod file
pub fn is_wabbajack_file(filename: &str) -> bool {
    if !has_valid_archive_extension(filename) {
        return false;
    }

    let lower = filename.to_lowercase();
    if lower.contains(".part")
        || lower.contains(".tmp")
        || lower.contains(".download")
        || lower.starts_with('~')
    {
        return false;
    }

    true
}

/// Parse a mod filename into its components
pub fn parse_mod_filename(filename: &str) -> Option<ModFile> {
    // Check extension
    let ext = ARCHIVE_EXTENSIONS
        .iter()
        .find(|ext| filename.to_lowercase().ends_with(*ext))?;

    // Remove extension
    let name_without_ext = &filename[..filename.len() - ext.len()];

    // Split by dash
    let parts: Vec<&str> = name_without_ext.split('-').collect();
    if parts.len() < 3 {
        return None;
    }

    // Last part should be timestamp (10+ digit number)
    let timestamp = *parts.last()?;
    if !is_numeric(timestamp) || timestamp.len() < 10 {
        return None;
    }

    // Find ModID (3-6 digit number in parts[1:len-1])
    let mut mod_id = None;
    let mut mod_id_index = None;

    for (i, part) in parts.iter().enumerate().take(parts.len() - 1).skip(1) {
        if is_numeric(part) && (3..=6).contains(&part.len()) {
            mod_id = Some(part.to_string());
            mod_id_index = Some(i);
            break;
        }
    }

    let mod_id = mod_id?;
    let mod_id_index = mod_id_index?;

    // Try to find FileID (numeric part after ModID, typically 4-7 digits)
    let mut file_id = None;
    let mut file_id_index = None;

    if mod_id_index + 1 < parts.len() - 1 {
        let next_part = parts[mod_id_index + 1];
        if is_numeric(next_part) && next_part.len() >= 4 {
            file_id = Some(next_part.to_string());
            file_id_index = Some(mod_id_index + 1);
        }
    }

    // ModName = parts[0:mod_id_index]
    let mod_name = parts[..mod_id_index].join("-");

    // Version = parts after ModID (and FileID if present) until timestamp
    let version_start = file_id_index.map(|i| i + 1).unwrap_or(mod_id_index + 1);
    let version = parts[version_start..parts.len() - 1].join("-");

    Some(ModFile {
        file_name: filename.to_string(),
        full_path: std::path::PathBuf::new(),
        mod_name,
        mod_id,
        file_id,
        version,
        timestamp: timestamp.to_string(),
        size: 0,
        is_patch: is_patch_or_hotfix(filename),
    })
}

/// Parse a .wabbajack file and extract modlist information
pub fn parse_wabbajack_file(file_path: &Path) -> Result<ModlistInfo> {
    log::info!("Parsing wabbajack file: {:?}", file_path);

    let file = File::open(file_path)
        .with_context(|| format!("Failed to open wabbajack file: {:?}", file_path))?;

    let mut archive =
        ZipArchive::new(file).with_context(|| "Failed to read wabbajack file as ZIP")?;

    // Find and read the "modlist" file
    let mut modlist_content = String::new();
    {
        let mut modlist_file = archive
            .by_name("modlist")
            .with_context(|| "modlist file not found in archive")?;
        modlist_file
            .read_to_string(&mut modlist_content)
            .with_context(|| "Failed to read modlist file")?;
    }

    let modlist: Modlist =
        serde_json::from_str(&modlist_content).with_context(|| "Failed to parse modlist JSON")?;

    // Build sets for used mods
    let mut used_mod_keys = HashSet::new();
    let mut used_mod_file_ids = HashSet::new();
    let mut used_file_names = HashSet::new();

    for arch in &modlist.archives {
        // Collect exact file names for precise matching
        if let Some(ref name) = arch.name {
            if !name.is_empty() {
                used_file_names.insert(name.clone());
            }
        }
        
        if let Some(mod_id) = arch.state.mod_id {
            if mod_id > 0 {
                // ModID-only key (backward compatibility)
                used_mod_keys.insert(mod_id.to_string());

                // ModID+FileID combination key for precise matching
                if let Some(file_id) = arch.state.file_id {
                    if file_id > 0 {
                        used_mod_file_ids.insert(format!("{}-{}", mod_id, file_id));
                    }
                }
            }
        }
    }

    log::info!(
        "Parsed modlist '{}': {} archives, {} unique ModIDs, {} file names",
        modlist.name,
        modlist.archives.len(),
        used_mod_keys.len(),
        used_file_names.len()
    );

    Ok(ModlistInfo {
        file_path: file_path.to_path_buf(),
        name: modlist.name,
        mod_count: modlist.archives.len(),
        used_mod_keys,
        used_mod_file_ids,
        used_file_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numeric() {
        assert!(is_numeric("123"));
        assert!(is_numeric("0"));
        assert!(is_numeric("-123"));
        assert!(!is_numeric("abc"));
        assert!(!is_numeric("12a"));
        assert!(!is_numeric(""));
    }

    #[test]
    fn test_is_version_pattern() {
        assert!(is_version_pattern("v1.0"));
        assert!(is_version_pattern("V2.3.4"));
        assert!(is_version_pattern("1.0"));
        assert!(is_version_pattern("0.18"));
        assert!(is_version_pattern("2-0-1"));
        assert!(!is_version_pattern("Part1"));
        assert!(!is_version_pattern("Main"));
        assert!(!is_version_pattern("abc"));
    }

    #[test]
    fn test_normalize_mod_name() {
        assert_eq!(normalize_mod_name("Interface v1.0"), "Interface");
        assert_eq!(normalize_mod_name("Simple Mod V2.0"), "Simple Mod");
        assert_eq!(normalize_mod_name("No Version Mod"), "No Version Mod");
        assert_eq!(normalize_mod_name("Mod 0.18"), "Mod");
    }

    #[test]
    fn test_is_patch_or_hotfix() {
        assert!(is_patch_or_hotfix("SkyUI-Patch.7z"));
        assert!(is_patch_or_hotfix("Mod-Hotfix-123.zip"));
        assert!(is_patch_or_hotfix("Update-Main.rar"));
        assert!(is_patch_or_hotfix("Bug-Fix-1.0.7z"));
        assert!(!is_patch_or_hotfix("Main File.7z"));
        assert!(!is_patch_or_hotfix("Normal Mod-123-1-0.7z"));
    }

    #[test]
    fn test_parse_mod_filename() {
        // Valid filename with ModID and FileID
        let result = parse_mod_filename("Skyrim 2020-12345-67890-1-0-1234567890.7z");
        assert!(result.is_some());
        let mod_file = result.unwrap();
        assert_eq!(mod_file.mod_id, "12345");
        assert_eq!(mod_file.file_id, Some("67890".to_string()));

        // Valid filename with ModID only
        let result = parse_mod_filename("Simple Mod-123-1-0-1234567890.rar");
        assert!(result.is_some());
        let mod_file = result.unwrap();
        assert_eq!(mod_file.mod_id, "123");
        assert_eq!(mod_file.file_id, None);

        // Invalid filename - no ModID
        assert!(parse_mod_filename("NoModID-1234567890.7z").is_none());

        // Invalid filename - wrong extension
        assert!(parse_mod_filename("Mod-123-1-0-1234567890.txt").is_none());
    }

    #[test]
    fn test_is_wabbajack_file() {
        assert!(is_wabbajack_file("Mod-123-1-0-1234567890.7z"));
        assert!(is_wabbajack_file("Test-456-2-0.zip"));
        assert!(!is_wabbajack_file("readme.txt"));
        assert!(!is_wabbajack_file("mod.part.7z"));
        assert!(!is_wabbajack_file("~temp.zip"));
    }
}
