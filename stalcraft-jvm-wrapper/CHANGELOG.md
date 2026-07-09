# Changelog

## v1.6.2 (2026-07-09)

- **Launch fix**: restore `game_dir_from_target` workdir fallback for EXBO runtime (reverts mistaken Go-only `java\bin` cwd from v1.6.1)
- **Presets**: `#[serde(default)]` on v1.0.8 fields so weak/medium/max import works

## v1.6.1 (2026-07-09)

- **Launch fix (EXBO parity)**: `resolve_work_dir` fallback matches upstream `process.go` — use exe parent dir, not inferred `runtime\stalcraft` root (fixes `exit_code=1` / critical client error)
- **IFEO**: launcher shims (`stalzone` / `stalcraftw` / …) inject regardless of install path; `java/javaw` stay game-scoped
- **Service**: unified `launch()` flow like EXBO `cmd/service/main.go`

## v1.6.0 (2026-07-09)

- **JVM**: restore EXBO parity — game-scoped injection for launchers (`stalzone` / `stalcraftw` / …) plus game `java.exe` / `javaw.exe`; system Java stays passthrough
- **Steam / multi-launcher**: classify Steam, EGS, VK paths; launch guide with per-platform `jvm_wrapper` paths
- **Logging**: `service_invoked` includes `scope`, `target_kind`; CLI `--probe-path` for offline diagnostics
- **Presets**: EXBO v1.0.8 `weak` / `medium` / `max` + `presets.manifest.json`; one-click import+apply in UI

## v1.5.8 (2026-07-07)

- **JVM hotfix**: inject flags only for game java.exe / javaw.exe; launcher exes run passthrough to prevent critical startup crash

## v1.5.7 (2026-07-07)

- **JVM**: java\\bin\\stalzone.exe now runs in passthrough mode to avoid startup crash after wrapper replacement
- **JVM**: flag injection kept for game java.exe / javaw.exe targets only

## v1.5.6 (2026-07-07)

- **Server Blocker**: raise hide/auto-block ping threshold from 100 ms to 200 ms

## v1.5.5 (2026-07-07)

- **Server Blocker**: removed allowlist mode — blocklist only
- **Server Blocker**: removed «All» region filter — RU / EU / NA / SEA with one region shown at a time
- Settings migration v5: legacy `allowlist` and `region: ALL` → `blocklist` + `RU`

## v1.5.4 (2026-07-07)

- **Server Blocker**: first-visit warning (firewall vs WinDivert, ExitLag/GearUP conflict, planned MITM-backend)
- **Server Blocker**: hide servers with ping >100 ms or not found; auto-block bad hosts after ping
- **Server Blocker**: progress bar + waveform in topbar during ping / firewall apply
- JVM presets from EXBO release history (`examples/`: balanced_mid, slow_ddr, throughput_v110, x3d_v110, 8khz, removed_fast_ddr)
- UI: one-click preset import; `list_examples` / `import_example_config` IPC
- Docs: `docs/server-blocker-architecture-ru.md` (MITM-backend plan per SilentBless)

## v1.5.3 (2026-07-07)

- JVM presets from [EXBO stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) release history in `examples/`
- Presets: `balanced_mid`, `slow_ddr`, `throughput_v110`, `x3d_v110`, `8khz`, `removed_fast_ddr`
- UI: one-click import into `configs/`; `list_examples` / `import_example_config` IPC

## v1.5.2 (2026-07-06)

- **Server Blocker GA**: Windows Firewall blocking on ports 29450–29460 (`STALZONE-SB-*` rules)
- 77 servers (RU live catalog + EU/NA/SEA), RU sub-zones by pool, TCP ping with colored latency
- Auto-best per pool; blocklist and allowlist modes (allowlist + auto-best fix)
- UAC elevation for firewall apply/clear (CLI `--sb-apply` / `--sb-clear`)
- Batch PowerShell firewall scripts; ping concurrency cap (10 threads)
- Welcome modals shown once (`stalcraft-jvm-welcome-v1`); removed nav «Скоро» badge
- Tests: expanded Rust unit tests, JS logic tests, UI smoke with preview lifecycle

## v1.5.1 (2026-07-06)

- Parity with [EXBO stalcraft-jvm-optimization v1.1.2](https://github.com/EXBO-Community/stalcraft-jvm-optimization/releases/tag/v1.1.2)
- IFEO: `stalzone.exe` / `stalzonew.exe` canonical, `stalcraft.exe` fallback
- Config fallback logging when active profile file is missing
- Work dir canonicalization for spawned game process

## v1.5.0 (2026-07-01)

- Two-step startup: license acceptance → authors & contacts
- Author contacts: GitHub, Discord, Telegram
- In-game support nickname **DementiyRezak**
- Custom yellow-orange scrollbars (UI + log)
- License text: non-commercial, trusted sources only, no liability for third parties
- Modals shown on every launch (no skip)
- Fluid UI scaling for dense layout (IFEO panel, hardware, log) via `clamp()`

## v1.4.0 (2026-07-01)

- Rebrand UI to **STALZONE** (titlebar, header, footer, i18n)
- License agreement modal (Nyrokume, SilentBless; non-commercial terms)
- Improved hardware detection (CPUID brand, WMI, multi-view registry)
- HWID spoofing hint on detection errors
- Custom layered app icon
- README screenshots in `docs/screenshots/`

## v1.3.0 (2026-07-01)

- Separate `service.exe` IFEO debugger (EXBO parity)
- IFEO targets: `stalcraft.exe`, `stalcraftw.exe`, `stalzone.exe`, `stalzonew.exe`, game-scoped `java.exe`/`javaw.exe`
- UAC elevation on Install/Uninstall from GUI
- Bilingual UI (RU/EN) with titlebar language switcher
- Hardware panel: robust parsing, early fetch during splash
- Registry write fix for x64 native view
- Release zip via `npm run build:prod`

## v1.2.x

- Initial Tauri port of [stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization)
- `NtCreateUserProcess` + JVM flag injection
- GUI config profiles and IFEO management
