# Wabbajack Library Cleaner (Rust Version)

Cross-platform GUI tool to clean orphaned mods and old versions from your Wabbajack downloads folder.

## Features

- **Orphaned Mods Cleanup**: Remove mods not used by selected modlists
- **Old Versions Cleanup**: Keep newest version of each mod, remove duplicates
- **Safe Deletion**: Move files to timestamped backup folder (restorable)
- **Statistics View**: View library size by game
- **Modern GUI**: Built with egui/eframe

## Binary Size

The release binary is approximately **6-7 MB** (compared to ~15-25 MB for the Go/Fyne version).

## Building

### Requirements

- Rust 1.88+ 
  - Note: eframe 0.30+ has a known compatibility issue with Rust 1.92+ due to winit 0.30.12. This project uses eframe 0.29 which works with all Rust versions.
- Linux: `libxkbcommon-dev`, `libwayland-dev` (for Wayland support)
- Windows: No additional dependencies

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized for size)
cargo build --release

# Run tests
cargo test
```

### Release Build Optimizations

The release profile is configured for minimal binary size:
- `opt-level = "z"` - Optimize for size
- `lto = true` - Link-Time Optimization
- `codegen-units = 1` - Better optimization
- `panic = "abort"` - Smaller binary
- `strip = true` - Strip symbols

For faster runtime at cost of larger binary:
```bash
cargo build --profile release-fast
```

## Project Structure

```
rust-version/
├── Cargo.toml          # Dependencies and build config
├── src/
│   ├── main.rs         # Entry point
│   ├── core/           # Core logic
│   │   ├── mod.rs
│   │   ├── types.rs    # Data structures
│   │   ├── parser.rs   # Mod filename and .wabbajack parsing
│   │   ├── scanner.rs  # Directory scanning and orphan detection
│   │   └── cleaner.rs  # File deletion and backup
│   └── gui/            # GUI layer
│       ├── mod.rs
│       └── app.rs      # eframe/egui application
```

## Dependencies

- **eframe/egui 0.29**: GUI framework
- **rfd**: Native file dialogs
- **serde/serde_json**: JSON parsing for .wabbajack files
- **zip**: Archive handling
- **chrono**: Date/time formatting
- **anyhow/thiserror**: Error handling
- **log/env_logger**: Logging

## License

GPL-3.0-or-later - See [LICENSE](../LICENSE)
