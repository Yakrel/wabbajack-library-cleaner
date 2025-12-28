// Integration tests for Wabbajack Library Cleaner
// Tests real-world scenarios using synthetic test files that mimic actual Wabbajack mod structures

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use wabbajack_library_cleaner::core::{
    delete_old_versions, delete_orphaned_mods, detect_orphaned_mods, get_all_mod_files,
    parse_wabbajack_file, scan_folder_for_duplicates, OrphanedMod,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a dummy .wabbajack file with realistic structure matching real wabbajack files
/// Real format example: "Mod Organizer 2.5.2-ML1.5 Archive-874-2-5-2-ML1-5-1723841178.7z"
fn create_dummy_wabbajack(path: &Path, archives: &[TestArchive]) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options: SimpleFileOptions =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("modlist", options).unwrap();

    // Create realistic JSON structure matching actual wabbajack files
    let mut archives_json = String::from("[");
    for (i, archive) in archives.iter().enumerate() {
        if i > 0 {
            archives_json.push(',');
        }
        archives_json.push_str(&format!(
            r#"{{
                "Hash": "{}",
                "Name": "{}",
                "Size": {},
                "State": {{
                    "$type": "NexusDownloader, Wabbajack.Lib",
                    "ModID": {},
                    "FileID": {},
                    "GameName": "{}",
                    "Name": "{}",
                    "Version": "{}"
                }}
            }}"#,
            archive.hash,
            archive.filename,
            archive.size,
            archive.mod_id,
            archive.file_id,
            archive.game_name,
            archive.mod_name,
            archive.version
        ));
    }
    archives_json.push(']');

    let json = format!(
        r#"{{
            "Name": "{}",
            "Version": "1.0.0",
            "Author": "TestAuthor",
            "Archives": {}
        }}"#,
        path.file_stem().unwrap().to_string_lossy(),
        archives_json
    );

    zip.write_all(json.as_bytes()).unwrap();
    zip.finish().unwrap();
}

/// Test archive data structure
struct TestArchive {
    filename: String,
    mod_id: i64,
    file_id: i64,
    game_name: String,
    mod_name: String,
    version: String,
    hash: String,
    size: u64,
}

impl TestArchive {
    fn new(mod_name: &str, mod_id: i64, file_id: i64, version: &str, timestamp: &str) -> Self {
        let filename = format!(
            "{}-{}-{}-{}-{}.7z",
            mod_name.replace(' ', "_"),
            mod_id,
            file_id,
            version.replace('.', "-"),
            timestamp
        );
        Self {
            filename,
            mod_id,
            file_id,
            game_name: "SkyrimSpecialEdition".to_string(),
            mod_name: mod_name.to_string(),
            version: version.to_string(),
            hash: format!("hash{}{}", mod_id, file_id),
            size: 1000000,
        }
    }
}

/// Create a dummy mod file on disk with realistic naming pattern
/// Real example: "SkyUI-12604-5-2-SE-1615410779.7z"
fn create_mod_file(
    dir: &Path,
    name: &str,
    mod_id: i64,
    file_id: i64,
    version: &str,
    timestamp: &str,
    size: usize,
) {
    let filename = format!(
        "{}-{}-{}-{}-{}.7z",
        name.replace(' ', "_"),
        mod_id,
        file_id,
        version.replace('.', "-"),
        timestamp
    );
    let path = dir.join(&filename);
    let mut file = File::create(&path).unwrap();

    // Write content to achieve desired size
    let content = vec![b'x'; size];
    file.write_all(&content).unwrap();
}

/// Create a mod file with simple name format (for old version tests)
fn create_simple_mod_file(dir: &Path, filename: &str, size: usize) {
    let path = dir.join(filename);
    let mut file = File::create(&path).unwrap();
    let content = vec![b'x'; size];
    file.write_all(&content).unwrap();
}

// ============================================================================
// ORPHANED MOD DETECTION TESTS
// ============================================================================

