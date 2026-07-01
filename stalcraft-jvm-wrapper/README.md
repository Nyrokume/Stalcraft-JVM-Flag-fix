# Stalcraft JVM Wrapper

Tauri/Rust port of [EXBO-Community/stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization).

## Layout

```
%AppData%\Roaming\EXBO\jvm_wrapper\
  stalcraft-jvm-wrapper.exe   ← GUI / installer
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
2. Run `npm run build:prod` or download release `wrapper.zip`.
3. Extract to `%AppData%\Roaming\EXBO\jvm_wrapper\`.
4. Launch GUI → **INSTALL** (UAC) → **VERIFY**.
5. Fully restart EXBO launcher → play.
6. `logs\wrapper.log` → `service_invoked` → `jvm_mode=INJECTED`.

**Do not run `service.exe` manually.**

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
