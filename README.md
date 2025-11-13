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
- **GUI Interface**: User-friendly graphical interface (NEW!)
- **Flexible Paths**: Select any directory for wabbajack files and mod downloads
- **Recycle Bin Support**: Option to send files to Recycle Bin instead of permanent deletion (NEW!)
- **Logging**: All operations are logged
- **Multi-Game Support**: Works with all game folders (Skyrim, Fallout 4, etc.)
- **Statistics**: View your mod library statistics by game
- **CLI Mode**: Command-line interface still available with `--cli` flag

## 🛡️ Safety Features

### Old Version Cleanup
- Keeps the newest version of each mod
- Skips files that are in use (file lock detection)
- Asks for confirmation before deleting anything
- Detailed logging with timestamps
- Skips temporary/incomplete files (`.part`, `.tmp`)

### Orphaned Mods Cleanup
- Dry-run preview required before deletion
- Confirmation dialog before any deletion
- Only deletes mods not in any active modlist
- Shared mods (used by multiple modlists) are protected
- File lock detection prevents deletion of in-use files
- **Recycle Bin option**: Choose between permanent deletion or Recycle Bin (Windows)

## 📖 Usage

### Installation

1. Download `wabbajack-library-cleaner.exe` from the [Releases](../../releases) page
2. Double-click to run the program (GUI mode by default)
3. **No setup required!** - You can place the executable anywhere you want

### GUI Mode (Default)

Simply double-click `wabbajack-library-cleaner.exe` to launch the graphical interface.

**Step 1: Select Directories**
- Click "Select Wabbajack Directory" to choose where your `.wabbajack` modlist files are located
- Click "Select Downloads Directory" to choose where your mod archives are stored (folder containing Skyrim, Fallout4, etc.)

**Step 2: Choose Options**
- Check "Send deleted files to Recycle Bin" if you want to be able to restore files (recommended)
- Uncheck for permanent deletion (cannot be undone!)

**Step 3: Perform Actions**
- Use "Scan" buttons to preview what will be deleted (dry-run)
- Use "Clean" buttons to actually delete files
- View Statistics to see your library size

All output and progress will be shown in the output area at the bottom of the window.

### CLI Mode (Advanced)

Run with `--cli` flag for command-line interface:
```bash
wabbajack-library-cleaner.exe --cli
```

In CLI mode, the tool must be placed in your Wabbajack downloads directory as before.

### CLI Mode Menu Options

When running in CLI mode (`--cli` flag), you get these menu options:

1. **Scan folder (Dry-run)** - Preview old versions to be deleted
2. **Clean folder** - Delete old versions of mods
3. **Scan for orphaned mods (Dry-run)** - Preview mods not used by active modlists
4. **Clean orphaned mods** - Delete mods not used by active modlists
5. **View statistics** - Show library statistics by game
6. **Exit**

See CLI mode workflow details in [CHANGELOG](CHANGELOG.md).

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

**Current Version:** v2.0.0

**Major Update:** Now with GUI interface and Recycle Bin support!

See [CHANGELOG](CHANGELOG.md) for version history.