#[test]
fn test_orphan_detection_basic() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    fs::create_dir(&downloads_dir).unwrap();
    fs::create_dir(&wabbajack_dir).unwrap();

    // Create wabbajack file with 2 mods
    let wabbajack_file = wabbajack_dir.join("TestModlist.wabbajack");
    create_dummy_wabbajack(
        &wabbajack_file,
        &[
            TestArchive::new("SkyUI", 12604, 52344, "5.2", "1615410779"),
            TestArchive::new("SKSE64", 30379, 111593, "2.0.20", "1622656000"),
        ],
    );

    // Create mod files on disk - some used, some orphaned
    // USED: SkyUI (exact file name matches modlist)
    create_mod_file(
        &downloads_dir,
        "SkyUI",
        12604,
        52344,
        "5.2",
        "1615410779",
        1000,
    );

    // USED: SKSE64 (exact file name matches modlist)
    create_mod_file(
        &downloads_dir,
        "SKSE64",
        30379,
        111593,
        "2.0.20",
        "1622656000",
        2000,
    );

    // ORPHANED: Random mod not in modlist
    create_mod_file(
        &downloads_dir,
        "UnusedMod",
        99999,
        88888,
        "1.0",
        "1600000000",
        500,
    );

    // ORPHANED: Different version of SkyUI (different file name - different FileID/version)
    create_mod_file(
        &downloads_dir,
        "SkyUI",
        12604,
        40000,
        "4.1",
        "1500000000",
        800,
    );

    // Parse and detect
    let modlist_info = parse_wabbajack_file(&wabbajack_file).unwrap();
    let game_folders = vec![downloads_dir.clone()];
    let all_files = get_all_mod_files(&game_folders).unwrap();
    let scan_result = detect_orphaned_mods(&all_files, &[modlist_info]);

    // Verify results
    assert_eq!(all_files.len(), 4, "Should find 4 files on disk");

    // With file name matching:
    // - SkyUI (5.2, 52344) -> USED (exact match)
    // - SKSE64 (2.0.20, 111593) -> USED (exact match)
    // - UnusedMod -> ORPHANED (not in modlist)
    // - SkyUI (4.1, 40000) -> ORPHANED (different file name)
    assert_eq!(
        scan_result.used_mods.len(),
        2,
        "Should have 2 used mods (exact file name matches)"
    );
    assert_eq!(
        scan_result.orphaned_mods.len(),
        2,
        "Should have 2 orphaned mods (unknown mod + old SkyUI version)"
    );

    // Verify specific files
    assert!(
        scan_result
            .used_mods
            .iter()
            .any(|f| f.mod_id == "12604" && f.file_id == Some("52344".to_string())),
        "SkyUI 5.2 should be marked as used"
    );
    assert!(
        scan_result
            .orphaned_mods
            .iter()
            .any(|f| f.file.mod_id == "99999"),
        "UnusedMod should be orphaned"
    );
}

#[test]
fn test_orphan_detection_multiple_modlists() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    fs::create_dir(&downloads_dir).unwrap();
    fs::create_dir(&wabbajack_dir).unwrap();

    // Create two modlists with overlapping mods
    let modlist1 = wabbajack_dir.join("Modlist1.wabbajack");
    create_dummy_wabbajack(
        &modlist1,
        &[
            TestArchive::new("SharedMod", 1000, 1001, "1.0", "1600000000"),
            TestArchive::new("Modlist1Only", 2000, 2001, "1.0", "1600000000"),
        ],
    );

    let modlist2 = wabbajack_dir.join("Modlist2.wabbajack");
    create_dummy_wabbajack(
        &modlist2,
        &[
            TestArchive::new("SharedMod", 1000, 1001, "1.0", "1600000000"),
            TestArchive::new("Modlist2Only", 3000, 3001, "1.0", "1600000000"),
        ],
    );

    // Create files
    create_mod_file(
        &downloads_dir,
        "SharedMod",
        1000,
        1001,
        "1.0",
        "1600000000",
        1000,
    );
    create_mod_file(
        &downloads_dir,
        "Modlist1Only",
        2000,
        2001,
        "1.0",
        "1600000000",
        1000,
    );
    create_mod_file(
        &downloads_dir,
        "Modlist2Only",
        3000,
        3001,
        "1.0",
        "1600000000",
        1000,
    );
    create_mod_file(
        &downloads_dir,
        "OrphanedMod",
        9000,
        9001,
        "1.0",
        "1600000000",
        1000,
    );

    // Parse both modlists
    let info1 = parse_wabbajack_file(&modlist1).unwrap();
    let info2 = parse_wabbajack_file(&modlist2).unwrap();

    let game_folders = vec![downloads_dir.clone()];
    let all_files = get_all_mod_files(&game_folders).unwrap();
    let scan_result = detect_orphaned_mods(&all_files, &[info1, info2]);

    assert_eq!(
        scan_result.used_mods.len(),
        3,
        "3 mods should be used across both modlists"
    );
    assert_eq!(
        scan_result.orphaned_mods.len(),
        1,
        "Only 1 mod should be orphaned"
    );
    assert_eq!(scan_result.orphaned_mods[0].file.mod_id, "9000");
}

