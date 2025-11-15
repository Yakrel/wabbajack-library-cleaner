# Wabbajack Library Cleaner v2.0

Windows GUI tool to clean orphaned mods and old versions from your Wabbajack downloads folder.

## What It Does

### PRIMARY: Orphaned Mods Cleanup (50-150 GB typical)

Removes mods from deleted/inactive modlists. Compares your files against active modlists using `.wabbajack` files.

**Requires:** `.wabbajack` files for ALL active modlists. Without them, tool can't tell which mods to keep.

### SECONDARY: Old Version Cleanup (10-30 GB typical)

Keeps newest version of each mod, removes old duplicates.

**Warning:** Based on file timestamps only, not actual modlist requirements. Use with caution.

## Screenshots

![Main Interface](screenshots/main-interface.png)
![Orphaned Mods Scan](screenshots/orphaned-scan.png)
![Old Versions Scan](screenshots/old-versions-scan.png)

## Features

- Orphaned mods cleanup (50-150 GB typical)
- Auto-scan for `.wabbajack` files
- Checkbox modlist selection
- Shared mod protection (never deletes mods used by multiple modlists)
- Old version cleanup (use with caution)
- Safe deletion folder (timestamped, restorable)
- Windows GUI
- Real-time progress bar
- Statistics view
- Complete logging

## Safety Features

- Deletion folder by default (timestamped, restorable)
- Preview mode before deletion
- Confirmation dialogs
- File lock detection
- Shared mod protection
- Complete logging
- Skips temp files (`.part`, `.tmp`)

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
- Tool uses timestamps only, not actual modlist requirements
- May delete old versions your modlist needs
- Use orphaned cleanup instead (safer)

### Build from Source

**Requirements:**
- Go 1.21+
- MinGW-w64 GCC: `choco install mingw` or from [TDM-GCC](https://jmeubank.github.io/tdm-gcc/download/)

**Build:**
```bash
set CGO_ENABLED=1
go build -ldflags="-s -w -H=windowsgui" -o wabbajack-library-cleaner.exe
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
- Old version cleanup uses timestamps, not modlist requirements
- Re-run Wabbajack to re-download

**"Many files skipped"**
- Normal - skips `.meta` and non-standard filenames

## License

MIT - see [LICENSE](LICENSE)

## Expected Savings

- Old versions: 10-30 GB
- Orphaned mods: 50-150 GB
- Combined: 100+ GB

---

**Version:** v2.0.1

See [CHANGELOG](CHANGELOG.md) for history.
