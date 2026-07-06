# STALZONE JVM Wrapper

Tauri/Rust port of [EXBO-Community/stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) for **STALZONE** (EXBO launcher).

**Latest release:** [v1.5.1](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases/latest) · `wrapper.zip`

## Screenshots

| Main UI | License (step 1) | Authors & contacts (step 2) |
|---|---|---|
| ![Main UI](docs/screenshots/main-ru.png) | ![License](docs/screenshots/license-ru.png) | ![Info](docs/screenshots/info-ru.png) |

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
| `stalzone.exe`, `stalzonew.exe`, `stalcraft.exe`, `stalcraftw.exe` | Always inject JVM flags |
| `java.exe`, `javaw.exe` | Inject only under `\runtime\stalcraft\` or `\exbo\` |

Registry: native 64-bit + WOW6432Node.

## Quick start

1. Add `%AppData%\Roaming\EXBO` to antivirus exclusions.
2. Download **`wrapper.zip`** from [Releases](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases) (recommended) or build locally: `npm run build:prod`.
3. Extract to `%AppData%\Roaming\EXBO\jvm_wrapper\` (both `.exe` files side by side).
4. Launch GUI:
   - **Step 1:** accept license agreement
   - **Step 2:** authors & contacts screen → **Got it**
5. Click **INSTALL** (UAC) → **VERIFY** (all targets `ok`).
6. Fully restart EXBO launcher → play STALZONE.
7. Check `logs\wrapper.log` → `service_invoked` → `jvm_mode=INJECTED`.

**Do not run `service.exe` manually.**

## Features (v1.5.1)

- [EXBO v1.1.2](https://github.com/EXBO-Community/stalcraft-jvm-optimization/releases/tag/v1.1.2) parity: `stalzone*` IFEO targets first
- Typed launcher paths (`paths.rs`), canonical `EXBO\jvm_wrapper` home
- Two-step startup: license + author info (every launch)
- Hardware detection: CPUID, WMI, registry (multi-view)
- IFEO via `service.exe` (EXBO parity), UAC install from GUI
- Custom app icon, orange scrollbars
- Authors: Nyrokume, SilentBless — GitHub / Discord / Telegram in app

## Build from source

```powershell
npm install
npm run build:prod
```

Creates `wrapper.zip` with `stalcraft-jvm-wrapper.exe`, `service.exe`, `examples/`.

```powershell
npm run screenshots   # optional: refresh docs/screenshots/
```

## CLI

```text
stalcraft-jvm-wrapper.exe --install
stalcraft-jvm-wrapper.exe --uninstall
stalcraft-jvm-wrapper.exe --status
```

## Authors & support

| Channel | Contact |
|---------|---------|
| GitHub | [Nyrokume](https://github.com/Nyrokume) |
| Discord | `@nyrokume` |
| Telegram | [@nyrokume](https://t.me/nyrokume) |
| In-game | **DementiyRezak** |

See [README_RU.md](./README_RU.md) for Russian.

**License:** free, non-commercial. Install only from trusted sources.