#[test]
fn test_orphan_detection_modid_fallback() {
    // Tests that with file name matching, different FileID = orphaned (not fallback)
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    fs::create_dir(&downloads_dir).unwrap();
    fs::create_dir(&wabbajack_dir).unwrap();

    let wabbajack_file = wabbajack_dir.join("Test.wabbajack");
    create_dummy_wabbajack(
        &wabbajack_file,
        &[TestArchive::new("TestMod", 5000, 5001, "2.0", "1600000000")],
    );

    // Create mod with same ModID but different FileID (different file name)
    create_mod_file(
        &downloads_dir,
        "TestMod",
        5000,
        9999,
        "1.0",
        "1500000000",
        1000,
    );

    let modlist_info = parse_wabbajack_file(&wabbajack_file).unwrap();
    let all_files = get_all_mod_files(&[downloads_dir]).unwrap();
    let scan_result = detect_orphaned_mods(&all_files, &[modlist_info]);

    // With file name matching, different FileID = different file name = ORPHANED
    assert_eq!(
        scan_result.used_mods.len(),
        0,
        "File with different file name should NOT be used"
    );
    assert_eq!(scan_result.orphaned_mods.len(), 1, "Should be orphaned");
}

// ============================================================================
// OLD VERSION DETECTION TESTS
// ============================================================================

#[test]
fn test_old_version_detection_basic() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&downloads_dir).unwrap();

    // Create same mod with different versions (different timestamps = different versions)
    create_simple_mod_file(&downloads_dir, "SkyUI-12604-52344-5-0-1600000000.7z", 1000);
    create_simple_mod_file(&downloads_dir, "SkyUI-12604-52344-5-1-1610000000.7z", 1000);
    create_simple_mod_file(&downloads_dir, "SkyUI-12604-52344-5-2-1620000000.7z", 1000);

    let result = scan_folder_for_duplicates(&downloads_dir).unwrap();

    assert_eq!(result.duplicates.len(), 1, "Should find 1 duplicate group");
    assert_eq!(result.total_files, 2, "Should mark 2 files as old versions");

    let group = &result.duplicates[0];
    assert_eq!(group.files.len(), 3, "Group should have 3 files");
    assert_eq!(
        group.newest_idx, 2,
        "Newest should be the last file (index 2)"
    );
}

#[test]
fn test_old_version_keeps_newest() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&downloads_dir).unwrap();

    // Timestamps: oldest first, newest last
    create_simple_mod_file(&downloads_dir, "TestMod-1000-2000-1-0-1500000000.7z", 500);
    create_simple_mod_file(&downloads_dir, "TestMod-1000-2000-1-1-1600000000.7z", 500);
    create_simple_mod_file(&downloads_dir, "TestMod-1000-2000-1-2-1700000000.7z", 500);

    let result = scan_folder_for_duplicates(&downloads_dir).unwrap();

    assert!(!result.duplicates.is_empty());
    let group = &result.duplicates[0];

    // Verify the newest file (timestamp 1700000000) is marked to keep
    let newest_file = &group.files[group.newest_idx];
    assert!(
        newest_file.timestamp == "1700000000",
        "Newest file should have timestamp 1700000000, got {}",
        newest_file.timestamp
    );
}

