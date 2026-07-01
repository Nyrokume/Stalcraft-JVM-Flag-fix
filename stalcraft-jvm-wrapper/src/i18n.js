const STORAGE_KEY = 'stalcraft-jvm-lang';

const STRINGS = {
    en: {
        appTitle: 'Stalcraft JVM Wrapper',
        headerTitle: 'STALCRAFT JVM WRAPPER',
        subtitle: 'Dynamic JVM optimization for STALCRAFT: X on Windows',
        statusOnline: 'SYSTEM ONLINE',
        execProtocol: 'EXECUTION PROTOCOL',
        guideTitle: 'How to launch',
        guideStep1: 'Unpack to %AppData%\\Roaming\\EXBO\\jvm_wrapper\\ — both stalcraft-jvm-wrapper.exe and service.exe',
        guideStep2: 'Click INSTALL and approve the UAC prompt',
        guideStep3: 'Click VERIFY — all targets must show ok, service.exe: present',
        guideStep4: 'Fully close the EXBO launcher, then start the game normally',
        guideStep5: 'Do not run service.exe manually — Windows starts it via IFEO',
        guideStep6: 'Check logs\\wrapper.log for service_invoked and jvm_mode=INJECTED',
        launchWarning: 'IFEO must be ACTIVE before the game starts. Restart the launcher after install.',
        config: 'CONFIGURATION',
        configLoading: 'Loading…',
        configNoActive: 'No active profile selected',
        configActive: 'Active: {name}',
        configActiveMissing: 'Active: {name} (missing — will use default)',
        savedProfiles: 'Saved profiles',
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
        ifeoDesc: 'Hooks stalcraft.exe, stalcraftw.exe, stalzone.exe, stalzonew.exe, game java.exe/javaw.exe via service.exe. Log: logs/wrapper.log',
        install: 'INSTALL',
        uninstall: 'UNINSTALL',
        verify: 'VERIFY',
        systemLog: 'SYSTEM OUTPUT LOG',
        footer: 'STALCRAFT JVM WRAPPER',
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
    },
    ru: {
        appTitle: 'Stalcraft JVM Wrapper',
        headerTitle: 'STALCRAFT JVM WRAPPER',
        subtitle: 'Динамическая оптимизация JVM для STALCRAFT: X в Windows',
        statusOnline: 'СИСТЕМА ОНЛАЙН',
        execProtocol: 'ПРОТОКОЛ ЗАПУСКА',
        guideTitle: 'Как запустить',
        guideStep1: 'Распакуйте в %AppData%\\Roaming\\EXBO\\jvm_wrapper\\ — stalcraft-jvm-wrapper.exe и service.exe',
        guideStep2: 'Нажмите INSTALL и подтвердите UAC',
        guideStep3: 'Нажмите VERIFY — все цели ok, service.exe: present',
        guideStep4: 'Полностью закройте лаунчер EXBO и запустите игру как обычно',
        guideStep5: 'Не запускайте service.exe вручную — его вызывает Windows через IFEO',
        guideStep6: 'Проверьте logs\\wrapper.log: service_invoked и jvm_mode=INJECTED',
        launchWarning: 'IFEO должен быть ACTIVE до старта игры. Перезапустите лаунчер после установки.',
        config: 'КОНФИГУРАЦИЯ',
        configLoading: 'Загрузка…',
        configNoActive: 'Профиль не выбран',
        configActive: 'Активный: {name}',
        configActiveMissing: 'Активный: {name} (нет файла — будет default)',
        savedProfiles: 'Сохранённые профили',
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
        ifeoDesc: 'Перехват stalcraft.exe, stalcraftw.exe, stalzone.exe, stalzonew.exe и java.exe/javaw.exe игры через service.exe. Лог: logs/wrapper.log',
        install: 'УСТАНОВИТЬ',
        uninstall: 'УДАЛИТЬ',
        verify: 'ПРОВЕРКА',
        systemLog: 'СИСТЕМНЫЙ ЛОГ',
        footer: 'STALCRAFT JVM WRAPPER',
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

export function renderLaunchGuide() {
    const root = document.getElementById('launch-guide-steps');
    if (!root) return;
    const hi = (s) => `<span class="highlight">${s}</span>`;
    const steps = [
        t('guideStep1'),
        t('guideStep2'),
        t('guideStep3'),
        t('guideStep4'),
        t('guideStep5'),
        t('guideStep6'),
    ];
    root.innerHTML = steps.map((text, i) => {
        let html = text
            .replace(/stalcraft-jvm-wrapper\.exe/g, hi('stalcraft-jvm-wrapper.exe'))
            .replace(/service\.exe/g, hi('service.exe'))
            .replace(/%AppData%\\Roaming\\EXBO\\jvm_wrapper\\/g, hi('%AppData%\\Roaming\\EXBO\\jvm_wrapper\\'))
            .replace(/INSTALL/g, hi('INSTALL'))
            .replace(/VERIFY/g, hi('VERIFY'))
            .replace(/ACTIVE/g, hi('ACTIVE'))
            .replace(/service_invoked/g, hi('service_invoked'))
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
    renderLaunchGuide();
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
