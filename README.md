# STALZONE JVM Flag Fix

[![eng](https://img.shields.io/badge/lang-English-blue)](stalcraft-jvm-wrapper/README.md)
[![ru](https://img.shields.io/badge/lang-Russian-blue)](stalcraft-jvm-wrapper/README_RU.md)

> [!WARNING]
> Данный проект является **неофициальной** утилитой, разработанной [Nyrokume](https://github.com/Nyrokume) и [SilentBless](https://github.com/SilentBless).
> Утилита **не аффилирована с EXBO**. Это порт и доработка идеи [stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) под лаунчер **STALZONE** с графическим интерфейсом на Tauri.

> [!CAUTION]
> Если после установки что-то пошло не так — сначала откройте [документ по устранению неполадок](https://github.com/EXBO-Community/stalcraft-jvm-optimization/blob/master/docs/TROUBLESHOOTING.md) оригинального проекта EXBO. Большинство типичных ситуаций (антивирус, IFEO, память, Large Pages) описаны там пошагово.
>
> Пожалуйста, **не обращайтесь с этой утилитой к модераторам EXBO, в техподдержку игры и к рупорам**. Они не знают, что происходит на вашем ПК. Инструкции ниже и в [TROUBLESHOOTING](https://github.com/EXBO-Community/stalcraft-jvm-optimization/blob/master/docs/TROUBLESHOOTING.md) — достаточно для самостоятельного решения.

**Утилита для модификации параметров запуска JVM и оптимизации её работы в STALZONE (EXBO).**

**JVM (Java Virtual Machine)** — среда выполнения, через которую работает [STALCRAFT: X](https://stalcraft.ru/). Код игры исполняется не напрямую в ОС, а внутри виртуальной машины Java: во время работы JVM компилирует байткод в машинный код под ваш ПК (JIT). Это дополнительный слой между игрой и железом, отвечающий за выполнение кода и адаптацию под систему.

Данная программа позволяет изменить параметры запуска JVM для повышения производительности, используя предустановленные и пользовательские JSON-файлы конфигурации. Управление — через **графическое окно** (в отличие от консольного `cli.exe` в оригинале EXBO).

> [!IMPORTANT]
> Утилита подбирает параметры JVM под любой объём ОЗУ, начиная с **8 ГБ**.
> На системах с меньшим объёмом `default.json` генерируется с минимально безопасным heap, но стабильной работы игры это **не гарантирует** — лучше увеличить ОЗУ или использовать стандартные настройки лаунчера EXBO.

[![Downloads](https://img.shields.io/github/downloads/Nyrokume/Stalcraft-JVM-Flag-fix/total?label=Downloads&color=green)](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases)
[![Latest Release](https://img.shields.io/github/v/release/Nyrokume/Stalcraft-JVM-Flag-fix?label=Latest)](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases/latest)

**[Скачать v1.5.1](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases/latest)** — `wrapper.zip`

---

## Вносимые изменения

Утилита состоит из **двух бинарников** в одной папке `jvm_wrapper`:

| Файл | Назначение |
|------|------------|
| **`stalcraft-jvm-wrapper.exe`** | Графический интерфейс: установка, удаление, проверка IFEO, выбор профиля, просмотр железа и лога. Запускается вручную, когда нужно что-то настроить. |
| **`service.exe`** | Тихий перехватчик IFEO: Windows запускает его при старте игры. **Не запускайте вручную.** |

`service.exe` перехватывает запуск процессов игры **`stalzone.exe`** / **`stalzonew.exe`** (канонические имена после ребренда STALZONE), а также **`stalcraft.exe`** / **`stalcraftw.exe`** (fallback, пока процесс не переименован), и JVM игры:

| Образ | Поведение |
|-------|-----------|
| `stalzone.exe`, `stalzonew.exe`, `stalcraft.exe`, `stalcraftw.exe` | Всегда подставляет оптимальные JVM-флаги |
| `java.exe`, `javaw.exe` | Только если процесс из папок игры (`\runtime\stalcraft\`, `\exbo\`) |

При перехвате утилита:

- **Подбирает конфигурацию JVM** — heap, режим GC, JIT и связанные флаги под ваше железо.
- **Повышает приоритет** игрового процесса относительно фоновых задач.

Утилита устанавливается **один раз** и автоматически срабатывает при каждом запуске игры через лаунчер.

> [!IMPORTANT]
> Файлы игры **не изменяются** и не модифицируются.
> Утилита не встраивается в процесс игры — только подменяет аргументы запуска JVM через механизм IFEO (Image File Execution Options).

### Скриншоты

| Главное окно | Лицензия (шаг 1) | Авторы и контакты (шаг 2) |
|---|---|---|
| ![Интерфейс](stalcraft-jvm-wrapper/docs/screenshots/main-ru.png) | ![Лицензия](stalcraft-jvm-wrapper/docs/screenshots/license-ru.png) | ![Инфо](stalcraft-jvm-wrapper/docs/screenshots/info-ru.png) |

---

## Требования

- **ОС:** Windows 10/11 (x64)
- **Игра:** STALZONE / EXBO-лаунчер, Steam, EGS, VK Play
- **Права:** администратор Windows — только для **Установить** / **Удалить** (запись IFEO в `HKLM`)
- **ЦП:** 4+ ядер
- **ОЗУ:** 8+ ГБ, рекомендуется 12+ ГБ (иначе часть оптимизаций, например `PreTouch`, может остаться отключённой)

---

## Работа с утилитой

### Установка

> [!TIP]
> Самая частая ошибка — положить `jvm_wrapper` глубоко внутрь `runtime/stalcraft/...`. Папка должна лежать **в корне директории EXBO**, рядом с `ExboLink.exe` и каталогом `runtime/`:
>
> ```
> %AppData%\Roaming\EXBO\
>   ExboLink.exe
>   runtime\
>   jvm_wrapper\          ← сюда
>     stalcraft-jvm-wrapper.exe
>     service.exe
>     examples\
> ```

1. Добавьте папку с игрой в исключения Защитника Windows или другого антивируса:
   - Лаунчер EXBO / STALZONE: `%AppData%\Roaming\EXBO`
   - Steam: `C:\Program Files\Steam\steamapps\common\STALCRAFT`
   - EGS: путь к установке STALCRAFT
2. Создайте каталог `jvm_wrapper` в корне директории лаунчера (см. схему выше).
3. Скачайте [**последний релиз**](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases/latest) и распакуйте `wrapper.zip` в `jvm_wrapper`. Внутри должны быть `stalcraft-jvm-wrapper.exe`, `service.exe` и каталог `examples/`.
4. Запустите **`stalcraft-jvm-wrapper.exe`**:
   - **Шаг 1:** примите лицензионное соглашение (некоммерческое ПО, только доверенные источники).
   - **Шаг 2:** экран авторов и контактов → **Понятно**.
5. Нажмите **УСТАНОВИТЬ** — подтвердите UAC. Это нормально: IFEO пишется в реестр `HKLM`.
6. Нажмите **ПРОВЕРКА** — все цели должны быть `ok`.
7. **Полностью закройте** лаунчер EXBO и запустите STALZONE снова.

**Теперь можно играть.**

В логе `%AppData%\Roaming\EXBO\jvm_wrapper\logs\wrapper.log` при успехе появятся строки вроде `service_invoked` и `jvm_mode=INJECTED`.

> [!IMPORTANT]
> Особенности работы:
>
> - Аппаратный **G-Sync** может давать артефакты — рекомендуется отключить.
> - Утилита **не** распространяется на посторонние Java-приложения вне путей игры.
> - На системах с **8–16 ГБ** ОЗУ желательно включить файл подкачки.

### Удаление

1. Запустите `stalcraft-jvm-wrapper.exe` → **УДАЛИТЬ** (UAC).
2. Удалите папку `jvm_wrapper`.
3. Перезапустите лаунчер и игру, если они были открыты.

Или из командной строки:

```text
stalcraft-jvm-wrapper.exe --uninstall
```

### Конфигурация

После установки утилита создаёт профиль **`configs/default.json`**, адаптированный под параметры вашего ПК. С ним игра запускается по умолчанию.

**Активный профиль в реестре:** `HKCU\Software\StalcraftWrapper\ActiveConfig`

Сменить профиль в GUI:

1. Положите `.json` в `jvm_wrapper/configs/` (или скопируйте из `examples/`).
2. В окне утилиты выберите нужный профиль в списке конфигураций.
3. Перезапустите игру.

> [!NOTE]
> По умолчанию доступен `default.json`, но это не единственный вариант — см. разделы ниже и [документацию параметров](https://github.com/EXBO-Community/stalcraft-jvm-optimization/blob/master/docs/PARAMS.md) оригинального проекта (формат совместим).

#### Примеры конфигурации

В репозитории есть **`examples/8khz.json`** — профиль для high-end систем (8+ ядер, 32 ГБ ОЗУ, мышь 8 kHz): упор на минимальные STW-паузы и стабильный фреймтайм.

1. Скопируйте файл из [`stalcraft-jvm-wrapper/examples/`](stalcraft-jvm-wrapper/examples/) в `jvm_wrapper/configs/`.
2. Выберите профиль в GUI.
3. Перезапустите игру.

#### Кастомная конфигурация

Скопируйте `default.json`, переименуйте (например, `my_setup.json`) и отредактируйте в текстовом редакторе.

> [!CAUTION]
> Кастомная конфигурация — только если вы **понимаете**, что меняете. Иначе рискуете стабильностью JVM, игры и в крайних случаях — ОС.

Описание полей — в [PARAMS.md](https://github.com/EXBO-Community/stalcraft-jvm-optimization/blob/master/docs/PARAMS.md).

> [!TIP]
> Чтобы вернуть рекомендованные настройки для вашего ПК — перегенерируйте `default.json` через соответствующую функцию в GUI (аналог `Regenerate Config` в оригинальной CLI).

---

## Дополнительно

### Логирование

Один структурированный файл:

```
%AppData%\Roaming\EXBO\jvm_wrapper\logs\wrapper.log
```

В лог попадают: запуск, детект железа, загрузка конфига, старт игрового процесса, код выхода. Путь пользователя маскируется до `<user>`, сырые аргументы лаунчера и JVM-флаги **не пишутся**. Файл усекается при превышении **2 МБ**.

При сообщении о проблеме приложите этот файл к [issue](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/issues) — личных данных в нём нет.

### Large Pages

**Large Pages** — режим виртуальной памяти со страницами больше стандартных 4 KB. Снижает накладные расходы на работу с heap и делает GC стабильнее за счёт меньшего числа промахов TLB.

> [!CAUTION]
> Large Pages закрепляют память за приложением. Неправильная настройка может ухудшить стабильность ОС. Heap в профиле не должен превышать **40–50%** от общего ОЗУ; желательно **16+ ГБ** ОЗУ в системе.

Включение в Windows:

1. `Win` + `R` → `secpol.msc` → Enter.
2. *Локальные политики → Назначение прав пользователя*.
3. Политика *«Блокировка страниц в памяти»* → добавить своего пользователя или группу «Администраторы».
4. Применить и **перезагрузить ПК**.

### CLI (без GUI)

```text
stalcraft-jvm-wrapper.exe --install
stalcraft-jvm-wrapper.exe --uninstall
stalcraft-jvm-wrapper.exe --status
```

### Сборка из исходников

Исходники и dev-инструкции — в каталоге [`stalcraft-jvm-wrapper/`](stalcraft-jvm-wrapper/):

```powershell
cd stalcraft-jvm-wrapper
npm install
npm run build:prod
```

Результат: `wrapper.zip` (`stalcraft-jvm-wrapper.exe`, `service.exe`, `examples/`).

### Техническая информация

- Порт логики [stalcraft-jvm-optimization](https://github.com/EXBO-Community/stalcraft-jvm-optimization) (Go) на **Rust + Tauri 2**.
- IFEO через отдельный `service.exe`, запись в native 64-bit и WOW6432Node.
- Подробности оригинальной архитектуры: [OVERVIEW.md](https://github.com/EXBO-Community/stalcraft-jvm-optimization/blob/master/docs/OVERVIEW.md).
- Расширенная документация репозитория:
  - [README.md (EN)](stalcraft-jvm-wrapper/README.md)
  - [README_RU.md](stalcraft-jvm-wrapper/README_RU.md)
  - [CHANGELOG](stalcraft-jvm-wrapper/CHANGELOG.md)

---

## Авторы и поддержка

| Канал | Контакт |
|-------|---------|
| GitHub | [Nyrokume](https://github.com/Nyrokume) |
| Discord | `@nyrokume` |
| Telegram | [@nyrokume](https://t.me/nyrokume) |
| В игре | **DementiyRezak** |

**Авторы:** [Nyrokume](https://github.com/Nyrokume), [SilentBless](https://github.com/SilentBless) · [Nyrokume.dev](https://nyrokume.dev)

**Лицензия:** бесплатно, **некоммерческое** использование. Устанавливайте только из [официальных релизов](https://github.com/Nyrokume/Stalcraft-JVM-Flag-fix/releases) этого репозитория.
