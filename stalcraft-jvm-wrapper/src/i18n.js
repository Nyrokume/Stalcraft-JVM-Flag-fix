const STORAGE_KEY = 'stalcraft-jvm-lang';
const LAUNCH_PLATFORM_KEY = 'stalcraft-jvm-launch-platform';
export const LAUNCH_PLATFORMS = ['exbo', 'steam', 'egs', 'vk'];

const STRINGS = {
    en: {
        appTitle: 'STALZONE JVM Wrapper',
        headerTitle: 'STALZONE JVM WRAPPER',
        subtitle: 'Dynamic JVM optimization for STALZONE on Windows',
        statusOnline: 'SYSTEM ONLINE',
        execProtocol: 'EXECUTION PROTOCOL',
        guideTitle: 'How to launch',
        guidePlatform_exbo: 'EXBO',
        guidePlatform_steam: 'Steam',
        guidePlatform_egs: 'EGS',
        guidePlatform_vk: 'VK Play',
        guidePath_exbo: '%AppData%\\Roaming\\EXBO\\jvm_wrapper\\',
        guidePath_steam: '…\\steamapps\\common\\STALCRAFT\\jvm_wrapper\\',
        guidePath_egs: '…\\Epic Games\\STALCRAFT\\jvm_wrapper\\',
        guidePath_vk: '…\\VK Play\\STALCRAFT\\jvm_wrapper\\',
        guideStep1: 'Unpack wrapper.zip to {path} — both stalcraft-jvm-wrapper.exe and service.exe',
        guideStep2: 'Click INSTALL and approve the UAC prompt',
        guideStep3: 'Click VERIFY — all 6 targets must show ok, service.exe: present',
        guideStep4: 'Click a JVM preset chip — it imports and applies automatically',
        guideStep5: 'Fully close the launcher, then start the game normally',
        guideStep6: 'Do not run service.exe manually — Windows starts it via IFEO',
        guideStep7: 'Check logs\\wrapper.log: inject=true, jvm_mode=INJECTED, launcher=exbo|steam|…',
        launchWarning: 'IFEO must be ACTIVE before the game starts. Restart the launcher after install.',
        config: 'CONFIGURATION',
        configLoading: 'Loading…',
        configNoActive: 'No active profile selected',
        configActive: 'Active: {name}',
        configActiveMissing: 'Active: {name} (missing — will use default)',
        savedProfiles: 'Saved profiles',
        jvmPresets: 'JVM presets (EXBO)',
        jvmPresetsHint: 'Source: EXBO release history — click a chip to import and apply',
        preset_weak: 'Weak',
        preset_medium: 'Medium',
        preset_max: 'Max',
        preset_balanced_mid: 'Balanced mid DDR',
        preset_slow_ddr: 'Slow DDR',
        preset_throughput_v110: 'Throughput v1.1.0',
        preset_x3d_v110: 'X3D v1.1.0',
        preset_8khz: '8 kHz mouse',
        preset_removed_fast_ddr: 'Fast DDR (removed)',
        presetHint_weak: 'EXBO v1.0.8 weak tier — low RAM / conservative GC',
        presetHint_medium: 'EXBO v1.0.8 medium tier — balanced default',
        presetHint_max: 'EXBO v1.0.8 max tier — high RAM / aggressive tuning',
        presetHint_balanced_mid: 'EXBO v1.1.1+ mid tier — combat-biased default for XMP DDR4 / DDR5',
        presetHint_slow_ddr: 'EXBO v1.1.1+ slow tier — DDR ≤2933 MT/s, looser GC pauses',
        presetHint_throughput_v110: 'EXBO v1.1.0 mainstream throughput profile',
        presetHint_x3d_v110: 'EXBO v1.1.0 X3D / big L3 profile (16T reference)',
        presetHint_8khz: 'EXBO examples — high-end, minimal STW pauses, 8 kHz mouse',
        presetHint_removed_fast_ddr: 'EXBO v1.1.1 removed fast DDR tier — experimental only',
        logImportPreset: 'Importing preset {name}…',
        logImportFail: 'Preset import failed: {err}',
        selectProfile: 'Select profile…',
        apply: 'Apply',
        regen: 'Regen',
        hardware: 'HARDWARE PROFILE',
        hwCpu: 'CPU',
        hwGpu: 'GPU',
        hwRam: 'RAM',
        hwMemory: 'MEMORY',
        hwHeap: 'TARGET HEAP',
        detecting: 'Detecting…',
        unknownCpu: 'Unknown CPU',
        unknownGpu: 'Unknown GPU',
        gpuSub: 'Graphics adapter',
        ramAvailable: '{n} GB free',
        memTier: 'Tier: {tier}',
        memTierSlow: 'Tier: slow (≤2933 MT/s)',
        memTierMid: 'Tier: mid',
        memSpeed: '{mts} MT/s',
        memSpeedUnknown: 'Speed unknown (mid tier fallback)',
        largePagesOn: 'Large pages ON',
        largePagesSize: 'Large pages {mb} MB',
        largePagesOff: 'Large pages off',
        coresThreads: '{cores} cores / {threads} threads',
        l3Cache: 'L3 {mb} MB',
        bigL3: 'Big L3',
        ifeo: 'IFEO REGISTRY',
        ifeoInactive: 'INACTIVE',
        ifeoActive: 'ACTIVE',
        ifeoDesc: 'Hooks game-scoped stalzone.exe, stalzonew.exe, stalcraft.exe, stalcraftw.exe and game java.exe/javaw.exe via service.exe. Log: logs/wrapper.log',
        install: 'INSTALL',
        uninstall: 'UNINSTALL',
        verify: 'VERIFY',
        systemLog: 'SYSTEM OUTPUT LOG',
        footer: 'STALZONE JVM WRAPPER',
        loadingInit: 'Initializing…',
        loadingDetect: 'Detecting hardware…',
        loadingSmbios: 'Reading SMBIOS memory speed…',
        loadingModules: 'Scanning memory modules…',
        loadingL3: 'Detecting L3 cache…',
        loadingJvm: 'Calculating JVM parameters…',
        loadingConfig: 'Loading config profiles…',
        loadingUi: 'Preparing interface…',
        loadingReady: 'Almost ready…',
        logStarted: 'Application started',
        logDetecting: 'Detecting system hardware…',
        logSystem: 'System: {cpu}, {ram} GB RAM, mem={tier}',
        logHeap: 'Heap: {heap} GB, config: {cfg}',
        logLargePages: 'Large pages: available',
        logDetectFail: 'System detection failed: {err}',
        logDetectFailHwid: 'Hardware detection failed — possible HWID change (spoofing). {err}',
        logIfeoInstall: 'Installing IFEO hook…',
        logIfeoOk: 'IFEO installed successfully',
        logIfeoFail: 'IFEO install failed: {err}',
        logIfeoRemove: 'Removing IFEO hook…',
        logIfeoRemoved: 'IFEO removed',
        logIfeoRemoveFail: 'IFEO remove failed: {err}',
        logIfeoCheck: 'Checking IFEO status…',
        logIfeoAllOk: 'IFEO: all targets configured',
        logIfeoMissing: 'IFEO: at least one target missing',
        logIfeoStatusFail: 'Status check failed: {err}',
        logServiceMissing: 'service.exe missing — copy it next to stalcraft-jvm-wrapper.exe',
        logIfeoPerTarget: 'IFEO registry (per target):',
        logIfeoAfterInstall: 'IFEO registry (verify after install):',
        logWrapperTail: '── logs/wrapper.log (tail) ──',
        logRegen: 'Regenerating default config for current hardware…',
        logRegenFail: 'Regenerate failed: {err}',
        logConfigFail: 'Config select failed: {err}',
        loadingBtn: 'LOADING',
        minimize: 'Minimize',
        maximize: 'Maximize',
        close: 'Close',
        langEn: 'EN',
        langRu: 'RU',
        welcomeTitle: 'License Agreement',
        welcomeAuthors: 'Authors: Nyrokume, SilentBless',
        welcomeLicenseP1: 'This software is free and non-commercial.',
        welcomeLicenseP2: 'The author is not responsible for the actions of third parties. Install only from trusted sources.',
        welcomeLicenseP3: 'By continuing, you accept these terms.',
        welcomeSupport: 'To support the author, send kind words via the contacts below or in-game — nickname',
        welcomeAccept: 'I accept the license agreement',
        welcomeOk: 'Accept and continue',
        welcomeInfoTitle: 'Authors & support',
        welcomeInfoOk: 'Got it',
        footerAuthor: 'by Nyrokume.dev',
        welcomeContacts: 'Contacts',
        contactGithub: 'GitHub',
        contactDiscord: 'Discord',
        contactTelegram: 'Telegram',
        navJvm: 'JVM Optimization',
        navServerBlocker: 'Server Blocker',
        navSoon: 'Soon',
        sbHeroKicker: 'Network filter',
        sbHeroTitle: 'Server Blocker',
        sbHeroDesc: 'Block game tunnel endpoints on ports 29450–29460 via Windows Firewall. UAC prompt on first block.',
        sbStatusIdle: 'NOT CONNECTED',
        sbStatusConfigured: 'RULES CONFIGURED',
        sbStatBlocked: 'Blocked',
        sbStatTotal: 'Servers',
        sbStatPools: 'Pools',
        sbStatRegions: 'Regions',
        sbPreviewBanner: 'Browser preview — install the Tauri app to apply firewall rules.',
        sbBlockingActive: 'Blocking active (firewall)',
        sbWarningTitle: 'Server blocking',
        sbWarningP1: 'This version applies outbound Windows Firewall rules on ports 29450–29460. Administrator rights (UAC) are required to start or stop blocking.',
        sbWarningP2: 'Do not use WinDivert-based local interceptors together with ExitLag, GearUP or similar game boosters — two stack hooks conflict and can break tunnels or cause lag spikes.',
        sbWarningP3: 'A server-side MITM via backend-*.stalzone and /address_list is planned so the client only sends allow/deny lists without kernel drivers. Details: docs/server-blocker-architecture-ru.md in the repository.',
        sbWarningAccept: 'I understand how blocking works and the booster risk',
        sbWarningOk: 'Open Server Blocker',
        sbBlockingBusy: 'Applying rules…',
        sbBlockSuccess: 'Firewall rules applied',
        sbBlockStopped: 'Firewall rules removed',
        sbBlockFailed: 'Blocking failed: {err}',
        sbAdminRequired: 'Administrator rights required',
        sbStopBlock: 'Stop blocking',
        sbRefreshCatalog: 'Refresh RU list',
        sbRegionRu: 'Russia',
        sbRegionEu: 'Europe',
        sbRegionNa: 'North America',
        sbRegionSea: 'Asia-Pacific',
        sbFilterRegion: 'Region',
        sbRegionAll: 'All',
        sbMode: 'Block mode',
        sbModeBlocklist: 'Block selected',
        sbModeBlocklistHint: 'Deny checked tunnels',
        sbModeAllowlist: 'Allow only selected',
        sbModeAllowlistHint: 'Whitelist mode',
        sbStartBlock: 'Start blocking',
        sbPingAll: 'Test ping',
        sbPingProgress: 'Pinging {done}/{total}…',
        sbPingDone: 'Ping: {ok}/{total} servers responded',
        sbPingDoneHidden: 'Ping: {ok}/{total} responded · {hidden} hidden (bad / not found)',
        sbPingFailed: 'Ping failed: {err}',
        sbAutoBest: 'Best per pool',
        sbAutoBestHint: 'Top 3 with ping ≤200 ms per region, block the rest',
        sbAutoBestDone: 'Best server selected in {pools} pool(s)',
        sbAutoBestDoneHidden: 'Best in {pools} pool(s) · {hidden} hidden (bad / not found)',
        sbAutoBestNoPing: 'No ping data — check network or try again',
        sbPingVeryBad: 'High latency (>300 ms)',
        sbPingNotFound: 'Server with IP {ip} not found',
        sbBestTag: 'BEST',
        sbBestPoolsShort: 'pools',
        sbReset: 'Reset selection',
        sbSearch: 'Search by name, pool or IP…',
        sbShowBlocked: 'Blocked only',
        sbBlocked: 'Blocked',
        sbAllowed: 'Allowed',
        sbBlockedShort: 'blocked',
        sbEmpty: 'No servers match your filters.',
        sbToggleHint: 'Mark server for blocking',
    },
    ru: {
        appTitle: 'STALZONE JVM Wrapper',
        headerTitle: 'STALZONE JVM WRAPPER',
        subtitle: 'Динамическая оптимизация JVM для STALZONE в Windows',
        statusOnline: 'СИСТЕМА ОНЛАЙН',
        execProtocol: 'ПРОТОКОЛ ЗАПУСКА',
        guideTitle: 'Как запустить',
        guidePlatform_exbo: 'EXBO',
        guidePlatform_steam: 'Steam',
        guidePlatform_egs: 'EGS',
        guidePlatform_vk: 'VK Play',
        guidePath_exbo: '%AppData%\\Roaming\\EXBO\\jvm_wrapper\\',
        guidePath_steam: '…\\steamapps\\common\\STALCRAFT\\jvm_wrapper\\',
        guidePath_egs: '…\\Epic Games\\STALCRAFT\\jvm_wrapper\\',
        guidePath_vk: '…\\VK Play\\STALCRAFT\\jvm_wrapper\\',
        guideStep1: 'Распакуйте wrapper.zip в {path} — stalcraft-jvm-wrapper.exe и service.exe',
        guideStep2: 'Нажмите INSTALL и подтвердите UAC',
        guideStep3: 'Нажмите VERIFY — все 6 целей ok, service.exe: present',
        guideStep4: 'Нажмите чип пресета JVM — импорт и применение сразу',
        guideStep5: 'Полностью закройте лаунчер и запустите игру как обычно',
        guideStep6: 'Не запускайте service.exe вручную — его вызывает Windows через IFEO',
        guideStep7: 'Проверьте logs\\wrapper.log: inject=true, jvm_mode=INJECTED, launcher=exbo|steam|…',
        launchWarning: 'IFEO должен быть ACTIVE до старта игры. Перезапустите лаунчер после установки.',
        config: 'КОНФИГУРАЦИЯ',
        configLoading: 'Загрузка…',
        configNoActive: 'Профиль не выбран',
        configActive: 'Активный: {name}',
        configActiveMissing: 'Активный: {name} (нет файла — будет default)',
        savedProfiles: 'Сохранённые профили',
        jvmPresets: 'Пресеты JVM (EXBO)',
        jvmPresetsHint: 'Источник: история релизов EXBO — клик по чипу импортирует и применяет',
        preset_weak: 'Weak',
        preset_medium: 'Medium',
        preset_max: 'Max',
        preset_balanced_mid: 'Balanced mid DDR',
        preset_slow_ddr: 'Медленная DDR',
        preset_throughput_v110: 'Throughput v1.1.0',
        preset_x3d_v110: 'X3D v1.1.0',
        preset_8khz: 'Мышь 8 kHz',
        preset_removed_fast_ddr: 'Fast DDR (снят)',
        presetHint_weak: 'EXBO v1.0.8 weak — мало RAM / мягкий GC',
        presetHint_medium: 'EXBO v1.0.8 medium — сбалансированный профиль',
        presetHint_max: 'EXBO v1.0.8 max — много RAM / агрессивная настройка',
        presetHint_balanced_mid: 'EXBO v1.1.1+ mid — боевой профиль для XMP DDR4 / DDR5',
        presetHint_slow_ddr: 'EXBO v1.1.1+ slow — DDR ≤2933 MT/s, более мягкие паузы GC',
        presetHint_throughput_v110: 'EXBO v1.1.0 — throughput для обычных CPU',
        presetHint_x3d_v110: 'EXBO v1.1.0 — профиль X3D / большой L3 (эталон 16T)',
        presetHint_8khz: 'EXBO examples — high-end, минимальные STW-паузы, мышь 8 kHz',
        presetHint_removed_fast_ddr: 'EXBO v1.1.1 — снятый fast DDR tier, только для тестов',
        logImportPreset: 'Импорт пресета {name}…',
        logImportFail: 'Ошибка импорта пресета: {err}',
        selectProfile: 'Выберите профиль…',
        apply: 'Применить',
        regen: 'Сброс',
        hardware: 'ЖЕЛЕЗО',
        hwCpu: 'CPU',
        hwGpu: 'GPU',
        hwRam: 'ОЗУ',
        hwMemory: 'ПАМЯТЬ',
        hwHeap: 'HEAP',
        detecting: 'Определение…',
        unknownCpu: 'CPU не определён',
        unknownGpu: 'GPU не определён',
        gpuSub: 'Видеоадаптер',
        ramAvailable: 'свободно {n} ГБ',
        memTier: 'Уровень: {tier}',
        memTierSlow: 'Уровень: slow (≤2933 MT/s)',
        memTierMid: 'Уровень: mid',
        memSpeed: '{mts} MT/s',
        memSpeedUnknown: 'Скорость неизвестна (mid tier)',
        largePagesOn: 'Large pages вкл.',
        largePagesSize: 'Large pages {mb} МБ',
        largePagesOff: 'Large pages выкл.',
        coresThreads: '{cores} ядер / {threads} потоков',
        l3Cache: 'L3 {mb} МБ',
        bigL3: 'Большой L3',
        ifeo: 'IFEO РЕЕСТР',
        ifeoInactive: 'НЕАКТИВЕН',
        ifeoActive: 'АКТИВЕН',
        ifeoDesc: 'Перехват game-scoped stalzone.exe, stalzonew.exe, stalcraft.exe, stalcraftw.exe и java.exe/javaw.exe игры через service.exe. Лог: logs/wrapper.log',
        install: 'УСТАНОВИТЬ',
        uninstall: 'УДАЛИТЬ',
        verify: 'ПРОВЕРКА',
        systemLog: 'СИСТЕМНЫЙ ЛОГ',
        footer: 'STALZONE JVM WRAPPER',
        loadingInit: 'Инициализация…',
        loadingDetect: 'Определение железа…',
        loadingSmbios: 'Чтение SMBIOS…',
        loadingModules: 'Сканирование модулей ОЗУ…',
        loadingL3: 'Определение L3…',
        loadingJvm: 'Расчёт параметров JVM…',
        loadingConfig: 'Загрузка профилей…',
        loadingUi: 'Подготовка интерфейса…',
        loadingReady: 'Почти готово…',
        logStarted: 'Приложение запущено',
        logDetecting: 'Определение железа…',
        logSystem: 'Система: {cpu}, {ram} ГБ ОЗУ, mem={tier}',
        logHeap: 'Heap: {heap} ГБ, конфиг: {cfg}',
        logLargePages: 'Large pages: доступны',
        logDetectFail: 'Ошибка определения железа: {err}',
        logDetectFailHwid: 'Ошибка определения железа — возможен HWID Change (Spoofing). {err}',
        logIfeoInstall: 'Установка IFEO…',
        logIfeoOk: 'IFEO установлен',
        logIfeoFail: 'Ошибка установки IFEO: {err}',
        logIfeoRemove: 'Удаление IFEO…',
        logIfeoRemoved: 'IFEO удалён',
        logIfeoRemoveFail: 'Ошибка удаления IFEO: {err}',
        logIfeoCheck: 'Проверка IFEO…',
        logIfeoAllOk: 'IFEO: все цели настроены',
        logIfeoMissing: 'IFEO: не все цели установлены',
        logIfeoStatusFail: 'Ошибка проверки: {err}',
        logServiceMissing: 'service.exe не найден — скопируйте рядом с stalcraft-jvm-wrapper.exe',
        logIfeoPerTarget: 'IFEO реестр (по целям):',
        logIfeoAfterInstall: 'IFEO реестр (после установки):',
        logWrapperTail: '── logs/wrapper.log (хвост) ──',
        logRegen: 'Пересоздание default.json под железо…',
        logRegenFail: 'Ошибка сброса: {err}',
        logConfigFail: 'Ошибка выбора профиля: {err}',
        loadingBtn: 'ЗАГРУЗКА',
        minimize: 'Свернуть',
        maximize: 'Развернуть',
        close: 'Закрыть',
        langEn: 'EN',
        langRu: 'RU',
        welcomeTitle: 'Лицензионное соглашение',
        welcomeAuthors: 'Авторы: Nyrokume, SilentBless',
        welcomeLicenseP1: 'Программа бесплатная и некоммерческая.',
        welcomeLicenseP2: 'Автор не несёт ответственности за действия третьих лиц. Программу рекомендуется устанавливать только из проверенных источников.',
        welcomeLicenseP3: 'Продолжая, вы принимаете эти условия.',
        welcomeSupport: 'Если хотите поддержать автора, напишите тёплые слова по контактным данным ниже или отправьте добрые слова в игре — никнейм',
        welcomeAccept: 'Я принимаю условия лицензии',
        welcomeOk: 'Принять и продолжить',
        welcomeInfoTitle: 'Авторы и поддержка',
        welcomeInfoOk: 'Понятно',
        footerAuthor: 'by Nyrokume.dev',
        welcomeContacts: 'Контакты',
        contactGithub: 'GitHub',
        contactDiscord: 'Discord',
        contactTelegram: 'Telegram',
        navJvm: 'Оптимизация JVM',
        navServerBlocker: 'Блокировка серверов',
        navSoon: 'Скоро',
        sbHeroKicker: 'Сетевой фильтр',
        sbHeroTitle: 'Блокировка серверов',
        sbHeroDesc: 'Блокировка туннелей на портах 29450–29460 через Windows Firewall. При первой блокировке — запрос UAC.',
        sbStatusIdle: 'НЕ ПОДКЛЮЧЕНО',
        sbStatusConfigured: 'ПРАВИЛА ЗАДАНЫ',
        sbStatBlocked: 'Заблок.',
        sbStatTotal: 'Серверов',
        sbStatPools: 'Пулов',
        sbStatRegions: 'Регионов',
        sbPreviewBanner: 'Превью в браузере — для блокировки установите Tauri-приложение.',
        sbBlockingActive: 'Блокировка активна (firewall)',
        sbWarningTitle: 'Блокировка серверов',
        sbWarningP1: 'В этой версии используются исходящие правила Windows Firewall на портах 29450–29460. Для старта и остановки блокировки нужны права администратора (UAC).',
        sbWarningP2: 'Не используйте локальные перехватчики на WinDivert вместе с ExitLag, GearUP и похожими бустерами — два перехвата сетевого стека конфликтуют и ломают туннели.',
        sbWarningP3: 'Планируется серверный MITM через backend-*.stalzone и API /address_list: клиент только отправляет списки allow/deny без драйверов в ядре. Подробности: docs/server-blocker-architecture-ru.md в репозитории.',
        sbWarningAccept: 'Я понимаю, как работает блокировка и риск при использовании бустеров',
        sbWarningOk: 'Открыть блокировку серверов',
        sbBlockingBusy: 'Применение правил…',
        sbBlockSuccess: 'Правила firewall применены',
        sbBlockStopped: 'Правила firewall сняты',
        sbBlockFailed: 'Ошибка блокировки: {err}',
        sbAdminRequired: 'Нужны права администратора',
        sbStopBlock: 'Разблокировать',
        sbRefreshCatalog: 'Обновить RU список',
        sbRegionRu: 'Россия',
        sbRegionEu: 'Европа',
        sbRegionNa: 'Северная Америка',
        sbRegionSea: 'Азия и Океания',
        sbFilterRegion: 'Регион',
        sbRegionAll: 'Все',
        sbMode: 'Режим блокировки',
        sbModeBlocklist: 'Блокировать выбранные',
        sbModeBlocklistHint: 'Запретить отмеченные туннели',
        sbModeAllowlist: 'Только выбранные',
        sbModeAllowlistHint: 'Режим белого списка',
        sbStartBlock: 'Начать блокировку',
        sbPingAll: 'Проверить пинг',
        sbPingProgress: 'Пинг {done}/{total}…',
        sbPingDone: 'Пинг: ответили {ok}/{total} серверов',
        sbPingDoneHidden: 'Пинг: {ok}/{total} · скрыто {hidden} (плохой / не найден)',
        sbPingFailed: 'Ошибка пинга: {err}',
        sbAutoBest: 'Лучший в пуле',
        sbAutoBestHint: 'Топ-3 с пингом ≤200 мс в регионе, остальные блокируются',
        sbAutoBestDone: 'Выбран лучший сервер в {pools} пул(ах)',
        sbAutoBestDoneHidden: 'Лучшие в {pools} пул(ах) · скрыто {hidden} (плохой / не найден)',
        sbAutoBestNoPing: 'Нет данных пинга — проверьте сеть или повторите',
        sbPingVeryBad: 'Плохой пинг (>300 мс)',
        sbPingNotFound: 'Сервер с указанным IP {ip} не найден',
        sbBestTag: 'ЛУЧШИЙ',
        sbBestPoolsShort: 'пулов',
        sbReset: 'Сбросить выбор',
        sbSearch: 'Поиск по имени, пулу или IP…',
        sbShowBlocked: 'Только заблок.',
        sbBlocked: 'Заблокирован',
        sbAllowed: 'Разрешён',
        sbBlockedShort: 'заблок.',
        sbEmpty: 'Нет серверов по вашим фильтрам.',
        sbToggleHint: 'Отметить для блокировки',
    },
};

