# STALZONE JVM Wrapper

Tauri/Rust port of [EXBO-Community/stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) for **STALZONE** (EXBO launcher).

## Screenshots

| Main UI (RU) | License agreement |
|---|---|
| ![Main UI](docs/screenshots/main-ru.png) | ![License](docs/screenshots/license-ru.png) |

## Layout

```
%AppData%\Roaming\EXBO\jvm_wrapper\
  stalcraft-jvm-wrapper.exe   ← GUI / installer (filename unchanged)
  service.exe                 ← IFEO debugger (required)
  configs\default.json
  logs\wrapper.log
  examples\
```

## IFEO targets

| Image | Behavior |
|-------|----------|
| `stalcraft.exe`, `stalcraftw.exe`, `stalzone.exe`, `stalzonew.exe` | Always inject JVM flags |
| `java.exe`, `javaw.exe` | Inject only under `\runtime\stalcraft\` or `\exbo\` |

Registry: native 64-bit + WOW6432Node.

## Quick start

1. Add `%AppData%\Roaming\EXBO` to antivirus exclusions.
2. Download `wrapper.zip` from [Releases](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases) or run `npm run build:prod`.
3. Extract to `%AppData%\Roaming\EXBO\jvm_wrapper\`.
4. Launch GUI → accept license → **INSTALL** (UAC) → **VERIFY**.
5. Fully restart EXBO launcher → play STALZONE.
6. `logs\wrapper.log` → `service_invoked` → `jvm_mode=INJECTED`.

**Do not run `service.exe` manually.**

## Features (v1.4.0)

- STALZONE branding, RU/EN UI
- Hardware detection (CPUID, WMI, registry)
- License agreement on startup (non-commercial)
- Custom app icon, `service.exe` IFEO parity with EXBO

## Build

```powershell
npm install
npm run build:prod
```

Output: `wrapper.zip` with both executables and `examples/`.

## CLI

```text
stalcraft-jvm-wrapper.exe --install
stalcraft-jvm-wrapper.exe --uninstall
stalcraft-jvm-wrapper.exe --status
```

See [README_RU.md](./README_RU.md) for Russian.

**Authors:** Nyrokume, SilentBless · [Nyrokume.dev](https://nyrokume.dev)
