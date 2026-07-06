# STALZONE JVM Wrapper

Порт [stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) для **STALZONE** (лаунчер EXBO).

**Актуальный релиз:** [v1.5.4](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases/latest) · `wrapper.zip`

## Скриншоты

| Главное окно | Лицензия (шаг 1) | Авторы и контакты (шаг 2) |
|---|---|---|
| ![Интерфейс](docs/screenshots/main-ru.png) | ![Лицензия](docs/screenshots/license-ru.png) | ![Инфо](docs/screenshots/info-ru.png) |

## Файлы

```
%AppData%\Roaming\EXBO\jvm_wrapper\
  stalcraft-jvm-wrapper.exe   ← GUI (имя файла без изменений)
  service.exe                 ← перехватчик IFEO (обязателен)
  configs\default.json
  logs\wrapper.log
  examples\                   ← пресеты JVM
```

## IFEO

| Образ | Поведение |
|-------|-----------|
| `stalzone.exe`, `stalzonew.exe`, `stalcraft.exe`, `stalcraftw.exe` | Всегда инжект JVM |
| `java.exe`, `javaw.exe` | Только из папок игры (`\runtime\stalcraft\`, `\exbo\`) |

## Быстрый старт

1. Исключения антивируса для папки `EXBO`.
2. Скачайте **`wrapper.zip`** из [Releases](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases) или соберите: `npm run build:prod`.
3. Распакуйте в `%AppData%\Roaming\EXBO\jvm_wrapper\`.
4. Запустите GUI:
   - **Шаг 1:** примите лицензию
   - **Шаг 2:** экран авторов и контактов → **Понятно**
5. **УСТАНОВИТЬ** (UAC) → **ПРОВЕРКА**.
6. Полностью закройте лаунчер → запустите STALZONE.
7. В логе: `service_invoked` → `jvm_mode=INJECTED`.

**Не запускайте `service.exe` вручную.**

## Блокировка серверов (v1.5.4)

Вторая вкладка: пинг 77 туннелей, auto-best по регионам, блокировка лишних серверов.

| Тема | Детали |
|------|--------|
| **Сейчас** | Windows Firewall, исходящие правила на портах **29450–29460** (нужен UAC) |
| **Не используется** | WinDivert на клиенте — конфликт с ExitLag/GearUP |
| **В планах** | Серверный MITM через `backend-*.stalzone` и API `/address_list` |
| **Первый вход** | Модальное предупреждение о firewall, бустерах и roadmap |
| **После пинга** | Серверы с пингом >100 мс и недоступные скрываются и блокируются |

Архитектура: [docs/server-blocker-architecture-ru.md](docs/server-blocker-architecture-ru.md)

## Пресеты JVM (EXBO)

В `examples/` внутри `wrapper.zip`: `balanced_mid`, `slow_ddr`, `throughput_v110`, `x3d_v110`, `8khz`, `removed_fast_ddr`.

Импорт через вкладку **КОНФИГУРАЦИЯ** (чипы) или копирование в `configs\`.

## v1.5.4

- Блокировка серверов GA + предупреждение при первом открытии
- Скрытие плохого пинга, прогресс-бар и волна в топбаре
- Пресеты JVM из истории релизов EXBO
- Паритет [EXBO v1.1.2](https://github.com/EXBO-Community/stalcraft-jvm-optimization/releases/tag/v1.1.2)
- IFEO через `service.exe`, определение железа, лицензия при первом запуске

## Сборка

```powershell
npm install
npm test
npm run build:prod
```

Результат: `wrapper.zip`.

## Авторы и поддержка

| Канал | Контакт |
|-------|---------|
| GitHub | [Nyrokume](https://github.com/Nyrokume) |
| Discord | `@nyrokume` |
| Telegram | [@nyrokume](https://t.me/nyrokume) |
| В игре | **DementiyRezak** |

Активный профиль: `HKCU\Software\StalcraftWrapper\ActiveConfig`

**Лицензия:** бесплатно, некоммерческое ПО. Устанавливайте только из проверенных источников.

**Авторы:** Nyrokume, SilentBless · [Nyrokume.dev](https://nyrokume.dev)