let currentLang = localStorage.getItem(STORAGE_KEY) || 'ru';

export function getLang() {
    return currentLang;
}

export function t(key, vars = {}) {
    const raw = STRINGS[currentLang]?.[key] ?? STRINGS.en[key] ?? key;
    return raw.replace(/\{(\w+)\}/g, (_, k) => (vars[k] != null ? String(vars[k]) : `{${k}}`));
}

export function loadingMessages() {
    return [
        t('loadingDetect'),
        t('loadingSmbios'),
        t('loadingModules'),
        t('loadingL3'),
        t('loadingJvm'),
        t('loadingConfig'),
        t('loadingUi'),
        t('loadingReady'),
    ];
}

function setText(el, text) {
    if (el) el.textContent = text;
}

function setHtml(el, html) {
    if (el) el.innerHTML = html;
}

export function getLaunchPlatform() {
    const saved = localStorage.getItem(LAUNCH_PLATFORM_KEY);
    return LAUNCH_PLATFORMS.includes(saved) ? saved : 'exbo';
}

export function setLaunchPlatform(platform) {
    if (!LAUNCH_PLATFORMS.includes(platform)) return;
    localStorage.setItem(LAUNCH_PLATFORM_KEY, platform);
    renderLaunchGuide();
    document.querySelectorAll('.launch-guide-platform').forEach((btn) => {
        btn.classList.toggle('active', btn.dataset.platform === platform);
    });
}

