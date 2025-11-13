# Changelog

All notable changes to Wabbajack Library Cleaner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 2.0.0 - 2025-11-13

### Major Changes
- **🖼️ Windows-Only GUI Application** - Complete rewrite as a GUI-only Windows application
- **🚫 CLI Mode Removed** - Simplified to GUI only (Wabbajack is Windows-only anyway)
- **🎯 Orphaned Mods = PRIMARY Feature** - Old versions cleanup is now secondary
- **📁 Simplified Workflow** - Step-by-step process that's easier to understand

### Added
- **NEW: Auto-Scanning Modlists** - Select folder → Tool automatically scans for .wabbajack files
  - Shows checkboxes for each found modlist
  - All modlists selected by default
  - "Select All" / "Deselect All" buttons
  - No more separate dialogs - everything in one place
- **NEW: Checkbox Modlist Selection** - Select which modlists you're currently using
  - Check/uncheck modlists directly in the main window
  - Clear visual indication of selected modlists
  - Selected modlists used for orphaned mods detection
- **NEW: Recycle Bin Support** - Files go to Recycle Bin by default
  - Checkbox to toggle between Recycle Bin and permanent deletion
  - Default: Recycle Bin (safer - can be restored)
  - Windows native implementation using SHFileOperation
- **NEW: Reorganized UI** - Primary and Secondary features clearly separated
  - PRIMARY: Orphaned Mods Cleanup (highlighted with high importance)
  - SECONDARY: Old Versions Cleanup (with warning about compatibility)
  - Warning labels about modlists requiring old versions
  - Emojis for better visual navigation (🔍, 🧹, 📁, 🗑️, ⚠️, etc.)
- **NEW: Simplified Directory Selection** - Only two folders needed
  - Step 1: Modlist folder (contains all .wabbajack files)
  - Step 2: Downloads folder (contains game mod folders)
- All features from v1.0.2 preserved and enhanced

### Removed
- **CLI Mode Completely Removed** - GUI-only application now
  - Simplified codebase
  - Better user experience
  - Windows-only focus (Wabbajack is Windows-only)

### Changed
- Main workflow completely redesigned for better UX
- Modlist selection now happens in Step 1 (not in a separate dialog)
- Actions reorganized to emphasize orphaned mods cleanup
- Enhanced safety with Recycle Bin as default deletion method
- Better status messages and output formatting

### Technical Details
- Added `fyne.io/fyne/v2` for cross-platform GUI framework
- New files:
  - `gui.go`: Complete GUI implementation
  - `fileops.go`: File operation helpers
  - `recyclebin_windows.go`: Windows Recycle Bin support
  - `recyclebin_unix.go`: Unix/Linux stub (falls back to regular deletion)
- Reorganized main() to support both GUI and CLI modes

## 1.0.2 - 2025-11-12

