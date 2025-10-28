# Wabbajack Library Cleaner

A safe and efficient tool to clean up duplicate mod versions in your Wabbajack library (downloads folder), freeing up disk space while keeping only the latest versions.

## 💡 The Problem

Managing Wabbajack modlists leads to bloated downloads folders filled with duplicate mod versions. Each modlist update pulls newer mod versions, leaving old ones wasting disk space. A 500GB downloads folder can often be reduced to 200-300GB.

This tool intelligently identifies and removes old versions while keeping only the latest.

**⚠️ Important:** Keeps ONLY the latest version. If using multiple modlists with different mod versions, older versions will be deleted (re-downloadable). Close MO2/Wabbajack before using. Always run dry-run first!

## 🎯 Features

- **Smart Detection**: Automatically identifies and groups mod versions by ModID and ModName
- **Safe by Design**: Always keeps the newest version, never deletes the latest files
- **Dry-Run Mode**: Preview what will be deleted before making any changes
- **Interactive Menu**: User-friendly interface with multiple scanning options
- **Comprehensive Logging**: Every operation is logged with timestamps
- **Size Filtering**: Option to delete only files larger than a specified size
- **Multi-Game Support**: Processes all game folders (Skyrim, Fallout 4, Fallout 3-NV, etc.)

## 🛡️ Safety Features

- Always keeps newest version
- File lock detection (skips files in use)
- Double confirmation before deletion
- Detailed logging with timestamps
- Skips temp files (`.part`, `.tmp`)

## 📥 Installation

### Option 1: Download Pre-compiled Binary
1. Download `wabbajack-library-cleaner.exe` from the [Releases](../../releases) page
2. Place it in your Wabbajack downloads directory (e.g., `F:\Wabbajack`)
3. Run the executable

### Option 2: Compile from Source
```bash
# Prerequisites: Go 1.25 or later
go build -ldflags="-s -w" -o wabbajack-library-cleaner.exe wabbajack-library-cleaner.go
```

## 🚀 Usage

### Quick Start

1. **Place the executable** in your Wabbajack downloads directory (same folder as Skyrim, Fallout 4, etc.)

2. **Double-click** `wabbajack-library-cleaner.exe` to run

3. **⚠️ IMPORTANT: Always run Dry-Run first!**
   - Select option `1` (Scan folder - Dry run)
   - Choose your game folder (e.g., Skyrim)
   - Review what will be deleted
   - Only proceed with deletion if satisfied

### Menu Options

The tool will detect all game folders in your directory and let you:
1. **Scan folder (Dry-run)** - Preview what will be deleted
2. **Delete old versions** - Clean up after reviewing
3. **Delete with size filter** - Only remove files larger than X MB
4. **Exit**

## 📋 Example Output

```
DUPLICATE REPORT: Skyrim

1. Alternate Perspective-50307
   KEEP:
     └─ Alternate Perspective-50307-4-0-3-1731841209.zip
        Version: 4-0-3, Date: 2024-11-17 14:00, Size: 4.64 MB

   DELETE (1 old version(s)):
     └─ Alternate Perspective-50307-3-1-0-1718214257.zip
        Version: 3-1-0, Date: 2024-06-12 20:44, Size: 4.57 MB

   [SPACE] Space to free: 4.57 MB
```

## 🗂️ Mod File Naming Convention

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

**File is locked:** Close Mod Organizer 2 / Wabbajack before running

**Skipped files:** Normal for `.meta` files and non-standard naming

**No duplicates found:** Your folder is already clean!

## 📜 License

MIT License - see [LICENSE](LICENSE) file for details

---

## 📈 Version

**Current Version:** v1.0.0

See [CHANGELOG](CHANGELOG.md) for version history.