#[test]
fn test_different_mods_not_grouped() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&downloads_dir).unwrap();

    // Different ModIDs - should NOT be grouped together
    create_simple_mod_file(&downloads_dir, "ModA-1000-2000-1-0-1600000000.7z", 500);
    create_simple_mod_file(&downloads_dir, "ModB-1001-2001-1-0-1600000000.7z", 500);
    create_simple_mod_file(&downloads_dir, "ModC-1002-2002-1-0-1600000000.7z", 500);

    let result = scan_folder_for_duplicates(&downloads_dir).unwrap();

    assert_eq!(
        result.duplicates.len(),
        0,
        "Different mods should not be grouped"
    );
}

#[test]
fn test_patch_and_main_not_grouped() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&downloads_dir).unwrap();

    // Main file and patch should NOT be deleted together
    create_simple_mod_file(&downloads_dir, "MainMod-1000-2000-1-0-1600000000.7z", 10000);
    create_simple_mod_file(
        &downloads_dir,
        "MainMod Patch-1000-2001-1-0-1700000000.7z",
        500,
    );

    let result = scan_folder_for_duplicates(&downloads_dir).unwrap();

    // Should either not group them or skip the group due to patch detection
    for group in &result.duplicates {
        let has_patch = group.files.iter().any(|f| f.is_patch);
        let has_main = group.files.iter().any(|f| !f.is_patch);
        assert!(
            !(has_patch && has_main),
            "Patch and main files should not be in same deletion group"
        );
    }
}

// ============================================================================
// DELETION SAFETY TESTS
// ============================================================================

#[test]
fn test_delete_orphaned_to_backup() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let backup_dir = temp_dir.path().join("backup");
    fs::create_dir(&downloads_dir).unwrap();

    // Create file to delete
    let filename = "OrphanMod-9999-8888-1-0-1234567890.7z";
    create_simple_mod_file(&downloads_dir, filename, 1000);

    let files = get_all_mod_files(&[downloads_dir.clone()]).unwrap();
    let orphaned = OrphanedMod {
        file: files[0].clone(),
    };

    // Delete with backup
    let result = delete_orphaned_mods(&[orphaned], Some(&backup_dir), None);

    assert_eq!(result.deleted_count, 1);
    assert_eq!(result.errors.len(), 0);

    // Original should be gone
    assert!(!downloads_dir.join(filename).exists());

    // Backup should exist
    assert!(backup_dir.join(filename).exists());
}

#[test]
fn test_delete_orphaned_permanent() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&downloads_dir).unwrap();

    let filename = "ToDelete-9999-8888-1-0-1234567890.7z";
    create_simple_mod_file(&downloads_dir, filename, 1000);

    let files = get_all_mod_files(&[downloads_dir.clone()]).unwrap();
    let orphaned = OrphanedMod {
        file: files[0].clone(),
    };

    // Delete without backup (permanent)
    let result = delete_orphaned_mods(&[orphaned], None, None);

    assert_eq!(result.deleted_count, 1);
    assert!(!downloads_dir.join(filename).exists());
}

#[test]
fn test_delete_old_versions_safety() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let backup_dir = temp_dir.path().join("backup");
    fs::create_dir(&downloads_dir).unwrap();

    // Create version history
    create_simple_mod_file(&downloads_dir, "TestMod-1000-2000-1-0-1500000000.7z", 1000);
    create_simple_mod_file(&downloads_dir, "TestMod-1000-2000-1-1-1600000000.7z", 1000);
    create_simple_mod_file(&downloads_dir, "TestMod-1000-2000-1-2-1700000000.7z", 1000);

    let scan_result = scan_folder_for_duplicates(&downloads_dir).unwrap();

    // Delete old versions
    let deletion_result = delete_old_versions(&scan_result.duplicates, Some(&backup_dir), None);

    assert_eq!(
        deletion_result.deleted_count, 2,
        "Should delete 2 old versions"
    );

    // Newest should still exist
    assert!(
        downloads_dir
            .join("TestMod-1000-2000-1-2-1700000000.7z")
            .exists(),
        "Newest version should be kept"
    );

    // Old versions should be in backup
    assert!(backup_dir
        .join("TestMod-1000-2000-1-0-1500000000.7z")
        .exists());
    assert!(backup_dir
        .join("TestMod-1000-2000-1-1-1600000000.7z")
        .exists());
}

