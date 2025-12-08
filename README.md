# Wabbajack Library Cleaner

Windows GUI tool to clean orphaned mods and old versions from your Wabbajack downloads folder.

## What It Does

### PRIMARY: Orphaned Mods Cleanup

Removes mods from deleted/inactive modlists. Compares your files against active modlists using `.wabbajack` files.

**Requires:** `.wabbajack` files for ALL active modlists. Without them, tool can't tell which mods to keep.

### SECONDARY: Old Version Cleanup

Keeps newest version of each mod, removes old duplicates.

**Warning:** Use with caution. Complex detection system with multiple safety checks, but may still remove versions your modlist needs.

## Screenshots

![Main Interface](screenshots/main-interface.png)
![Orphaned Mods Scan](screenshots/orphaned-scan.png)
![Old Versions Scan](screenshots/old-versions-scan.png)

## Technical Architecture

This tool is built with a focus on **Reverse Engineering** and **System Safety**. It parses proprietary file formats and employs heuristic algorithms to ensure data integrity.

### 1. Reverse Engineering & Parsing
- **Wabbajack Archive Analysis:** The tool reverse-engineers the `.wabbajack` file format (which is a custom ZIP structure containing a JSON manifest).
- **Dependency Mapping:** It parses the internal `modlist` JSON to extract `ModID` and `FileID` pairs, creating a precise dependency map of your active modlists.
- **Intelligent Matching:** 
  - **Precise Match:** Checks `ModID + FileID` for exact version matching.
  - **Fuzzy Match:** Fallback to `ModID` only to prevent accidental deletion of valid variants.

### 2. Heuristic Analysis
- **Version Normalization:** Uses custom string parsing logic (no heavy regex) to normalize version numbers (e.g., treating `Interface 1.3.6` and `Interface 1.4.0` as the same mod group).
- **Patch Detection:** Implements a heuristic size analysis algorithm (size ratio < 0.1) to intelligently distinguish between full mod updates and small hotfixes/patches, preventing the deletion of required base files.
- **Context Awareness:** Detects content descriptors (e.g., "1K" vs "4K", "CBBE" vs "UNP") to avoid flagging different mod variants as duplicates.

### 3. System Safety & Concurrency
- **Atomic Operations:** File deletions are simulated first (Dry-Run). Actual deletions can be performed as "Move" operations to a timestamped backup folder, ensuring zero data loss.
- **File Locking:** Direct interaction with Windows syscalls to detect if a file is locked by another process (e.g., Mod Organizer 2), preventing corruption.
- **Asynchronous Scanning:** Built with Go's goroutines to perform non-blocking parallel file scanning, keeping the Fyne GUI responsive even with libraries containing 100,000+ files.

## Features

- Orphaned mods cleanup
- Auto-scan for `.wabbajack` files
- Checkbox modlist selection
- Shared mod protection (never deletes mods used by multiple modlists)
- Old version cleanup with safety checks (use with caution)
- Safe deletion folder (timestamped, restorable)
- Windows GUI (Fyne Framework)
- Real-time progress bar & async processing
- Statistics view
- Complete logging

## Usage

### Installation

Download `wabbajack-library-cleaner.exe` from [Releases](../../releases). No installation needed.

### Quick Start

**Step 1: Select Modlist Folder**
- Click "Select Modlist Folder"
- Navigate to folder with `.wabbajack` files
  - Common: `C:\Users\YourName\AppData\Local\Wabbajack\downloaded_mod_lists\`
- Check ONLY your active modlists

**Step 2: Select Downloads Folder**
- Click "Select Downloads Folder"
- Choose your Wabbajack downloads directory
  - Parent folder: `F:\Wabbajack` (scans all game folders)
  - Specific game: `F:\Wabbajack\Fallout 4` (scans only one)

**Step 3: Configure Deletion**
- "Move to deletion folder" (recommended - restorable)
  - Location: `[Downloads]\WLC_Deleted\[timestamp]\`
  - Example: `F:\Wabbajack\WLC_Deleted\2025-11-14_15-30-45\`
- Uncheck for permanent deletion (cannot be undone)

**Step 4: Clean**

**Orphaned Mods:**
1. "Scan for Orphaned Mods" - analyzes which mods are used
2. Review output
3. "Clean Orphaned Mods" - moves files after confirmation

**Old Versions (use with caution):**
1. "Scan for Old Versions" - select game folder
2. Review duplicates
3. "Clean Old Versions" - keeps newest only

**Statistics:**
- View library size by game

### Requirements

**Orphaned Mods:**
- Need `.wabbajack` files for ALL active modlists
- Missing file = mods marked as orphaned
- Verify all active modlists are checked

**Old Versions:**
- Checks timestamps, file sizes, patch detection, content descriptors, version normalization
- Has safety checks but may still miss edge cases
- May delete old versions your modlist needs
- Use orphaned cleanup instead (safer)

### Build from Source

**Requirements:**
- Go 1.21+
- MinGW-w64 GCC: `choco install mingw` or from [TDM-GCC](https://jmeubank.github.io/tdm-gcc/download/)

**Build:**
```bash
set CGO_ENABLED=1
go build -trimpath -ldflags="-s -w -H=windowsgui" -o wabbajack-library-cleaner.exe
```

## How It Works

Parses Nexus Mods/Wabbajack filename format:
```
ModName-ModID-Version-Timestamp.extension
```

Example: `Alternate Perspective-50307-4-0-3-1731841209.zip`
- ModID: `50307` - matched against `.wabbajack` files
- Timestamp: `1731841209` - for version detection

**Orphaned cleanup:** Matches ModID against selected modlists  
**Old version cleanup:** Groups by ModID, keeps newest timestamp

## Logging

Operations logged to: `wabbajack-library-cleaner_YYYY-MM-DD_HH-MM-SS.log`

## Technical

- Supported: `.7z`, `.zip`, `.rar`, `.tar`, `.gz`, `.exe`
- Fast scanning, low memory
- Single executable, no dependencies

## Common Issues

**"No .wabbajack files found"**
- Check correct folder: `C:\Users\YourName\AppData\Local\Wabbajack\downloaded_mod_lists\`

**"Too many mods marked as orphaned"**
- Check ALL active modlists
- Missing `.wabbajack` file? Re-download modlist

**"Failed to parse .wabbajack file"**
- File corrupted - re-download modlist

**"File is locked"**
- Close MO2, Wabbajack, mod tools

**"Deleted version my modlist needs"**
- Old version cleanup has multiple safety checks but may miss edge cases
- Re-run Wabbajack to re-download

**"Many files skipped"**
- Normal - skips `.meta` and non-standard filenames

## License

MIT - see [LICENSE](LICENSE)

## Expected Savings

Space savings depend on number of modlists, mods, and file sizes. Can range from a few GB to hundreds of GB.

---

**Version:** v2.0.1

See [CHANGELOG](CHANGELOG.md) for history.