### Added
- **FEATURE: Orphaned Mods Cleanup** - Remove mods not used by any active modlist
  - Parses `.wabbajack` files to determine which mods are actually needed
  - Interactive modlist selection (choose which modlists you're actively using)
  - Detailed reporting showing used vs orphaned mods
  - Typical space savings: 50-150 GB from deleted/inactive modlists
  - Double confirmation required ("DELETE" in uppercase) for safety
  - Automatic .meta file cleanup with main archives
- **NEW: Statistics View** (Menu option 5)
  - View total files and size across all game folders
  - Breakdown by individual game
- **Enhanced Menu System**
  - Option 1: Scan folder (Dry-run) - Preview old versions
  - Option 2: Clean folder - Delete old versions
  - Option 3: Scan for orphaned mods (Dry-run) - Preview unused mods
  - Option 4: Clean orphaned mods - Delete unused mods
  - Option 5: View statistics
  - Option 6: Exit

### Technical Details
- Added JSON parsing for `.wabbajack` files (ZIP archives)
- Added `archive/zip` and `encoding/json` imports
- New data structures:
  - `ModlistArchive`: Represents archive entries in modlists
  - `ModlistModState`: Contains ModID, FileID, GameName, etc.
  - `Modlist`: Full modlist structure
  - `ModlistInfo`: Tracking information for each modlist
  - `OrphanedMod`: Represents an unused mod file
- New functions:
  - `findWabbajackFiles()`: Locates .wabbajack files
  - `parseWabbajackFile()`: Extracts and parses modlist data
  - `getAllModFiles()`: Collects all mod files from game folders
  - `detectOrphanedMods()`: Classifies mods as used or orphaned
  - `scanOrphanedMods()`: Main workflow for orphaned mods feature
  - `showOrphanedReport()`: Displays detailed analysis
  - `deleteOrphanedMods()`: Safely removes orphaned files
  - `viewStatistics()`: Shows library statistics

### Security
- Orphaned mods feature includes multiple safety checks
- Shared mods (used by multiple modlists) are always protected
- File lock detection prevents deletion of in-use files
- Dry-run mode required before actual deletion
- All operations logged for audit trail

## 1.0.1 - 2025-10-29

### Fixed
- **Critical:** Fixed duplicate detection algorithm to correctly identify mod versions
  - Now uses ModID + normalized ModName + part indicator instead of just ModID
  - Prevents accidental deletion of different files from same mod page (e.g., "Armor" vs "Hair" files)
  - **Multi-part mod protection:** Keeps parts separate (e.g., "Mod -1- Meshes" and "Mod -2- Textures" are NOT grouped)
    - Detects patterns: `-1-`, `-2-`, `Part 1`, `Part 2`, `(Part 1)`, `Pt1`, etc.
    - Prevents accidental deletion of required mod components
  - Added version pattern normalization to group different versions of same file (e.g., "Interface 1.3.6" and "Interface 1.4.0" are now correctly grouped)
  - Added timestamp uniqueness check to prevent grouping identical files
  - **Enhanced Safety:** Detects and skips same-version files with different content (variants)
    - Skips files with same version but 10x+ size difference (e.g., "ESP only" vs "Full textures")
    - Skips files uploaded within 1 hour with same version (likely different variants like "CLEAN" vs "DIRTY")
  - **Patch/Hotfix Detection:** Prevents deletion of base files when only patches/hotfixes are present
    - Detects patch keywords: "patch", "hotfix", "update", "fix" in filenames
    - Skips groups where newest file is <10% size of older versions (likely a patch that needs the base file)
    - Skips groups containing both "PATCH" and "MAIN" labeled files
    - Prevents accidental deletion of full mod files when keeping small patches
  - **Content Descriptor Detection:** Prevents grouping files with different content types
    - Detects 8 categories of descriptors: texture quality (1K/2K/4K/8K), body types (CBBE/UUNP), mod components (armor/weapons/clothes), file types (ESP/ESM), compatibility variants (ASLAL/No Worldspace), editions (Lite/Full/Extended), clean variants, and optional content
    - Skips groups where files have different descriptors (e.g., "2K Textures" vs "Gloves Only")
    - Skips groups where one file has descriptors but another doesn't (e.g., "No Worldspace Edits" vs base version)
    - Prevents accidental deletion of different variants or mod components
- Improved safety: Will not delete files if all timestamps are identical

### Added
- Application icon with transparency
- Enhanced duplicate grouping algorithm with version normalization
- Suspicious pattern detection for same-version different-content files
- Patch/hotfix detection system to preserve base files
- Comprehensive logging of skipped groups for audit purposes

## 1.0.0 - 2025-10-28

### Added
- Initial release
- Smart mod version detection using ModID and timestamp
- Interactive menu system with 6 options
- Dry-run mode for safe preview
- Comprehensive safety checks:
  - File lock detection
  - Newest version protection
  - Mod group validation
  - Temp file filtering
- Detailed logging with timestamps
- Size-based filtering option
- Multi-game support (Skyrim, Fallout 4, Fallout 3-NV, etc.)
- Automatic .meta file cleanup
- Color-coded terminal output
- Double confirmation for deletion operations

### Security
- Never deletes the newest version
- Validates file patterns before processing
- Skips files in use by other programs
- Logs all operations for audit trail

### Performance
- Fast scanning (~10,000 files/second)
- Low memory footprint
- Compiled binary size: 2.8 MB
- No external dependencies

---

[1.0.1]: https://github.com/Yakrel/wabbajack-library-cleaner/releases/tag/v1.0.1
[1.0.0]: https://github.com/Yakrel/wabbajack-library-cleaner/releases/tag/v1.0.0
