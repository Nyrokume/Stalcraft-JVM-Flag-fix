# Go to Tauri Migration Summary

## What Was Converted

This document summarizes the migration from the Go-based JVM wrapper to a Tauri-based application with a modern GUI.

## Original Go Project (wrapper/)

The original Go project consisted of:
- `main.go` - Entry point, process management, console hiding
- `jvm.go` - JVM flag generation logic
- `install.go` - IFEO registry installation
- `menu.go` - Interactive CLI menu
- `system.go` - System hardware detection
- `go.mod` - Go module definition

## New Tauri Project (stalcraft-jvm-wrapper/)

### Backend (Rust)

| Go File | Rust File | Description |
|---------|-----------|-------------|
| `main.go` | `main.rs` + `commands.rs` | Tauri app entry, IPC commands |
| `jvm.go` | `jvm.rs` | JVM flag generation (identical logic) |
| `install.go` | `ifeo.rs` | IFEO registry management |
| `menu.go` | Frontend (JS) | Interactive menu replaced with GUI |
| `system.go` | `system.rs` | Hardware detection via Windows API |
| `go.mod` | `Cargo.toml` | Rust dependencies |

### Frontend (New)

| File | Purpose |
|------|---------|
| `index.html` | Main HTML structure |
| `styles.css` | Modern dark theme styling |
| `main.js` | Frontend logic and Tauri IPC |

## Key Changes

### Replaced
- ❌ CLI menu system → ✅ Modern GUI with buttons
- ❌ Console window manipulation → ✅ Native Tauri window
- ❌ Go standard library → ✅ Rust + Windows API (windows-sys)
- ❌ Terminal-only interface → ✅ Rich visual interface

### Preserved
- ✅ All JVM flag generation logic (identical algorithms)
- ✅ IFEO registry management (same registry paths)
- ✅ Process boosting (same Windows API calls)
- ✅ System detection (RAM, CPU, large pages)
- ✅ All JVM flag calculations and formulas

### Improved
- ✅ Real-time system information display
- ✅ Visual feedback for all operations
- ✅ Error handling with user-friendly messages
- ✅ Both IFEO and direct launch options
- ✅ Responsive design for different window sizes

## Technology Stack

### Go Version
- **Language**: Go 1.25
- **Dependencies**: golang.org/x/sys (Windows registry)
- **UI**: CLI with interactive menu
- **Binary**: ~5-10 MB

### Tauri Version
- **Backend**: Rust 2021 Edition
- **Frontend**: Vanilla JS + CSS
- **Framework**: Tauri 2.x
- **Dependencies**: 
  - windows-sys 0.59 (Windows API)
  - tauri 2.x (App framework)
  - serde (Serialization)
- **Binary**: ~10-15 MB (with embedded webview)

## Building

### Go Version
```bash
go build -o wrapper.exe -ldflags="-s -w" .
```

### Tauri Version
```bash
npm install
npm run build
# or
cargo build --manifest-path src-tauri/Cargo.toml --release
```

## Usage Comparison

### Go Version
```bash
# Interactive menu
wrapper.exe

# Direct commands
wrapper.exe --install
wrapper.exe --uninstall
wrapper.exe --status

# As IFEO debugger (automatic)
wrapper.exe stalcraft.exe [args...]
```

### Tauri Version
```bash
# GUI application
npm run dev    # Development
npm run build  # Production

# All operations through GUI buttons
# - Refresh System Info
# - Install (IFEO)
# - Uninstall (IFEO)
# - Check Status
# - Launch Game
```

## File Structure Comparison

### Go Version
```
wrapper/
├── main.go          (150 lines)
├── jvm.go           (141 lines)
├── install.go       (89 lines)
├── menu.go          (120 lines)
├── system.go        (53 lines)
├── go.mod
└── go.sum
Total: ~553 lines
```

### Tauri Version
```
stalcraft-jvm-wrapper/
├── src/
│   ├── index.html   (52 lines)
│   ├── styles.css   (191 lines)
│   └── main.js      (103 lines)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs       (19 lines)
│   │   ├── commands.rs   (69 lines)
│   │   ├── jvm.rs        (158 lines)
│   │   ├── ifeo.rs       (164 lines)
│   │   ├── system.rs     (54 lines)
│   │   └── process.rs    (200 lines)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
Total: ~1010 lines (including styling and config)
```

## Performance

Both versions provide:
- ⚡ Near-instant JVM flag generation
- 🚀 Same process boosting capabilities
- 💾 Minimal resource usage
- 🎯 Identical optimization results

## When to Use Which

**Go Version** - Best for:
- Headless/server environments
- Minimal binary size
- CLI-only workflows
- Integration with scripts

**Tauri Version** - Best for:
- Desktop users who prefer GUI
- Visual system information
- One-click operations
- Users who want direct launch option
- Modern application experience

## Migration Path

If you have the Go version installed:
1. Uninstall Go version (`wrapper.exe --uninstall`)
2. Build/run Tauri version
3. Use GUI to install IFEO hook
4. Continue using STALCRAFT normally

Both versions use the same IFEO registry keys, so they're interchangeable.

## Future Enhancements (Tauri)

Potential additions:
- 📊 Real-time performance monitoring
- 📝 Launch history and statistics
- ⚙️ Custom JVM flag presets
- 🔄 Auto-update notifications
- 🌐 Multi-language support (RU/EN)
- 📱 System tray integration
