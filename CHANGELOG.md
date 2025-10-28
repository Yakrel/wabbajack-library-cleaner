# Changelog

All notable changes to Wabbajack Library Cleaner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[1.0.0]: https://github.com/Yakrel/wabbajack-library-cleaner/releases/tag/v1.0.0