// ============================================================================
// REAL WABBAJACK FILE STRUCTURE TEST (using sample fixture)
// ============================================================================

#[test]
fn test_parse_sample_modlist_structure() {
    // Test with the sample modlist JSON fixture (extracted from a real wabbajack file)
    let sample_path = std::path::Path::new("tests/fixtures/sample_modlist.json");

    if !sample_path.exists() {
        println!("Skipping sample modlist test - fixture not present");
        return;
    }

    // Read sample JSON
    let json_content = fs::read_to_string(sample_path).expect("Failed to read sample modlist");

    // Create a temporary wabbajack file from the sample JSON
    let temp_dir = TempDir::new().unwrap();
    let wabbajack_path = temp_dir.path().join("sample.wabbajack");

    {
        let file = File::create(&wabbajack_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("modlist", options).unwrap();
        zip.write_all(json_content.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    // Parse the wabbajack file
    let result = parse_wabbajack_file(&wabbajack_path);
    assert!(
        result.is_ok(),
        "Should parse sample wabbajack file: {:?}",
        result.err()
    );

    let modlist_info = result.unwrap();

    // Verify parsing worked correctly
    assert_eq!(
        modlist_info.name, "Begin Again",
        "Should parse modlist name"
    );
    assert_eq!(
        modlist_info.mod_count, 50,
        "Should have 50 archives in sample"
    );
    assert!(
        !modlist_info.used_mod_keys.is_empty(),
        "Should have ModID keys"
    );
    assert!(
        !modlist_info.used_mod_file_ids.is_empty(),
        "Should have ModID+FileID pairs"
    );

    // Verify some known mods from the sample (first archive is Mod Organizer 2)
    assert!(
        modlist_info.used_mod_keys.contains("874"),
        "Should contain Mod Organizer 2 (ModID 874)"
    );
    assert!(
        modlist_info.used_mod_file_ids.contains("874-3835"),
        "Should contain exact MO2 version"
    );
}

#[test]
fn test_realistic_orphan_detection_with_sample() {
    // Use sample fixture to test realistic orphan detection
    let sample_path = std::path::Path::new("tests/fixtures/sample_modlist.json");

    if !sample_path.exists() {
        println!("Skipping realistic orphan test - fixture not present");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    fs::create_dir(&downloads_dir).unwrap();
    fs::create_dir(&wabbajack_dir).unwrap();

    // Create wabbajack file from sample
    let json_content = fs::read_to_string(sample_path).unwrap();
    let wabbajack_path = wabbajack_dir.join("sample.wabbajack");
    {
        let file = File::create(&wabbajack_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("modlist", options).unwrap();
        zip.write_all(json_content.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    // Create mod files matching real naming patterns from the sample:
    // Exact file name from sample: "Mod Organizer 2.5.2-ML1.5 Archive-874-2-5-2-ML1-5-1723841178.7z"
    // Create a USED file (exact file name match)
    create_simple_mod_file(
        &downloads_dir,
        "Mod Organizer 2.5.2-ML1.5 Archive-874-2-5-2-ML1-5-1723841178.7z",
        1000,
    );

    // Create an ORPHANED file (not in sample at all)
    create_simple_mod_file(
        &downloads_dir,
        "SomeRandomMod-99999-88888-1-0-1600000000.7z",
        500,
    );

    // Parse and detect
    let modlist_info = parse_wabbajack_file(&wabbajack_path).unwrap();
    let all_files = get_all_mod_files(&[downloads_dir]).unwrap();
    let scan_result = detect_orphaned_mods(&all_files, &[modlist_info]);

    assert_eq!(all_files.len(), 2, "Should find 2 files");
    assert_eq!(scan_result.used_mods.len(), 1, "Should have 1 used mod");
    assert_eq!(
        scan_result.orphaned_mods.len(),
        1,
        "Should have 1 orphaned mod"
    );
    assert_eq!(
        scan_result.orphaned_mods[0].file.mod_id, "99999",
        "Orphaned mod should be the unknown one"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();

    let files = get_all_mod_files(&[empty_dir]).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_non_mod_files_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&downloads_dir).unwrap();

    // Create various non-mod files
    File::create(downloads_dir.join("readme.txt")).unwrap();
    File::create(downloads_dir.join("notes.md")).unwrap();
    File::create(downloads_dir.join("image.png")).unwrap();
    
    // Create a valid archive without dashes (should now be detected as generic mod)
    File::create(downloads_dir.join("nodash.7z")).unwrap();

    // Create one valid standard mod file
    create_simple_mod_file(&downloads_dir, "ValidMod-1000-2000-1-0-1234567890.7z", 100);

    let files = get_all_mod_files(&[downloads_dir]).unwrap();

    assert_eq!(files.len(), 2, "Should detect both standard and generic archive files");
    assert!(files.iter().any(|f| f.file_name == "ValidMod-1000-2000-1-0-1234567890.7z"));
    assert!(files.iter().any(|f| f.file_name == "nodash.7z"));
}

#[test]
fn test_size_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    fs::create_dir(&downloads_dir).unwrap();
    fs::create_dir(&wabbajack_dir).unwrap();

    let wabbajack_file = wabbajack_dir.join("Test.wabbajack");
    create_dummy_wabbajack(
        &wabbajack_file,
        &[TestArchive::new("UsedMod", 1000, 1001, "1.0", "1600000000")],
    );

    // Create files with different sizes
    create_mod_file(
        &downloads_dir,
        "UsedMod",
        1000,
        1001,
        "1.0",
        "1600000000",
        10000,
    );
    create_mod_file(
        &downloads_dir,
        "OrphanedMod",
        9000,
        9001,
        "1.0",
        "1600000000",
        5000,
    );

    let modlist_info = parse_wabbajack_file(&wabbajack_file).unwrap();
    let all_files = get_all_mod_files(&[downloads_dir]).unwrap();
    let scan_result = detect_orphaned_mods(&all_files, &[modlist_info]);

    assert_eq!(
        scan_result.used_size, 10000,
        "Used size should be 10000 bytes"
    );
    assert_eq!(
        scan_result.orphaned_size, 5000,
        "Orphaned size should be 5000 bytes"
    );
}

#[test]
fn test_meta_file_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let backup_dir = temp_dir.path().join("backup");
    fs::create_dir(&downloads_dir).unwrap();

    let mod_filename = "TestMod-1000-2000-1-0-1234567890.7z";
    let meta_filename = format!("{}.meta", mod_filename);

    // Create mod file and its .meta file
    create_simple_mod_file(&downloads_dir, mod_filename, 1000);
    File::create(downloads_dir.join(&meta_filename))
        .unwrap()
        .write_all(b"meta content")
        .unwrap();

    let files = get_all_mod_files(&[downloads_dir.clone()]).unwrap();
    let orphaned = OrphanedMod {
        file: files[0].clone(),
    };

    // Delete with backup
    delete_orphaned_mods(&[orphaned], Some(&backup_dir), None);

    // Both files should be moved
    assert!(!downloads_dir.join(mod_filename).exists());
    assert!(!downloads_dir.join(&meta_filename).exists());
    assert!(backup_dir.join(mod_filename).exists());
    assert!(backup_dir.join(&meta_filename).exists());
}

#[test]
fn test_parse_real_files() {
    use wabbajack_library_cleaner::core::{parse_mod_filename, is_wabbajack_file};
    
    let files = [
        ("1) Point That Somewhere Else - Main File-73938-2-22-2-1766239208.zip", true, Some("73938")),
        ("BHYSYS-71112-13-02-1766329383.rar", true, Some("71112")),
        ("Pip-Boy UI Tweaks-85343-5-0-1-1766262984.zip", true, Some("85343")),
        ("testttt.zip", false, None),
    ];
    
    for (filename, should_be_wabbajack, expected_mod_id) in files {
        let is_wb = is_wabbajack_file(filename);
        println!("File: {}", filename);
        println!("  is_wabbajack_file: {} (expected: {})", is_wb, should_be_wabbajack);
        
        if let Some(parsed) = parse_mod_filename(filename) {
            println!("  mod_id: {} (expected: {:?})", parsed.mod_id, expected_mod_id);
            if let Some(exp) = expected_mod_id {
                assert_eq!(parsed.mod_id, exp, "ModID mismatch for {}", filename);
            }
        } else {
            println!("  PARSE FAILED");
            if expected_mod_id.is_some() {
                panic!("Expected to parse {} but failed", filename);
            }
        }
        println!();
    }
}

#[test]
fn test_parse_file_id() {
    use wabbajack_library_cleaner::core::parse_mod_filename;
    
    let files = [
        ("1) Point That Somewhere Else - Main File-73938-2-22-2-1766239208.zip", "73938", None), // No file_id in Nexus format
        ("SkyUI_5_2SE-12604-5-2SE-52344-1615410779.7z", "12604", Some("52344")), // Test format with file_id
    ];
    
    for (filename, expected_mod_id, expected_file_id) in files {
        if let Some(parsed) = parse_mod_filename(filename) {
            println!("File: {}", filename);
            println!("  mod_id: {} (expected: {})", parsed.mod_id, expected_mod_id);
                    println!("  file_id: {:?} (expected: {:?})", parsed.file_id, expected_file_id);
        }
    }
}

#[test]
fn test_real_modlist_parsing() {
    // Test with the REAL 14MB modlist file provided by the user
    let real_path = std::path::Path::new("tests/fixtures/real_modlist.json");

    if !real_path.exists() {
        println!("Skipping real modlist test - fixture not present");
        return;
    }

    println!("Testing with real modlist file: {:?}", real_path);
    let json_content = fs::read_to_string(real_path).expect("Failed to read real modlist");

    // Create a temporary wabbajack file
    let temp_dir = TempDir::new().unwrap();
    let wabbajack_path = temp_dir.path().join("real.wabbajack");

    {
        let file = File::create(&wabbajack_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("modlist", options).unwrap();
        zip.write_all(json_content.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    // Measure time to parse
    let start = std::time::Instant::now();
    let result = parse_wabbajack_file(&wabbajack_path);
    let duration = start.elapsed();

    assert!(
        result.is_ok(),
        "Should parse real wabbajack file successfully"
    );

    let modlist_info = result.unwrap();

    println!("Parsed real modlist in {:?}", duration);
    println!("  Name: {}", modlist_info.name);
    println!("  Archives: {}", modlist_info.mod_count);
    println!("  Unique ModIDs: {}", modlist_info.used_mod_keys.len());
    println!("  Unique FileIDs: {}", modlist_info.used_mod_file_ids.len());

    // Basic sanity checks for a real large modlist
    assert!(!modlist_info.name.is_empty(), "Modlist should have a name");
    assert!(modlist_info.mod_count >= 50, "Real modlist should have at least 50 mods");
    assert!(!modlist_info.used_mod_keys.is_empty(), "Should have detected ModIDs");
}

#[test]
fn test_simulation_with_real_modlist() {
    // This test simulates the user's manual "test mods" setup using the REAL modlist data.
    // It creates:
    // 1. Files that SHOULD exist (Used) - taken directly from modlist
    // 2. Files that are OLD versions (Duplicates) - same ModID, older timestamp
    // 3. Files that are ORPHANED (Unused) - random ModIDs not in list
    
    let real_path = std::path::Path::new("tests/fixtures/real_modlist.json");
    if !real_path.exists() {
        println!("Skipping simulation test - real_modlist.json not found");
        return;
    }

    // 1. Load the real modlist
    let json_content = fs::read_to_string(real_path).expect("Failed to read real modlist");
    let temp_dir = TempDir::new().unwrap();
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    let downloads_dir = temp_dir.path().join("downloads");
    fs::create_dir(&wabbajack_dir).unwrap();
    fs::create_dir(&downloads_dir).unwrap();

    let wabbajack_path = wabbajack_dir.join("Simulation.wabbajack");
    {
        let file = File::create(&wabbajack_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("modlist", options).unwrap();
        zip.write_all(json_content.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let modlist_info = parse_wabbajack_file(&wabbajack_path).expect("Failed to parse modlist");
    
    // 2. Generate Files
    println!("Generating test files based on '{}'...", modlist_info.name);

    // Extract some real filenames from the modlist to "install"
    // Now we take ALL filenames, including non-standard ones like "Vanilla.UI.Plus..."
    let real_files: Vec<String> = modlist_info.used_file_names.iter()
        .take(5)
        .cloned()
        .collect();
    
    // A) Create 5 USED files (Keep)
    for filename in &real_files {
        create_simple_mod_file(&downloads_dir, filename, 1024);
        println!("  [CREATED USED] {}", filename);
    }

    // B) Create OLD VERSION files (Old)
    // We take some filenames but change the timestamp to be older
    let mut old_versions_created = 0;
    for filename in &real_files {
        // Find the last dash to isolate timestamp
        if let Some(last_dash_idx) = filename.rfind('-') {
            let (prefix, suffix) = filename.split_at(last_dash_idx);
            
            // Find extension
            if let Some(dot_idx) = suffix.find('.') {
                let timestamp_str = &suffix[1..dot_idx]; 
                let ext = &suffix[dot_idx..];
                
                if let Ok(ts) = timestamp_str.parse::<u64>() {
                    if ts > 100000 {
                        let old_ts = ts - 100000;
                        let old_filename = format!("{}-{}{}", prefix, old_ts, ext);
                        
                        create_simple_mod_file(&downloads_dir, &old_filename, 1024);
                        println!("  [CREATED OLD ] {}", old_filename);
                        old_versions_created += 1;
                    } else {
                        println!("  [SKIP OLD GEN] {} (timestamp too small)", filename);
                    }
                }
            }
        }
    }

    // C) Create 5 ORPHANED files (Delete)
    // These have valid naming structure but random IDs/Names not in the list
    for i in 0..5 {
        let filename = format!("Orphaned Mod {}-9999{}-11111-1-0-1600000000.7z", i, i);
        create_simple_mod_file(&downloads_dir, &filename, 1024);
        println!("  [CREATED ORPH] {}", filename);
    }

    // D) Create User's Specific Edge Cases
    create_simple_mod_file(&downloads_dir, "1) Point That Somewhere Else - Main File-73938-2-22-2-1766239208.zip", 1024);
    create_simple_mod_file(&downloads_dir, "BHYSYS-71112-13-02-1766329383.rar", 1024);

    // 3. Run Analysis
    let all_files = get_all_mod_files(&[downloads_dir.clone()]).unwrap();
    let orphan_result = detect_orphaned_mods(&all_files, &[modlist_info]);
    let old_ver_result = scan_folder_for_duplicates(&downloads_dir).unwrap();

    // 4. Verification
    let total_expected = 5 + old_versions_created + 5 + 2;
    assert_eq!(all_files.len(), total_expected, "Should have detected all created files");

    // Check Orphans
    let orphans_found = orphan_result.orphaned_mods.iter()
        .filter(|m| m.file.mod_name.contains("Orphaned Mod"))
        .count();
    assert_eq!(orphans_found, 5, "Should detect 5 artificial orphans");

    // Check Used
    let real_files_found = orphan_result.used_mods.len();
    assert!(real_files_found >= 5, "Should detect at least the 5 real files as used");

    // Check Old Versions
    let duplicate_groups = old_ver_result.duplicates.len();
    assert_eq!(duplicate_groups, old_versions_created, "Should detect correct number of duplicate groups");
    
    // Check that the "Newest" in each group is the one with the larger timestamp
    for group in &old_ver_result.duplicates {
        let newest_file = &group.files[group.newest_idx];
        let kept_ts: u64 = newest_file.timestamp.parse().unwrap_or(0);
        for file in &group.files {
            let ts: u64 = file.timestamp.parse().unwrap_or(0);
            assert!(kept_ts >= ts, "Should keep the newest file");
        }
    }

    println!("Simulation Test Passed!");
    println!("  Files Scanned: {}", all_files.len());
    println!("  Used Mods: {}", orphan_result.used_mods.len());
    println!("  Orphaned Mods: {}", orphan_result.orphaned_mods.len());
    println!("  Duplicate Groups: {}", old_ver_result.duplicates.len());
}
