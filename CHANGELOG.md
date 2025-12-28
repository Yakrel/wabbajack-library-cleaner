# Changelog

All notable changes to Wabbajack Library Cleaner will be documented in this file.

## 2.1.0 - 2025-12-28

### Major Rewrite (Rust)
- Complete rewrite in Rust for maximum performance and stability
- Executable size reduced by ~60% (from ~15MB to ~6MB)
- Parallel processing with rayon for lightning-fast scans
- Memory safe with no race conditions or memory leaks

### Added
- Modern GUI built with egui/eframe
- Native OS file dialogs
- Automated releases via GitHub Actions
- Cross-platform support (Windows & Linux)
- Full support for generic archive files (zip/rar/7z without standard naming)
- Validated with real-world Wabbajack modlist data

### Changed
- Improved GUI layout (1280x800 default size)
- Modern 2-column About window
- Optimized Log panel usage
- Legacy Go implementation moved to `legacy_go/`
- Release profile optimizations for smallest binary size
- Improved logging in GUI

## 2.0.1 - 2025-11-15

### Fixed
- UI thread violations causing error log spam
- Loop variable capture bugs in closures
- Tested with 435 file deletions successfully

## 2.0.0 - 2025-11-14

### Major Changes
- Complete rewrite as Windows GUI application
- CLI mode removed
- Orphaned mods cleanup as primary feature

### Added
- Auto-scanning for .wabbajack files with checkboxes
- Safe deletion folder (timestamped backup)
- Real-time progress bar
- About dialog with license info

### Changed
- Modlist selection in main window
- Deletion folder enabled by default
- Window size increased to 1200x900
- Async operations (no UI freeze)

## 1.0.2 - 2025-11-12

### Added
- Orphaned mods cleanup feature
- Statistics view by game
- Enhanced menu with 6 options

### Security
- Double confirmation for deletion
- Shared mods protection
- File lock detection

## 1.0.1 - 2025-10-29

### Fixed
- Duplicate detection algorithm improvements
- Enhanced safety checks for version detection
- Patch/hotfix detection

### Added
- Application icon
- Comprehensive logging

## 1.0.0 - 2025-10-28

Initial CLI release with mod version detection, dry-run mode, and safety checks.
