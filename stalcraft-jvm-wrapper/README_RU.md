# STALZONE JVM Wrapper

Порт [stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) для **STALZONE** (лаунчер EXBO).

## Скриншоты

| Главное окно (RU) | Лицензия |
|---|---|
| ![Интерфейс](docs/screenshots/main-ru.png) | ![Лицензия](docs/screenshots/license-ru.png) |

## Файлы

```
%AppData%\Roaming\EXBO\jvm_wrapper\
  stalcraft-jvm-wrapper.exe   ← GUI (имя файла без изменений)
  service.exe                 ← перехватчик IFEO
  configs\default.json
  logs\wrapper.log
```

## IFEO

| Образ | Поведение |
|-------|-----------|
| `stalcraft.exe`, `stalcraftw.exe`, `stalzone.exe`, `stalzonew.exe` | Всегда инжект JVM |
| `java.exe`, `javaw.exe` | Только из папок игры (`\runtime\stalcraft\`, `\exbo\`) |

## Быстрый старт

1. Исключения антивируса для папки `EXBO`.
2. Скачайте `wrapper.zip` из [Releases](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases) или соберите `npm run build:prod`.
3. Распакуйте в `%AppData%\Roaming\EXBO\jvm_wrapper\`.
4. GUI → примите лицензию → **УСТАНОВИТЬ** (UAC) → **ПРОВЕРКА**.
5. Полностью закройте лаунчер → запустите STALZONE.
6. В логе: `service_invoked` → `jvm_mode=INJECTED`.

**Не запускайте `service.exe` вручную.**

## v1.4.0

- Брендинг STALZONE, интерфейс RU/EN
- Определение железа (CPUID, WMI, реестр)
- Лицензионное соглашение (некоммерческое ПО)
- Иконка приложения, IFEO через `service.exe`

## Сборка

```powershell
npm install
npm run build:prod
```

## Интерфейс

Переключатель **RU / EN** в titlebar. Активный профиль: `HKCU\Software\StalcraftWrapper\ActiveConfig`

**Авторы:** Nyrokume, SilentBless · [Nyrokume.dev](https://nyrokume.dev)
