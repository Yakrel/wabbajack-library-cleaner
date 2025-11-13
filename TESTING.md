# Testing Guide

## Running Tests Locally

### Prerequisites
- Go 1.21 or later
- Windows OS (for GUI components)

### Run All Tests
```bash
go test -v ./...
```

### Run Specific Tests
```bash
# Core functionality tests (no GUI required)
go test -v -run "^(TestParseModFilename|TestDetectOrphanedMods|TestIsNumeric)$"

# File operations tests
go test -v -run "^(TestIsValidPath|TestFileExists|TestDeleteFile)$"
```

### Run Tests with Coverage
```bash
go test -v -coverprofile=coverage.txt -covermode=atomic ./...
```

### View Coverage Report
```bash
go tool cover -html=coverage.txt
```

## Test Structure

### Core Tests (`wabbajack-library-cleaner_test.go`)
- `TestParseModFilename` - Tests mod filename parsing with ModID and FileID extraction
- `TestDetectOrphanedMods` - Tests orphaned mod detection algorithm
- `TestDetectOrphanedModsWithPreciseFileID` - Tests precise FileID matching
- `TestIsNumeric` - Tests numeric string validation

### File Operations Tests (`fileops_test.go`)
- `TestIsValidPath` - Tests path validation
- `TestFileExists` - Tests file existence checking
- `TestDeleteFile` - Tests file deletion

## CI/CD Pipeline

The GitHub Actions workflow runs:
1. **Tests** - Non-GUI unit tests on Windows
2. **Build** - Compiles the Windows GUI executable
3. **Lint** - Runs gofmt, go vet, and golangci-lint

### Why Non-GUI Tests Only in CI?
GUI tests require a display/window manager which isn't available in CI environments. The non-GUI tests cover:
- Filename parsing logic
- Orphaned mod detection algorithm
- File operations
- Path validation

These tests validate the core business logic without requiring GUI components.

## Test Coverage

Current test coverage focuses on:
- ✅ Mod filename parsing (ModID, FileID extraction)
- ✅ Orphaned mod detection logic
- ✅ Path validation
- ✅ File operations
- ✅ Numeric validation

## Writing New Tests

When adding new functionality:
1. Write tests for core logic (non-GUI)
2. Use table-driven tests for multiple scenarios
3. Test both success and failure cases
4. Keep tests independent and idempotent

Example:
```go
func TestNewFeature(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    string
        wantErr bool
    }{
        {"valid input", "test", "expected", false},
        {"invalid input", "", "", true},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := NewFeature(tt.input)
            if (err != nil) != tt.wantErr {
                t.Errorf("NewFeature() error = %v, wantErr %v", err, tt.wantErr)
                return
            }
            if got != tt.want {
                t.Errorf("NewFeature() = %v, want %v", got, tt.want)
            }
        })
    }
}
```

## Troubleshooting

### "cannot find package" error
```bash
go mod download
go mod tidy
```

### GUI-related build errors
GUI components require Windows and Fyne dependencies. The tests are designed to run without GUI components by testing only core logic.

### Test timeout
Increase timeout for slow operations:
```bash
go test -timeout 30s ./...
```
