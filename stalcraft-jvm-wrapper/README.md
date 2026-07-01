# STALART JVM Wrapper

JVM tuning wrapper for **STALART** / **STALCRAFT** on Windows (Tauri + Rust).

## Features

- Hardware detection: RAM, CPU threads, L3 cache, SMBIOS memory speed tier, large pages
- Auto-generated `configs/default.json` (Go-compatible schema)
- IFEO install/uninstall/status via GUI or CLI flags
- Single `wrapper.exe`: GUI + silent debugger service mode
- Structured log at `logs/wrapper.log` (paths redacted, 2 MB rotation)

## IFEO targets

Hooks client executables only:

- `stalart.exe`, `stalartw.exe`
- `stalcraft.exe`, `stalcraftw.exe`

Registry active profile: `HKCU\Software\StalcraftWrapper\ActiveConfig` (migrates from legacy `StalartJvmWrapper`).

## Quick start

1. Add launcher/game folders to antivirus exclusions.
2. Place `jvm_wrapper/` in the **launcher root** (next to the main launcher exe).
3. Run `wrapper.exe` → **Install** (admin) → launch the game normally.

### CLI

```text
wrapper.exe --install
wrapper.exe --uninstall
wrapper.exe --status
```

## Config profiles

- JSON files in `configs/` next to the exe
- **Regen** rebuilds `default.json` from detected hardware
- **Apply** sets the active profile in registry
- Optional high-end profile: copy [`examples/8khz.json`](examples/8khz.json) to `configs/`

## Build

```bash
npm install
npm run tauri build
```

## Requirements

- Windows 10/11, admin for IFEO install
- 8+ GB RAM (12+ GB recommended for PreTouch)
