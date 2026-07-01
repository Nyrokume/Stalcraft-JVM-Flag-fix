# STALART JVM Wrapper

JVM-обёртка для **STALART** / **STALCRAFT** на Windows.

## Быстрый старт

1. Папка `jvm_wrapper` в **корне** лаунчера (рядом с основным `.exe`).
2. Запустите `wrapper.exe` **от администратора**.
3. **IFEO REGISTRY** → **INSTALL**.
4. Запускайте игру через лаунчер как обычно.

## IFEO

Перехват только клиентских exe: `stalart.exe`, `stalartw.exe`, `stalcraft.exe`, `stalcraftw.exe`.

Активный профиль: `HKCU\Software\StalcraftWrapper\ActiveConfig`.

## Конфигурация

- `configs/*.json` рядом с exe
- **Regen** — пересобрать `default.json` по железу
- **Apply** — выбрать активный профиль
- Пример high-end: [`examples/8khz.json`](examples/8khz.json) → скопировать в `configs/`

## CLI

```text
wrapper.exe --install
wrapper.exe --uninstall
wrapper.exe --status
```

## Логи

`logs/wrapper.log` — без сырых JVM-флагов и путей пользователя.

Подробнее: [README.md](./README.md) (English).
