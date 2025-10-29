# Changelog

All notable changes to Wabbajack Library Cleaner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2025-10-29

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

## [1.0.0] - 2025-10-28

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
