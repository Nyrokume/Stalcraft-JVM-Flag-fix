# Changelog

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
