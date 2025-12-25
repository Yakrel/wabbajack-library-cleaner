# Changelog

All notable changes to Wabbajack Library Cleaner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 2.1.0 - 2025-12-25

### Major Rewrite (Rust)
- **Complete Rewrite in Rust:** The application has been rewritten from scratch using Rust for maximum performance and stability.
- **Smaller Binary:** Executable size reduced by ~60% (from ~15MB to ~6MB).
- **Parallel Processing:** Integrated `rayon` for multi-threaded file scanning and hash calculation. Uses 100% of CPU cores for lightning-fast scans.
- **Memory Safety:** Eliminated all race conditions and memory leaks thanks to Rust's ownership model.

### Added
- **Modern GUI:** New interface built with `egui` (eframe), providing a smoother and more responsive experience.
- **Native Dialogs:** Uses native OS file pickers via `rfd`.
- **Automated Releases:** GitHub Actions workflow now automatically builds and releases binaries for both Windows and Linux on new tags.
- **Cross-Platform:** Native Linux support added (compiled as binary).

### Changed
- Moved the legacy Go implementation to `legacy_go/` directory for archival purposes.
- Defaulted to "Release" profile optimizations (`lto`, `strip`, `opt-level='z'`) for smallest possible binary size.
- Improved logging format and output window in the GUI.

## 2.0.1 - 2025-11-15

### Fixed
- Fixed UI thread violations causing 2M+ error log lines
- Fixed loop variable capture bugs in closures  
- Removed unnecessary code nesting
- Tested with 435 file deletions - no errors

## 2.0.0 - 2025-11-14

### Major Changes
- Complete rewrite as Windows-only GUI application
- CLI mode removed
- Orphaned mods cleanup is now the primary feature
- Simplified step-by-step workflow

### Added
- Auto-scanning for .wabbajack files with checkboxes
- Safe deletion folder (files moved to timestamped backup)
- Real-time progress bar with percentage
- Reorganized UI with primary/secondary feature separation
- About dialog with license info
- Support for both parent and single game folder selection

### Removed
- CLI mode completely removed

### Changed
- Modlist selection in main window (no separate dialogs)
- Deletion folder enabled by default
- Window size increased to 1200x900
- Operations run in goroutines (no UI freeze)
- Auto-scroll to progress section during operations

## 1.0.2 - 2025-11-12

### Added
- Orphaned mods cleanup feature
  - Parses .wabbajack files to determine needed mods
  - Interactive modlist selection
  - Typical space savings: 50-150 GB
- Statistics view showing library breakdown by game
- Enhanced menu with 6 options

### Security
- Double confirmation required for orphaned mods deletion
- Shared mods always protected
- File lock detection

## 1.0.1 - 2025-10-29

### Fixed
- Fixed duplicate detection algorithm
  - Now uses ModID + normalized ModName + part indicator
  - Multi-part mod protection (e.g., "-1- Meshes" and "-2- Textures" kept separate)
  - Version pattern normalization
  - Timestamp uniqueness check
- Enhanced safety checks
  - Detects same-version files with different content (10x+ size difference)
  - Patch/hotfix detection to preserve base files
  - Content descriptor detection (texture quality, body types, etc.)
- Won't delete files if all timestamps are identical

### Added
- Application icon with transparency
- Comprehensive logging of skipped groups

## 1.0.0 - 2025-10-28

Initial CLI release.

### Features
- Mod version detection using ModID and timestamp
- Interactive menu system
- Dry-run mode
- Safety checks (file locks, version protection, group validation)
- Size-based filtering
- Multi-game support
- Automatic .meta file cleanup
- Deletion logging

---

[1.0.1]: https://github.com/Yakrel/wabbajack-library-cleaner/releases/tag/v1.0.1
[1.0.0]: https://github.com/Yakrel/wabbajack-library-cleaner/releases/tag/v1.0.0
