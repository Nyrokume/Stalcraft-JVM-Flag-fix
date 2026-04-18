
# STALCRAFT JVM Wrapper - Tauri Edition
<img width="1918" height="854" alt="image" src="https://github.com/user-attachments/assets/681a65ce-e8c9-4836-908a-755e009e903e" />


> [!WARNING]
> This project is an **unofficial** utility developed by [nyrok](https://github.com/nyrokume).
> It is **not affiliated with EXBO**, but was verified and classified as safe software.

> [!CAUTION]
> If you encounter issues after installing — check the [TROUBLESHOOTING](./docs/TROUBLESHOOTING.md) document for your situation.
> Please **do not bother EXBO moderators** or game technical support. All instructions are already in the documentation.

Modern GUI JVM optimization wrapper for STALCRAFT built with Tauri, ported from the original [Go version](https://github.com/EXBO-Community/stalcraft-jvm-optimization).

## Features

- **🎨 Modern GUI** - Beautiful, responsive interface with dark theme
- **🔍 System Detection** - Automatically detects RAM, CPU cores, L3 cache, large page support
- **⚡ Dynamic JVM Flags** - Generates optimal JVM flags based on your hardware (X3D support)
- **🚀 Process Boosting** - Enhances game process priority via NtCreateUserProcess
- **🔧 IFEO Management** - Easy install/uninstall/status checking via GUI
- **📁 Config profiles & presets** - JSON profiles next to the exe, eight one-click presets layered on hardware-tuned defaults, plus an in-app JSON editor (`load` / `save`)
- **🎮 Single EXE** - Works as both GUI and IFEO debugger (no separate service.exe needed)

## What It Does

1. **Detects** system hardware: Total/free RAM, CPU cores/threads, L3 cache, large page support
2. **Generates** optimal JVM flags: Heap size, GC threads, G1 region size, metaspace, JIT settings
3. **Filters** conflicting launcher flags and injects optimized ones
4. **Boosts** game process: Memory priority, I/O priority, disables priority decay
5. **Installs** transparently via Windows IFEO - game files remain untouched

## Requirements

- **OS:** Windows 10/11
- **Game version:** Steam/Launcher/EGS/VK Play
- **Rights:** Administrator rights (only for IFEO install/uninstall)
- **CPU:** 4+ cores
- **RAM:** 8+ GB (12+ GB recommended for full optimizations)

## Quick Start

### Installation

1. Add game folder to Windows Defender exclusions:
   - Steam: `C:\Program Files\Steam\steamapps\common\STALCRAFT`
   - Launcher: `C:\Users\User\AppData\Roaming\EXBO`
   - EGS: `C:\Games\EGS Stalcraft\STALCRAFT`

2. Create `jvm_wrapper` folder in EXBO launcher root (next to `ExboLink.exe` and `runtime/`)

3. Download latest release and extract to `jvm_wrapper` folder

4. Run `wrapper.exe`, click **Install** in IFEO Registry section

5. **Now launch the game normally!**

> [!TIP]
> The most common installation error is placing `jvm_wrapper` inside `runtime/stalcraft/...`. 
> The folder should be in the **root** of the EXBO directory, next to `ExboLink.exe`.

### Usage

1. **Launch** the application (administrator for IFEO install)
2. **Refresh** hardware detection; optionally pick a **preset** or a **saved profile** and **Apply**
3. **Install** the IFEO hook
4. **Launch** the game through your normal launcher (EXBO / Steam / etc.)

The wrapper intercepts the game process and applies the active `configs/*.json` profile.

## Technical Details

### IFEO (Image File Execution Options)

The wrapper uses Windows IFEO to intercept game launch:
- `stalcraft.exe` → wrapper.exe with game args
- `stalcraftw.exe` → wrapper.exe with game args

### Process Creation

The wrapper uses `ntdll!NtCreateUserProcess` with:
- `PS_ATTRIBUTE_IFEO_SKIP_DEBUGGER` - prevents re-intercept through IFEO
- `RTL_USER_PROC_PARAMS_NORMALIZED` - normalized process parameters

### Process Boosting

After game starts, wrapper applies:
- `SetProcessPriorityBoost(handle, 1)` - disables priority decay
- `NtSetInformationProcess(handle, PROCESS_MEMORY_PRIORITY, 5)` - High memory priority
- `NtSetInformationProcess(handle, PROCESS_IO_PRIORITY, 3)` - High I/O priority
- Exits when game window becomes visible

### Hardware Detection

Uses Windows API:
- `GlobalMemoryStatusEx` - Total/free RAM
- `GetLargePageMinimum` - Large page support
- `GetLogicalProcessorInformationEx` - CPU cores, L3 cache
- `OpenProcessToken`/`LookupPrivilegeValueW`/`PrivilegeCheck` - SeLockMemoryPrivilege check

### Dynamic Calculation

| Parameter | Formula | Min | Max |
|-----------|---------|-----|-----|
| Heap | >=24GB: 8GB, >=16GB: 6GB, >=12GB: 5GB, >=8GB: 4GB, >=6GB: 3GB, default: 2GB | 2GB | 8GB |
| ParallelGCThreads | clamp(threads-2, 2, 10) | 2 | 10 |
| ConcGCThreads | clamp(parallel/2, 1, 5) | 1 | 5 |
| G1HeapRegionSize | <=3GB: 4MB, <=5GB: 8MB, >5GB: 16MB | 4MB | 16MB |
| Metaspace | 512MB fixed | 512MB | 512MB |
| PreTouch | enabled if RAM >= 12GB | - | - |
| Large Pages | enabled only with SeLockMemoryPrivilege | - | - |

### X3D (Big Cache) Detection

If L3 cache >= 64MB (X3D class CPUs):
- `max_inline_level` = 20 (vs 15)
- `freq_inline_size` = 750 (vs 500)
- `inline_small_code` = 6000 (vs 4000)
- `max_node_limit` = 320000 (vs 240000)
- `InitiatingHeapOccupancyPercent` = 15 (vs 20)
- `MaxGCPauseMillis` = 25 (vs 50)
- Extra concurrent GC thread if threads >= 16

## JVM Flags Generated

All flags from the original Go version are fully implemented:

### Memory
- `-Xmx{X}g -Xms{Y}g` (Xms = min(heap, 4GB))
- `-XX:MetaspaceSize={N}m -XX:MaxMetaspaceSize={N}m`
- `-XX:+AlwaysPreTouch` (if RAM >= 12GB)
- `-XX:+UseLargePages -XX:LargePageSizeInBytes={N}m`

### G1 GC
- `-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions`
- `-XX:MaxGCPauseMillis={N}`
- `-XX:G1HeapRegionSize={N}m`
- `-XX:G1NewSizePercent=23 (30 X3D)`
- `-XX:G1MaxNewSizePercent=50`
- `-XX:G1ReservePercent=20`
- `-XX:G1HeapWastePercent=5`
- `-XX:G1MixedGCCountTarget=3 (4 X3D)`
- `-XX:+G1UseAdaptiveIHOP`
- `-XX:InitiatingHeapOccupancyPercent=20 (15 X3D)`
- `-XX:G1MixedGCLiveThresholdPercent=90`
- `-XX:G1RSetUpdatingPauseTimePercent=0`
- `-XX:SurvivorRatio=32 -XX:MaxTenuringThreshold=1`
- `-XX:ParallelGCThreads={N} -XX:ConcGCThreads={N}`
- `-XX:+ParallelRefProcEnabled`
- `-XX:+UseDynamicNumberOfGCThreads`
- `-XX:+UseStringDeduplication`

### GC Advanced
- `-XX:G1SATBBufferEnqueueingThresholdPercent=30`
- `-XX:G1ConcRSHotCardLimit=16`
- `-XX:G1ConcRefinementServiceIntervalMillis=150`
- `-XX:GCTimeRatio=99`

### JIT
- `-XX:ReservedCodeCacheSize=400m`
- `-XX:MaxInlineLevel=15 (20 X3D)`
- `-XX:FreqInlineSize=500 (750 X3D)`
- `-XX:InlineSmallCode=4000 (6000 X3D)`
- `-XX:MaxNodeLimit=240000 (320000 X3D)`
- `-XX:NodeLimitFudgeFactor=8000`
- `-XX:NmethodSweepActivity=1`
- `-XX:-DontCompileHugeMethods`
- `-XX:AllocatePrefetchStyle=3`
- `-XX:+AlwaysActAsServerClassMachine`
- `-XX:+UseXMMForArrayCopy`
- `-XX:+UseFPUForSpilling`

### Java 9 Specific
- `-Dsun.reflect.inflationThreshold=0`
- `-XX:AutoBoxCacheMax=4096`
- `-XX:+UseThreadPriorities -XX:ThreadPriorityPolicy=1`
- `-XX:-UseCounterDecay`
- `-XX:CompileThresholdScaling=0.5`

### Other
- `-XX:-UseBiasedLocking`
- `-XX:+DisableAttachMechanism`
- `-Djdk.nio.maxCachedBufferSize=131072`

## Config Management

After first run, the app creates a `configs` folder next to `wrapper.exe` and ensures `configs/default.json` exists (hardware-tuned baseline). The **active profile name** is stored in the registry value `HKCU\Software\StalcraftWrapper\ActiveConfig` (string). IFEO / debugger mode loads that profile when launching the game.

### Saved profiles (`configs/*.json`)

- **Apply** in the GUI sets `ActiveConfig` to the selected stem (filename without `.json`).
- **Regen** rebuilds `default.json` from current hardware detection and switches the active profile to `default`.
- You can add or edit JSON files manually in `configs/`; they appear in the **Saved profiles** list after refresh.

### Presets (GUI)

Presets start from the same auto-tuned `generate()` profile as `default`, then apply a small, fixed adjustment and write **one file per preset**, always the same paths:

| Preset | File | Intent |
|--------|------|--------|
| Balanced | `preset_balanced.json` | Same tuning as auto `generate()` (baseline snapshot) |
| Latency | `preset_latency.json` | Tighter GC pause target, earlier mixed collection behaviour |
| Throughput | `preset_throughput.json` | Softer pause budget, slightly higher IHOP |
| Nursery | `preset_nursery.json` | Larger young-gen tilt, slightly lower pause target |
| Conservative | `preset_conservative.json` | Disables large pages and pre-touch, gentler JIT limits |
| Low RAM | `preset_low_ram.json` | Heap reduced by 1 GB vs auto (minimum 2 GB), region size recalculated |
| Streaming | `preset_streaming.json` | Lower heap, no pre-touch, softer GC and larger metaspace for capture/alt-tab |
| Power | `preset_power.json` | Extra parallel/concurrent GC threads (capped), larger code cache |

Applying a preset saves the file, sets it active, and updates the on-screen heap summary (same as choosing a profile and clicking Apply).

### Profile JSON editor (GUI)

The **Configuration** panel includes a textarea backed by `get_active_config`, `load_config_by_name`, and `save_config`: load the active profile or any selected file, edit the JSON (field names match the Rust `Config` struct), then **Save to selected** overwrites that profile on disk. Use **Apply** on the saved profile list to switch the active registry pointer; IFEO launches always read the active profile.

## Logging

The wrapper logs to `logs/wrapper.log` next to the executable:
- System detection
- Config loading
- Process start/exit
- Exit codes

Raw launcher arguments and JVM flags are **not logged** for security.

## Large Pages (Optional)

For better performance with large heaps:

1. Press `Win` + `R`, run `secpol.msc`
2. Local Policies → User Rights Assignment → Lock pages in memory
3. Add your user account
4. **Reboot** (policy applies at logon)

The wrapper auto-detects this and enables `-XX:+UseLargePages`.

## Project Structure

```
stalcraft-jvm-wrapper/
├── src/                      # Frontend (Vanilla JS)
│   ├── index.html           # Main HTML
│   ├── main.js              # Frontend logic
│   └── assets/styles.css    # Styling
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point (GUI + debugger mode)
│   │   ├── commands.rs     # Tauri IPC commands
│   │   ├── system.rs       # Hardware detection
│   │   ├── config.rs       # Config generation & persistence
│   │   ├── jvm.rs          # JVM flag generation & filtering
│   │   ├── ifeo.rs         # IFEO registry management
│   │   └── process.rs      # NtCreateUserProcess & boosting
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── docs/                    # Documentation
│   ├── PARAMS.md           # JVM parameter explanations
│   ├── OVERVIEW.md         # Technical overview
│   └── TROUBLESHOOTING.md  # Common issues
├── package.json            # Node.js dependencies
└── README.md               # This file
```

## Building from Source

```bash
# Install dependencies
npm install

# Development build
npm run tauri dev

# Production build
npm run tauri build
```

## CLI Mode

The exe also supports command-line arguments:

```bash
wrapper.exe --install      # Install IFEO hook
wrapper.exe --uninstall    # Remove IFEO hook
wrapper.exe --status       # Check IFEO status
```

## Comparison with Go Version

| Feature | Go Original | Tauri Port |
|---------|-------------|------------|
| UI | CLI menu | Modern GUI |
| System Detection | sysinfo.go | system.rs (identical) |
| Config Generation | config/generate.go | config.rs (identical) |
| JVM Flags | jvm/flags.go | jvm.rs (identical) |
| Filter | jvm/filter.go | jvm.rs (identical) |
| IFEO | installer/installer.go | ifeo.rs (identical) |
| Process | process/process.go | process.rs (identical) |
| Binary | cli.exe + service.exe | Single exe |

## Credits

- Original Go version: [EXBO-Community/stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization)
- Tauri port: [nyrok](https://github.com/nyrokume)
- Based on work by SilentBless

## License

MIT
