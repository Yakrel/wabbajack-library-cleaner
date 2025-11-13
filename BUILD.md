# Building Wabbajack Library Cleaner

## Prerequisites

- Go 1.25 or later
- For Windows build: Windows 10+ with Go installed
- For GUI build: Fyne dependencies (automatically downloaded by Go)

## Building

### Windows GUI (Recommended)

```bash
# Install Go if you haven't already
# Download from: https://go.dev/dl/

# Clone the repository
git clone https://github.com/Yakrel/wabbajack-library-cleaner.git
cd wabbajack-library-cleaner

# Download dependencies
go mod download

# Build GUI version (default)
go build -ldflags="-s -w -H windowsgui" -o wabbajack-library-cleaner.exe

# The -H windowsgui flag prevents console window from appearing
```

### Windows CLI Only

If you want a CLI-only version without GUI dependencies:

```bash
# Build without GUI (smaller binary)
go build -tags nogui -ldflags="-s -w" -o wabbajack-library-cleaner-cli.exe
```

### Linux Build

```bash
# Install required dependencies first
sudo apt-get install libgl1-mesa-dev xorg-dev

# Build
go build -ldflags="-s -w" -o wabbajack-library-cleaner
```

### Cross-Compiling from Linux to Windows

```bash
# This may not work due to CGO dependencies in Fyne
# Recommended to build on Windows for Windows target

# If you must try:
GOOS=windows GOARCH=amd64 go build -ldflags="-s -w" -o wabbajack-library-cleaner.exe
```

## Build Flags Explained

- `-ldflags="-s -w"`: Strip debug information to reduce binary size
- `-H windowsgui`: Hide console window on Windows (GUI mode only)
- `-tags nogui`: Build without GUI dependencies (not recommended for end users)

## Troubleshooting

### "C compiler not found" or similar CGO errors

Fyne requires CGO for GUI rendering. On Windows:
1. Install MinGW-w64 or TDM-GCC
2. Add to PATH
3. Try building again

### Missing dependencies on Linux

Install development packages:
```bash
# Ubuntu/Debian
sudo apt-get install golang libgl1-mesa-dev xorg-dev

# Fedora
sudo dnf install golang mesa-libGL-devel libXcursor-devel libXrandr-devel libXinerama-devel libXi-devel libXxf86vm-devel
```

## Size Optimization

The GUI binary will be larger (~20-30 MB) due to GUI framework. This is normal.

For smaller builds, use UPX compression (optional):
```bash
# Install UPX from https://upx.github.io/
upx --best --lzma wabbajack-library-cleaner.exe
```

This can reduce size by 60-70%.
