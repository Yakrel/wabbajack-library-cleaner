# Wabbajack Library Cleaner

A tool to clean up old mod versions from your Wabbajack downloads folder, freeing up disk space.

## 💡 The Problem

When a modlist updates, Wabbajack downloads new mod versions but keeps the old ones. Over time, you end up with multiple versions of the same mods eating disk space.

Many users keep their downloads folder because they don't have Nexus Premium - re-downloading everything would be slow and painful. But this leads to bloated folders with duplicate versions.

This tool scans your downloads folder, identifies duplicate mod versions, and removes the old ones while keeping the newest.

**⚠️ Important Note:** 
- Not all modlists use the newest mod versions. There's a small chance you might delete a version that's actually needed.
- Always run the dry-run preview (option 1) first to see what will be deleted.
- If a needed version gets deleted, you can re-download it from Nexus.
- Close MO2/Wabbajack before running this tool.

## 📸 Screenshots

### Preview Mode (Dry-Run)
![Preview Mode](screenshots/preview.jpg)
*Shows what will be deleted before making any changes*

### Cleaning in Action
![Cleaning](screenshots/cleaning.jpg)
*Removing old versions and freeing up disk space*

## 💡 The Orphaned Mods Problem

**Scenario:** You tried 5 different modlists. You liked 2 of them and deleted the other 3. But the mods from those deleted modlists are still sitting in your downloads folder, wasting 50-150 GB of space!

**Solution:** The new orphaned mods feature identifies and removes mods that aren't used by any of your active modlists.

## 🎯 Features

### Old Version Cleanup
- **Smart Detection**: Groups mod versions by ModID and name
- **Safe**: Keeps the newest version of each mod
- **Preview First**: Dry-run mode shows what will be deleted before doing anything
- **Interactive**: Simple menu to choose which folders to scan

### Orphaned Mods Cleanup (NEW!)
- **Modlist-Based Detection**: Identifies mods not used by any active modlist
- **Massive Space Savings**: Remove mods from deleted/inactive modlists (50-150 GB typical)
- **Smart Analysis**: Parses .wabbajack files to determine which mods you actually need
- **Safe Selection**: Choose which modlists you're actively using
- **Detailed Reports**: See exactly which mods are orphaned and why

### General
- **Logging**: All operations are logged
- **Multi-Game Support**: Works with all game folders (Skyrim, Fallout 4, etc.)
- **Statistics**: View your mod library statistics by game

## 🛡️ Safety Features

### Old Version Cleanup
- Keeps the newest version of each mod
- Skips files that are in use (file lock detection)
- Asks for confirmation before deleting anything
- Detailed logging with timestamps
- Skips temporary/incomplete files (`.part`, `.tmp`)

### Orphaned Mods Cleanup
- Dry-run preview required before deletion
- Double confirmation with "DELETE" typed in uppercase
- Only deletes mods not in any active modlist
- Shared mods (used by multiple modlists) are protected
- File lock detection prevents deletion of in-use files

## 📖 Usage

### Installation

1. Download `wabbajack-library-cleaner.exe` from the [Releases](../../releases) page
2. Place it in your Wabbajack downloads directory (the folder that contains your game folders like Skyrim, Fallout4, etc.)
3. **For orphaned mods feature:** Place your `.wabbajack` modlist files in the same directory
4. Run the executable

### Directory Structure

```
F:\Wabbajack\                          <-- Place the tool here
├─ wabbajack-library-cleaner.exe
├─ Uranium Fever.wabbajack             <-- Your modlist files
├─ FAnomaly.wabbajack
├─ Skyrim\                             <-- Mod archives for Skyrim
│  ├─ SkyUI-3863-5-2-1234567890.7z
│  ├─ SkyUI-3863-5-1-1111111111.7z     <-- Old version (will be detected)
│  └─ ...
├─ Fallout4\                           <-- Mod archives for Fallout 4
└─ [other game folders]\
```

### Menu Options

1. **Scan folder (Dry-run)** - Preview old versions to be deleted
2. **Clean folder** - Delete old versions of mods
3. **Scan for orphaned mods (Dry-run)** - Preview mods not used by active modlists
4. **Clean orphaned mods** - Delete mods not used by active modlists
5. **View statistics** - Show library statistics by game
6. **Exit**

### Workflow

#### Cleaning Old Versions
1. Select option `1` to preview what will be deleted
2. Review the report
3. Select option `2` to actually delete old versions

