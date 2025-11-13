# Wabbajack Library Cleaner v2.0

**A Windows-only GUI tool to clean orphaned mods and old versions from your Wabbajack downloads folder.**

## 💡 What Does It Do?

### PRIMARY: Orphaned Mods Cleanup (50-150 GB savings)

**The Problem:** You tried 5 modlists, kept 2, deleted 3. But those deleted modlists' mods are **still in your downloads folder wasting space!**

**The Solution:** This tool compares your mod files against your **active modlists** (using their `.wabbajack` files) and removes mods that aren't needed anymore.

**⚠️ You Need:** The `.wabbajack` files for ALL modlists you're currently using. Without them, the tool can't tell which mods to keep.

### SECONDARY: Old Version Cleanup (10-30 GB savings)

**The Problem:** Modlists update, Wabbajack downloads new versions, but old versions accumulate.

**The Solution:** Keeps the newest version of each mod, removes old duplicates.

**⚠️ Important:** Some modlists require specific OLD versions for compatibility. The tool's old version cleanup is based on **file timestamps only**, not the actual version your modlist needs. Use with caution!

## 📸 Screenshots

Coming soon! (GUI screenshots will be added after build)

## ✨ Key Features

- 🎯 **Orphaned Mods Cleanup** - Remove mods from deleted/inactive modlists (50-150 GB typical)
- 📂 **Auto-Scan** - Automatically finds and displays all `.wabbajack` files
- ✅ **Checkbox Selection** - Pick which modlists you're actively using
- 🧠 **Smart Parsing** - Reads `.wabbajack` files to determine which mods are needed
- 🛡️ **Shared Protection** - Mods used by multiple modlists are never deleted
- ⚠️ **Old Version Cleanup** - Remove duplicate mod versions (use with caution!)
- 🗑️ **Recycle Bin** - Deleted files can be restored (recommended!)
- 🖼️ **Modern GUI** - Easy-to-use Windows interface
- 📊 **Statistics** - View library size by game
- 📝 **Complete Logging** - All operations tracked with timestamps

## 🛡️ Safety Protections

- 🗑️ **Recycle Bin by default** - Files can be restored
- 🔍 **Preview mode** - See what will be deleted before confirming
- ✋ **Confirmation dialogs** - Multiple checks before deletion
- 🔒 **File lock detection** - Skips files in use
- 🛡️ **Shared mod protection** - Never deletes mods used by checked modlists
- 📝 **Complete logging** - All operations recorded
- 🚫 **Smart filtering** - Skips temp files (`.part`, `.tmp`)

## 📖 How to Use

### Installation

1. Download `wabbajack-library-cleaner.exe` from the [Releases](../../releases) page
2. Double-click to launch - **No installation needed!**

### Quick Start Guide

**Step 1: Select Modlist Folder**
- Click **"📁 Select Modlist Folder"**
- Navigate to the folder containing your `.wabbajack` files
  - Common location: `C:\Users\YourName\AppData\Local\Wabbajack\downloaded_mod_lists\`
  - Alternative: Wherever you keep your `.wabbajack` files
- Tool auto-scans and displays all found modlists with checkboxes
- **✅ Check ONLY the modlists you're actively using**

**Step 2: Select Downloads Folder**
- Click **"📁 Select Downloads Folder"**
- Choose your Wabbajack downloads directory
  - Example: `F:\Wabbajack\downloads` (contains game-specific folders like Skyrim, Fallout4, etc.)

**Step 3: Configure Safety Options**
- ✅ **"Send deleted files to Recycle Bin"** ← Recommended! (you can restore files if needed)
- ❌ Uncheck for permanent deletion (⚠️ cannot be undone!)

**Step 4: Clean Your Library**

**🎯 PRIMARY: Clean Orphaned Mods** (Major space savings!)
1. **"🔍 Scan for Orphaned Mods"** → Preview what will be deleted
2. Review output: shows which mods are kept vs removed
3. **"🧹 Clean Orphaned Mods"** → Delete after confirmation

**⚠️ SECONDARY: Clean Old Versions** (Use with caution!)
- ⚠️ **Warning:** Your modlist may require old versions! This feature only looks at timestamps, not what your modlist actually needs.
- Only use if you understand the risks
1. Select game folder to scan
2. Preview old versions detected
3. Clean only if you're certain they're safe to remove

**📊 View Statistics**
- See your mod library size breakdown by game

### ⚠️ CRITICAL Requirements

**For Orphaned Mods Cleanup:**
- You **MUST** have the `.wabbajack` files for ALL modlists you're currently using
- If a modlist's `.wabbajack` file is missing, the tool **will mark that modlist's mods as orphaned!**
- After selecting modlists, **verify all YOUR ACTIVE modlists are checked**

**For Old Versions Cleanup:**
- Modlist version: This tool doesn't know which version your modlist actually uses
- It only looks at file timestamps to find "newer" files
- **Risk:** May delete an old version that your modlist specifically requires
- **Recommendation:** Use orphaned mods cleanup instead - it's much safer!

### Compile from Source (Optional)

**Requirements:**
- Go 1.21 or later ([Download](https://go.dev/dl/))
- MinGW-w64 GCC compiler (for GUI support)
  - Install via: `choco install mingw` or `winget install -e --id jmeubank.tdm-gcc`
  - Or download from: [TDM-GCC](https://jmeubank.github.io/tdm-gcc/download/)

**Build Command:**
```bash
# Set CGO_ENABLED for GUI compilation
set CGO_ENABLED=1

# Build with icon and no console window
go build -ldflags="-s -w -H=windowsgui" -o wabbajack-library-cleaner.exe
```

**Note:** The icon is automatically embedded via `rsrc_windows_amd64.syso` (already included in repo).

## 📋 How It Works

The tool parses Nexus Mods/Wabbajack filename format:
```
ModName-ModID-Version-Timestamp.extension
```

Example: `Alternate Perspective-50307-4-0-3-1731841209.zip`
- Mod ID: `50307` - Used to match against `.wabbajack` file's mod list
- Timestamp: `1731841209` - Used for old version detection

**For orphaned cleanup:** Matches ModID against your selected modlists' `.wabbajack` files  
**For old version cleanup:** Groups by ModID, keeps newest timestamp

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

- **Supported formats:** `.7z`, `.zip`, `.rar`, `.tar`, `.gz`, `.exe`
- **Performance:** Fast scanning, low memory usage
- **Deployment:** Single executable, no dependencies required

## 🐛 Common Issues

### Orphaned Mods Cleanup

**❌ "No .wabbajack files found"**
- Make sure you're pointing to the correct folder containing `.wabbajack` files
- Check: `C:\Users\YourName\AppData\Local\Wabbajack\downloaded_mod_lists\`

**❌ "Too many mods marked as orphaned"**
- Did you check ALL the modlists you're using?
- **Missing .wabbajack file?** The tool can't know what to keep without it!
- Solution: Re-download the modlist to get its `.wabbajack` file, then run the tool again

**❌ "Failed to parse .wabbajack file"**
- File may be corrupted - try re-downloading the modlist

**✅ "Shared mods being kept"**
- This is correct! If ANY checked modlist uses a mod, it's kept safe

### Old Version Cleanup

**❌ "File is locked"**
- Close Mod Organizer 2, Wabbajack, and any mod management tools

**⚠️ "I deleted a version my modlist needs!"**
- Old version cleanup is timestamp-based, not modlist-version-aware
- The tool doesn't know which specific version your modlist requires
- If this happens: Re-run Wabbajack to re-download the correct version

**✅ "Many files skipped"**
- Normal! Tool skips `.meta` files and non-standard filenames automatically

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
