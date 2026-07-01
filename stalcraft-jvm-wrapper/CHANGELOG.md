# Changelog

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
