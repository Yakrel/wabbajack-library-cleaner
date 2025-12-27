use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use wabbajack_library_cleaner::core::{
    delete_orphaned_mods, detect_orphaned_mods, get_all_mod_files, parse_wabbajack_file,
    DeletionResult,
};
use zip::write::FileOptions;
use zip::ZipWriter;

// --- Helper: Create a dummy .wabbajack file ---
fn create_dummy_wabbajack(path: &Path, mod_ids: &[(i64, i64)]) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    zip.start_file("modlist", options).unwrap();

    // Create a minimal valid JSON structure based on what the parser expects
    let mut archives_json = String::from("[");
    for (i, (mod_id, file_id)) in mod_ids.iter().enumerate() {
        if i > 0 {
            archives_json.push(',');
        }
        archives_json.push_str(&format!(
            r#"{{
                "State": {{
                    "ModID": {},
                    "FileID": {}
                }}
            }}"#,
            mod_id, file_id
        ));
    }
    archives_json.push(']');

    let json = format!(
        r#"{{
            "Name": "Test Modlist",
            "Archives": {}
        }}"#,
        archives_json
    );

    zip.write_all(json.as_bytes()).unwrap();
    zip.finish().unwrap();
}

// --- Helper: Create a dummy mod file on disk ---
fn create_dummy_mod_file(dir: &Path, name: &str, mod_id: i64, file_id: i64, timestamp: &str) {
    // Format: Name-ModID-FileID-Version-Timestamp.7z
    // Example: "Mod A-100-200-1-0-1234567890.7z"
    let filename = format!("{}-{}-{}-1-0-{}.7z", name, mod_id, file_id, timestamp);
    let path = dir.join(filename);
    let mut file = File::create(path).unwrap();
    // Write a few bytes just so it's not empty (though 0-byte works too usually)
    writeln!(file, "dummy content").unwrap();
}

#[test]
fn test_end_to_end_orphan_detection() {
    // 1. Setup Environment
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let wabbajack_dir = temp_dir.path().join("wabbajack");
    std::fs::create_dir(&downloads_dir).unwrap();
    std::fs::create_dir(&wabbajack_dir).unwrap();

    // 2. Create a "Used" Mod in the Wabbajack list (ID: 100, FileID: 101)
    let wabbajack_file = wabbajack_dir.join("test.wabbajack");
    create_dummy_wabbajack(&wabbajack_file, &[(100, 101)]);

    // 3. Create Files on Disk
    // File A: USED (Matches ID 100, FileID 101)
    create_dummy_mod_file(&downloads_dir, "Used Mod", 100, 101, "1600000000");

    // File B: ORPHANED (ID 999 is not in the list)
    create_dummy_mod_file(&downloads_dir, "Orphan Mod", 999, 888, "1600000000");

    // File C: OLD VERSION (Same ID 100, but different FileID 102 - if strictly matching FileID)
    // Note: The parser logic creates a set of "used_mod_file_ids".
    // If exact FileID match is required, this is Orphaned.
    // If only ModID match is required, this is Used.
    // Based on code: `used_mod_file_ids.insert(format!("{}-{}", mod_id, file_id));`
    // It seems strict.
    create_dummy_mod_file(&downloads_dir, "Old Version", 100, 102, "1500000000");

    // 4. Run "Parse"
    let modlist_info = parse_wabbajack_file(&wabbajack_file).expect("Failed to parse dummy wabbajack");
    let selected_modlists = vec![modlist_info];

    // 5. Run "Scan Disk"
    // We pass a vector of directories to `get_all_mod_files`
    let game_folders = vec![downloads_dir.clone()];
    let all_files = get_all_mod_files(&game_folders).expect("Failed to scan disk");

    assert_eq!(all_files.len(), 3, "Should have found 3 files on disk");

    // 6. Run "Detect Orphans"
    let scan_result = detect_orphaned_mods(&all_files, &selected_modlists);

    // 7. Assertions
    // "Used Mod" (100-101) should be in used_mods
    assert!(
        scan_result.used_mods.iter().any(|f| f.mod_id == "100" && f.file_id == Some("101".to_string())),
        "Should detect the used mod"
    );

    // "Orphan Mod" (999-888) should be orphaned
    assert!(
        scan_result.orphaned_mods.iter().any(|f| f.file.mod_id == "999"),
        "Should detect the orphan mod"
    );

    // "Old Version" (100-102) is NOT in the list (list only has 100-101).
    // So it should be considered orphaned/unused by *this specific modlist*.
    assert!(
        scan_result.orphaned_mods.iter().any(|f| f.file.mod_id == "100" && f.file.file_id == Some("102".to_string())),
        "Old version (different FileID) should be orphaned if not in modlist"
    );
}

#[test]
fn test_deletion_safety() {
    let temp_dir = TempDir::new().unwrap();
    let downloads_dir = temp_dir.path().join("downloads");
    let backup_dir = temp_dir.path().join("backup");
    std::fs::create_dir(&downloads_dir).unwrap();

    // Create a file to be deleted
    create_dummy_mod_file(&downloads_dir, "ToDelete", 500, 500, "1234567890");
    let files = get_all_mod_files(&vec![downloads_dir.clone()]).unwrap();
    let file_to_delete = &files[0];

    // Mock an "OrphanedMod" wrapper
    let orphaned = wabbajack_library_cleaner::core::types::OrphanedMod {
        file: file_to_delete.clone(),
        reason: "Test".to_string(),
    };

    // Perform Deletion (Move to Backup)
    let result = delete_orphaned_mods(&[orphaned], Some(&backup_dir), None);

    assert_eq!(result.deleted_count, 1);
    assert!(!file_to_delete.full_path.exists(), "Original should be gone");
    
    // Check backup
    let backup_file = backup_dir.join(&file_to_delete.file_name);
    assert!(backup_file.exists(), "Backup should exist");
}
