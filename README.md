# Wabbajack Library Cleaner

Cross-platform GUI tool to clean orphaned mods and old versions from your Wabbajack downloads folder.

## Features

- **Orphaned Mods Cleanup**: Remove mods not used by selected modlists
- **Old Versions Cleanup**: Keep newest version of each mod, remove duplicates
- **Safe Deletion**: Move files to timestamped backup folder (restorable)
- **Statistics View**: View library size by game
- **Modern GUI**: Built with egui/eframe

## Download

Download the latest release from the [Releases](https://github.com/Yakrel/wabbajack-library-cleaner/releases) page.

## Building from Source

### Requirements

- Rust 1.70+
- For Windows cross-compilation from Linux: `mingw-w64`

### Build Commands

```bash
# Linux build
cargo build --release

# Windows build (native on Windows)
cargo build --release

# Windows build (cross-compile from Linux)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# Run tests
cargo test
```

### Output Locations

- Linux: `target/release/wabbajack-library-cleaner`
- Windows (native): `target/release/wabbajack-library-cleaner.exe`
- Windows (cross): `target/x86_64-pc-windows-gnu/release/wabbajack-library-cleaner.exe`

### Release Build Optimizations

The release profile is configured for minimal binary size (~4-7 MB):
- `opt-level = "z"` - Optimize for size
- `lto = true` - Link-Time Optimization
- `codegen-units = 1` - Better optimization
- `strip = true` - Strip symbols

## Project Structure

```
.
├── Cargo.toml          # Dependencies and build config
├── build.rs            # Build script for Windows icon
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Library exports
│   ├── core/           # Core logic
│   │   ├── types.rs    # Data structures
│   │   ├── parser.rs   # Mod filename and .wabbajack parsing
│   │   ├── scanner.rs  # Directory scanning and orphan detection
│   │   └── cleaner.rs  # File deletion and backup
│   └── gui/            # GUI layer
│       └── app.rs      # eframe/egui application
├── tests/              # Integration tests
│   ├── integration_test.rs
│   └── fixtures/       # Test fixtures
└── winres/             # Windows resources (icons)
```

## Dependencies

- **eframe/egui 0.29**: GUI framework
- **rayon**: Parallel processing
- **rfd**: Native file dialogs
- **serde/serde_json**: JSON parsing
- **zip**: Archive handling

## License

GPL-3.0-or-later - See [LICENSE](LICENSE)
