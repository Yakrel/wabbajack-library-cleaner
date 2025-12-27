# Current Status: Refactoring for Integration Tests

We are in the middle of refactoring the project to support robust **Integration Tests** without requiring large (250MB+) binary files in the repository.

## 1. What has been done
- **UI Redesign:** The UI (`src/gui/app.rs`) has been completely rewritten to a "Modern Dashboard" style.
- **Library Split:** Created `src/lib.rs` and modified `Cargo.toml` to define both a `[lib]` and `[[bin]]` target. This allows `tests/` to import the core logic as a library.
- **Main Update:** Updated `src/main.rs` to consume the new library crate instead of declaring modules locally.
- **Test Creation:** Created `tests/integration_test.rs` which programmatically creates dummy `.wabbajack` ZIP files and fake mod files to test detection logic.

## 2. What is Broken / Incomplete (TODO)

### A. Fix Integration Test Compilation
The file `tests/integration_test.rs` fails to compile with the following error:
```text
error[E0283]: type annotations needed for `FileOptions<'_, _>`
   --> tests/integration_test.rs:17:9
    |
 17 |     let options = FileOptions::default()
```
**Fix:** Change line 17 to specify the type:
```rust
let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
// OR specifically:
let options: FileOptions<()> = FileOptions::default();
```

### B. Verify Imports in `src/gui/app.rs`
Check if `src/gui/app.rs` still compiles correctly as part of the library. It uses `use crate::core::...`. Since it is now a module under `src/lib.rs`, `crate::` refers to the library root, so this should be correct. However, please run `cargo check` to ensure no circular dependencies or visibility issues were introduced.

### C. Remove Large Test Files
Once the integration test passes:
1. Delete `tests/Beginagain_@@_beginagain.wabbajack` (250MB).
2. Delete `tests/Beginagain_@@_beginagain.wabbajack.metadata`.

### D. CI/CD Pipeline
Add the new `cargo test` command to the GitHub Actions workflow to ensure the synthetic test runs on every commit.

## 3. How to Resume
1. Run `cargo test --test integration_test` to see the current errors.
2. Fix the `FileOptions` type inference in `tests/integration_test.rs`.
3. Verify the test passes.
4. Delete the large files.