#### Cleaning Orphaned Mods
1. Ensure your `.wabbajack` files are in the base directory
2. Select option `3` to scan for orphaned mods
3. Choose which modlists you're actively using (e.g., `1,2` or `all`)
4. Review the detailed report showing:
   - Used mods (safe, needed by your active modlists)
   - Orphaned mods (not used by any active modlist)
5. Select option `4` to delete orphaned mods
6. Type `DELETE` (in uppercase) to confirm

**Example Session:**
```
=== MODLIST-BASED CLEANUP ===

[FOUND] Detected 5 modlist file(s):
  1. Uranium Fever.wabbajack ... [OK] (1,245 mods)
  2. FAnomaly.wabbajack ... [OK] (892 mods)
  3. Badlands.wabbajack ... [OK] (1,103 mods)
  4. ModdingLinked.wabbajack ... [OK] (756 mods)
  5. Wildlander.wabbajack ... [OK] (2,234 mods)

[SELECT] Which modlists are you CURRENTLY USING?
Enter numbers separated by commas (e.g., 1,2,3) or 'all': 1,2

[SELECTED] Active modlists:
  ✓ Uranium Fever
  ✓ FAnomaly

[SCANNING] Collecting mod files from game folders...
[OK] Found 2,159 mod files

[ANALYZING] Detecting orphaned mods...

=== RESULTS ===

✓ USED MODS: 1,847 mods (45.2 GB)
  These mods are used by your active modlist(s):
    • Uranium Fever
    • FAnomaly

✗ ORPHANED MODS: 312 mods (23.8 GB)
  These mods are NOT used by any of your active modlists.
  They may be from deleted or inactive modlists.

  Examples:
    • Some Mod-12345-1-0-1234567890.7z (156.3 MB)
    • Another Mod-67890-2-1-9876543210.7z (89.2 MB)
    ... and 310 more
```

### Compile from Source (Optional)
```bash
# Prerequisites: Go 1.25 or later
go build -ldflags="-s -w" -o wabbajack-library-cleaner.exe
```

## ️ Mod File Naming Convention

The tool recognizes Wabbajack/Nexus Mods file naming:

```
ModName-ModID-Version-Timestamp.extension
```

Example:
```
Alternate Perspective-50307-4-0-3-1731841209.zip
├─ Mod Name: Alternate Perspective
├─ Mod ID: 50307
├─ Version: 4-0-3
└─ Timestamp: 1731841209 (Unix timestamp)
```

## 📝 Logging

All operations are logged to timestamped log files:
```
wabbajack-library-cleaner_YYYY-MM-DD_HH-MM-SS.log
```

Example log entries:
```
2025/10/28 10:56:59 [INFO] Scanning folder: F:\Wabbajack\Skyrim
2025/10/28 10:56:59 [INFO] Found 498 mod groups with duplicates
2025/10/28 10:56:59 [INFO] Skipped 9959 files (non-standard naming or meta files)
```


## 🔧 Technical Details

### Supported Archive Formats
- `.7z`
- `.zip`
- `.rar`
- `.tar`
- `.gz`
- `.exe`

### Performance
- Fast scanning
- Low memory usage
- Single executable, no dependencies

## 🐛 Troubleshooting

### Old Version Cleanup

**"File is locked" error:** Close Mod Organizer 2 and Wabbajack before running

**Many files skipped:** This is normal - the tool automatically skips `.meta` files and files with non-standard naming patterns. When a mod archive is deleted, its `.meta` file is also automatically deleted.

**No duplicates found:** Your folder is already clean!

**Accidentally deleted a needed version:** You can re-download it from Nexus Mods

### Orphaned Mods Cleanup

**"No .wabbajack files found":** Place your `.wabbajack` modlist files in the same directory as the tool (the base Wabbajack directory, not inside game folders)

**"Failed to parse wabbajack file":** The .wabbajack file may be corrupted. Try re-downloading the modlist.

**Too many mods marked as orphaned:** Make sure you selected ALL the modlists you're currently using. If you're using a modlist but don't have its .wabbajack file, the tool will mark its mods as orphaned.

**Shared mods being kept:** This is correct! If a mod is used by any of your active modlists, it will be kept even if other (inactive) modlists also used it.

## 📜 License

MIT License - see [LICENSE](LICENSE) file for details

## 📊 Expected Space Savings

Based on user reports:
- **Old version cleanup:** 10-30 GB typical
- **Orphaned mods cleanup:** 50-150 GB typical
- **Combined:** 100+ GB possible!

The orphaned mods feature is particularly effective if you've tried multiple modlists and deleted some of them.

---

**Current Version:** v1.0.2 (Development)

See [CHANGELOG](CHANGELOG.md) for version history.