export function setupLaunchGuidePlatforms() {
    const root = document.getElementById('launch-guide-platforms');
    if (!root) return;
    const current = getLaunchPlatform();
    root.replaceChildren();
    for (const platform of LAUNCH_PLATFORMS) {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = `launch-guide-platform${platform === current ? ' active' : ''}`;
        btn.dataset.platform = platform;
        btn.textContent = t(`guidePlatform_${platform}`);
        btn.addEventListener('click', () => setLaunchPlatform(platform));
        root.appendChild(btn);
    }
}

export function renderLaunchGuide() {
    const root = document.getElementById('launch-guide-steps');
    if (!root) return;
    const platform = getLaunchPlatform();
    const path = t(`guidePath_${platform}`);
    const hi = (s) => `<span class="highlight">${s}</span>`;
    const steps = [
        t('guideStep1', { path }),
        t('guideStep2'),
        t('guideStep3'),
        t('guideStep4'),
        t('guideStep5'),
        t('guideStep6'),
        t('guideStep7'),
    ];
    root.innerHTML = steps.map((text) => {
        let html = text
            .replace(/stalcraft-jvm-wrapper\.exe/g, hi('stalcraft-jvm-wrapper.exe'))
            .replace(/service\.exe/g, hi('service.exe'))
            .replace(/wrapper\.zip/g, hi('wrapper.zip'))
            .replace(/%AppData%\\Roaming\\EXBO\\jvm_wrapper\\/g, hi('%AppData%\\Roaming\\EXBO\\jvm_wrapper\\'))
            .replace(/…\\steamapps\\common\\STALCRAFT\\jvm_wrapper\\/g, hi('…\\steamapps\\common\\STALCRAFT\\jvm_wrapper\\'))
            .replace(/…\\Epic Games\\STALCRAFT\\jvm_wrapper\\/g, hi('…\\Epic Games\\STALCRAFT\\jvm_wrapper\\'))
            .replace(/…\\VK Play\\STALCRAFT\\jvm_wrapper\\/g, hi('…\\VK Play\\STALCRAFT\\jvm_wrapper\\'))
            .replace(/INSTALL/g, hi('INSTALL'))
            .replace(/VERIFY/g, hi('VERIFY'))
            .replace(/inject=true/g, hi('inject=true'))
            .replace(/jvm_mode=INJECTED/g, hi('jvm_mode=INJECTED'))
            .replace(/logs\\wrapper\.log/g, hi('logs\\wrapper.log'));
        return `<li>${html}</li>`;
    }).join('');
}

