# Stalcraft JVM Wrapper

Порт [stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization).

## Файлы

```
%AppData%\Roaming\EXBO\jvm_wrapper\
  stalcraft-jvm-wrapper.exe   ← GUI, установка IFEO
  service.exe                 ← перехватчик (обязателен!)
  configs\default.json
  logs\wrapper.log
```

## IFEO

| Образ | Поведение |
|-------|-----------|
| `stalcraft.exe`, `stalcraftw.exe`, `stalzone.exe`, `stalzonew.exe` | Всегда инжект JVM |
| `java.exe`, `javaw.exe` | Только из папок игры (`\runtime\stalcraft\`, `\exbo\`) |

Реестр: 64-bit + WOW6432Node.

## Быстрый старт

1. Исключения антивируса для папки `EXBO`.
2. Соберите `npm run build:prod` или скачайте `wrapper.zip` из Releases.
3. Распакуйте в `%AppData%\Roaming\EXBO\jvm_wrapper\` (рядом с `ExboLink.exe`).
4. GUI → **УСТАНОВИТЬ** (UAC) → **ПРОВЕРКА**.
5. Полностью закройте лаунчер → запустите игру.
6. В логе: `service_invoked` → `jvm_mode=INJECTED`.

**Не запускайте `service.exe` вручную.**

## Интерфейс

Переключатель **RU / EN** в titlebar слева от кнопок окна.

## Сборка

```powershell
npm install
npm run build:prod
```

Создаёт `wrapper.zip`.

## Конфигурация

- `default.json` — под железо при Install
- **Сброс** / **Применить** в GUI
- Активный профиль: `HKCU\Software\StalcraftWrapper\ActiveConfig`
