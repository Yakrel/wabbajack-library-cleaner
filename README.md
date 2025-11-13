# Wabbajack Library Cleaner v2.0

**A Windows-only GUI tool to clean orphaned mods and old versions from your Wabbajack downloads folder.**

## 💡 The PRIMARY Problem: Orphaned Mods

**The Big Issue:** You tried 5 different modlists. You kept 2 and deleted 3. But the mods from those deleted modlists are **still sitting in your downloads folder, wasting 50-150 GB of space!**

**The Solution:** This tool identifies and removes mods that aren't used by your active modlists.

## 💡 The Secondary Problem: Old Versions

When a modlist updates, Wabbajack downloads new mod versions but keeps the old ones. Over time, you accumulate duplicate versions eating disk space.

**⚠️ Important:** Not all modlists use the newest mod versions. Some require old versions for compatibility. This tool warns you before cleaning old versions.

## 📸 Screenshots

Coming soon! (GUI screenshots will be added after build)

## 🎯 Features

### PRIMARY: Orphaned Mods Cleanup
- **🎯 Major Space Savings**: Remove mods from deleted/inactive modlists (50-150 GB typical)
- **Auto-Detection**: Select modlist folder → Auto-scans and shows all .wabbajack files
- **Checkbox Selection**: Check which modlists you're actively using
- **Smart Analysis**: Parses .wabbajack files to determine which mods you actually need
- **Detailed Reports**: See exactly which mods are orphaned and why
- **Shared Protection**: Mods used by multiple modlists are always kept safe

### SECONDARY: Old Version Cleanup
- **Smart Detection**: Groups mod versions by ModID and name
- **⚠️ Safety Warning**: Tool warns that modlists may require old versions
- **Preview First**: Dry-run mode shows what will be deleted
- **Safe**: Keeps the newest version of each mod

### General Features
- **🖼️ Modern GUI**: User-friendly Windows-only graphical interface
- **📁 Flexible Paths**: Select any directory for modlist files and mod downloads
- **🗑️ Recycle Bin**: Send deleted files to Recycle Bin (can be restored)
- **📊 Statistics**: View your mod library size by game
- **📝 Logging**: All operations are logged
- **🛡️ Multiple Safety Checks**: File lock detection, confirmations, dry-run previews

## 🛡️ Safety Features

- **🗑️ Recycle Bin Default**: Files go to Recycle Bin (can be restored) unless you disable it
- **🔍 Preview First**: All actions have dry-run preview mode
- **✋ Confirmation Dialogs**: Multiple confirmations before any deletion
- **🔒 File Lock Detection**: Skips files in use by other programs
- **🛡️ Shared Mod Protection**: Never deletes mods used by multiple modlists
- **⚠️ Old Version Warning**: Warns that some modlists may require old versions
- **📝 Detailed Logging**: All operations logged with timestamps
- **🚫 Smart Filtering**: Skips temporary files (`.part`, `.tmp`)

## 📖 Usage

### Installation

1. Download `wabbajack-library-cleaner.exe` from the [Releases](../../releases) page
2. Double-click to run the program (GUI mode by default)
3. **No setup required!** - You can place the executable anywhere you want

### Using the GUI

Simply double-click `wabbajack-library-cleaner.exe` to launch the program.

#### Step 1: Select Modlist Folder
1. Click **"📁 Select Modlist Folder"**
2. Choose the folder containing your `.wabbajack` files
   - Example: `F:\Wabbajack\downloads_modlist`
3. The tool automatically scans and shows all modlists
4. **Check the modlists you're currently using** (all are selected by default)
   - Use "Select All" / "Deselect All" buttons for convenience

#### Step 2: Select Downloads Folder
1. Click **"📁 Select Downloads Folder"**
2. Choose the folder containing your mod archives
   - Example: `F:\Wabbajack\downloads` (contains Skyrim, Fallout4, etc. folders)

#### Step 3: Choose Deletion Options
- ✅ **"Send deleted files to Recycle Bin"** (recommended - can be restored)
- ❌ Uncheck for permanent deletion (cannot be undone!)

#### Step 4: Cleanup Actions

**PRIMARY: Orphaned Mods Cleanup** (50-150 GB typical savings)
1. Click **"🔍 Scan for Orphaned Mods"** to preview
2. Review the output showing used vs orphaned mods
3. Click **"🧹 Clean Orphaned Mods"** to delete
4. Confirm the action

**SECONDARY: Old Versions Cleanup**
⚠️ **Warning**: Some modlists may require old versions! Check carefully.
1. Select a game folder to scan
2. Preview old versions found
3. Clean if safe to do so

**Statistics**
- Click **"📊 View Statistics"** to see library size by game

All output appears in the scrollable text area at the bottom.

### Compile from Source (Optional)

**Requirements:** Go 1.25 or later ([Download](https://go.dev/dl/))

```bash
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
