# Makefile Quick Reference

## Platform Support

✅ **Windows** | ✅ **macOS** | ✅ **Linux**

---

## Common Commands

```bash
# View help (shows current platform)
make help

# Check environment
make check

# Show project info
make info

# Download resources
make resources

# Install dependencies
make install

# Development mode
make dev

# Build application
make build

# Run tests
make test

# Clean build artifacts
make clean

# Clean everything
make clean-all
```

---

## Platform-Specific Behavior

### Windows
- Downloads: `node-windows-x64.zip`
- Outputs: `.msi` / `.exe`
- Uses: PowerShell scripts

### macOS
- Downloads: `node-macos-arm64.tar.gz` + `node-macos-x64.tar.gz`
- Outputs: `.dmg` / `.app`
- Uses: Bash scripts

### Linux
- Downloads: `node-linux-x64.tar.gz`
- Outputs: `.AppImage` / `.deb`
- Uses: Bash scripts

---

## Quick Start

```bash
# One command to do everything
make quickstart

# Or step by step
make check      # Verify environment
make resources  # Download resources  
make build      # Build application
```

---

## Documentation

- `MAKEFILE_GUIDE.md` - Complete guide
- `MAKEFILE_CROSS_PLATFORM.md` - Platform details
- `CROSS_PLATFORM_SUMMARY.md` - Upgrade summary

---

**Version**: 2.0 (Cross-Platform)  
**Last Updated**: 2026-02-15