export function applyI18n() {
    document.documentElement.lang = currentLang;
    document.querySelectorAll('[data-i18n]').forEach((el) => setText(el, t(el.dataset.i18n)));
    document.querySelectorAll('[data-i18n-title]').forEach((el) => {
        el.title = t(el.dataset.i18nTitle);
    });
    document.querySelectorAll('[data-i18n-placeholder]').forEach((el) => {
        el.placeholder = t(el.dataset.i18nPlaceholder);
    });
    setupLaunchGuidePlatforms();
    renderLaunchGuide();
    if (typeof window.__renderServerBlocker === 'function') window.__renderServerBlocker();
    updateLangButtons();
    document.title = t('appTitle');
}

function updateLangButtons() {
    document.querySelectorAll('.lang-btn').forEach((btn) => {
        btn.classList.toggle('active', btn.dataset.lang === currentLang);
    });
}

export function setLang(lang) {
    if (!STRINGS[lang]) return;
    currentLang = lang;
    localStorage.setItem(STORAGE_KEY, lang);
    applyI18n();
    if (typeof window.__onLangChange === 'function') window.__onLangChange();
}

export function setupLangSwitcher() {
    document.querySelectorAll('.lang-btn').forEach((btn) => {
        btn.addEventListener('click', () => setLang(btn.dataset.lang));
    });
    applyI18n();
}
