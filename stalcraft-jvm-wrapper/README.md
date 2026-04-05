# STALCRAFT JVM Wrapper - Tauri Edition

[![en](https://img.shields.io/badge/lang-English-blue)](README.en.md)

Modern GUI JVM optimization wrapper for STALCRAFT built with Tauri.

## Features

- **🎨 Modern GUI** - Beautiful, responsive interface with dark theme
- **🔍 System Detection** - Automatically detects RAM, CPU cores, and large page support
- **⚡ Dynamic JVM Flags** - Generates optimal JVM flags based on your hardware
- **🚀 Process Boosting** - Enhances game process priority (CPU, memory, I/O)
- **🔧 IFEO Management** - Easy install/uninstall/status checking via GUI
- **🎮 Direct Launch** - Launch game with optimized settings directly from the app

## What It Does

1. **Detects** system hardware: Total/free RAM, CPU cores, large page support
2. **Generates** optimal JVM flags: Heap size, GC threads, G1 region size, metaspace, code cache
3. **Replaces** standard launcher JVM arguments with optimized ones
4. **Boosts** game process: `HIGH_PRIORITY_CLASS`, memory priority, I/O priority
5. **Installs** transparently via Windows IFEO - game files remain untouched

## Quick Start

### Prerequisites

- Windows 10/11
- Node.js 18+ and npm
- Rust 1.70+ (for building)
- STALCRAFT installed

### Installation

```bash
# Navigate to project directory
cd stalcraft-jvm-wrapper

# Install dependencies
npm install

# Run in development mode
npm run dev

# Build for production
npm run build
```

### Using the Application

1. **Launch** the application
2. **Click** "Refresh System Info" to detect your hardware
3. **Choose** one of two approaches:

   **A. IFEO Hook (Recommended)**
   - Click "Install" to register the IFEO hook (requires admin)
   - Launch game normally through STALCRAFT launcher
   - Wrapper automatically intercepts and optimizes

   **B. Direct Launch**
   - Enter game executable path
   - Click "Launch Game"
   - Game launches immediately with optimized flags

4. **Check Status** - Verify IFEO installation status anytime
5. **Uninstall** - Remove IFEO hook cleanly

## IFEO Commands (Terminal)

You can also manage IFEO from terminal (requires admin):

```bash
# Through the GUI - use the Install/Uninstall/Status buttons

# Or via Tauri CLI
npm run tauri -- run --args --install
npm run tauri -- run --args --status
npm run tauri -- run --args --uninstall
```

## How It Works

### Windows IFEO (Image File Execution Options)

Windows IFEO intercepts the launch of `stalcraft.exe` / `stalcraftw.exe` and redirects it through the wrapper. The wrapper:

1. Detects hardware via `GlobalMemoryStatusEx`, `runtime::available_parallelism()`
2. Calculates optimal JVM flags based on available resources
3. Removes conflicting flags from launcher arguments
4. Launches real `java.exe` / `javaw.exe` with optimized flags and `HIGH_PRIORITY_CLASS`
5. Applies post-boost: disables priority reduction, sets maximum memory and I/O priority

### Dynamic Calculation

| Parameter | Formula |
|-----------|---------|
| Heap | 50% free RAM, floor 25% of total, cap min(16GB, 75% of total) |
| ParallelGCThreads | cores - 2, minimum 2 |
| ConcGCThreads | parallel / 4, minimum 1 |
| G1HeapRegionSize | 4MB / 8MB / 16MB / 32MB depending on heap |
| Metaspace | 128MB / 256MB / 512MB depending on heap |
| CodeCache | heap/16, within 128-512MB |
| SurvivorRatio | 32 (≤4 cores) or 8 (>4 cores) |
| Large Pages | Enabled only with `SeLockMemoryPrivilege` |

### Supported Targets

- `stalcraft.exe` (main launcher) → `java.exe`
- `stalcraftw.exe` (Steam) → `javaw.exe`

## Large Pages (Optional)

For better performance, enable large pages:

1. Run `secpol.msc`
2. Local Policies → User Rights Assignment → Lock pages in memory
3. Add your user, reboot

The wrapper will auto-detect this and add `-XX:+UseLargePages`.

## Project Structure

```
stalcraft-jvm-wrapper/
├── src/                      # Frontend
│   ├── index.html           # Main HTML
│   ├── styles.css           # Styling
│   └── main.js              # Frontend logic
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri app entry point
│   │   ├── commands.rs     # Tauri commands
│   │   ├── system.rs       # System detection
│   │   ├── jvm.rs          # JVM flag generation
│   │   ├── ifeo.rs         # IFEO registry management
│   │   └── process.rs      # Process launching & boosting
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── package.json            # Node.js dependencies
└── README.md               # This file
```

## Building from Source

```bash
# Development build with hot reload
npm run dev

# Production build
npm run build

# Direct Rust build
cargo build --manifest-path src-tauri/Cargo.toml --release
```

The built executable will be in `src-tauri/target/release/`

## Architecture

### Frontend (Vanilla JS + CSS)
- Clean, modern UI with responsive design
- Real-time system information display
- One-click IFEO management
- Direct game launch interface

### Backend (Rust + Tauri)
- **system.rs**: Hardware detection via Windows API
- **jvm.rs**: Intelligent JVM flag generation
- **ifeo.rs**: Windows registry management
- **process.rs**: Process launching and priority boosting
- **commands.rs**: Tauri IPC command handlers

## Migration from Go Version

This Tauri version provides:
- ✅ Same core functionality as the Go version
- ✅ Modern GUI instead of CLI menu
- ✅ Easier to use for non-technical users
- ✅ Real-time system monitoring
- ✅ Direct launch option (no IFEO required)
- ✅ Cross-platform potential (currently Windows-only)

## License

MIT

## Credits

Rewritten from the original Go version to Tauri for a better user experience.
