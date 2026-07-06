// main.js — slim GUI (Go cli parity)

import { t, loadingMessages, applyI18n, setupLangSwitcher } from './i18n.js';
import { initServerBlocker } from './server-blocker.js';

let invoke;
let isTauri = false;
let useMockBackend = false;
let currentWindow;

let refreshBtn, installBtn, uninstallBtn, verifyBtn, ifeoResult;
let logContainer, currentTimeEl;
let cpuInfo, gpuInfo, ramFill, ramTotal, ramAvailable, heapSize, ifeoStatus;
let memTier, memSpeed;
let btnMinimize, btnMaximize, btnClose;
let btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading;
let loadingScreen, loadingProgress, loadingStatus;
let configSelect, configActiveLabel, regenerateConfigBtn, selectConfigBtn, configPresetGrid;

const PRESET_ORDER = [
    'balanced_mid',
    'slow_ddr',
    'throughput_v110',
    'x3d_v110',
    '8khz',
    'removed_fast_ddr',
];
let ifeoStatusText;

const APP_PAGES = ['jvm', 'server-blocker'];
const PAGE_STORAGE_KEY = 'stalcraft-jvm-page';
const WELCOME_STORAGE_KEY = 'stalcraft-jvm-welcome-v1';
const SB_WARNING_STORAGE_KEY = 'stalcraft-jvm-sb-warning-v1';
let serverBlockerApi = null;

function hasBackend() {
    return isTauri || useMockBackend;
}

async function initTauriAPI() {
    const { isTauri: inTauri, invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    isTauri = inTauri();
    currentWindow = null;

    if (isTauri) {
        invoke = typeof window !== 'undefined' && window.__TAURI__?.core?.invoke
            ? window.__TAURI__.core.invoke.bind(window.__TAURI__.core)
            : tauriInvoke;
        try {
            const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
            currentWindow = getCurrentWebviewWindow();
        } catch (_) {
            currentWindow = null;
        }
        return;
    }

    if (import.meta.env.DEV || import.meta.env.MODE === 'test') {
        const { createDevMockInvoke, showDevBanner } = await import('./dev-mock.js');
        invoke = createDevMockInvoke();
        useMockBackend = true;
        showDevBanner();
        return;
    }

    invoke = async (cmd) => {
        throw new Error(`Tauri command '${cmd}' not available outside the app`);
    };
}

function formatDetectError(err) {
    const msg = String(err);
    if (/invoke|TypeError|not available|undefined/i.test(msg)) {
        return t('logDetectFailHwid', { err: msg });
    }
    return t('logDetectFail', { err: msg });
}

function initElements() {
    refreshBtn = document.getElementById('refresh-btn');
    installBtn = document.getElementById('install-btn');
    uninstallBtn = document.getElementById('uninstall-btn');
    verifyBtn = document.getElementById('verify-btn');
    ifeoResult = document.getElementById('ifeo-result');
    logContainer = document.getElementById('log-container');
    currentTimeEl = document.getElementById('current-time');
    cpuInfo = document.getElementById('cpu-info');
    gpuInfo = document.getElementById('gpu-info');
    ramFill = document.getElementById('ram-fill');
    ramTotal = document.getElementById('ram-total');
    ramAvailable = document.getElementById('ram-available');
    heapSize = document.getElementById('heap-size');
    memTier = document.getElementById('mem-tier');
    memSpeed = document.getElementById('mem-speed');
    ifeoStatus = document.getElementById('ifeo-status');
    ifeoStatusText = document.getElementById('ifeo-status-text');
    btnMinimize = document.getElementById('btn-minimize');
    btnMaximize = document.getElementById('btn-maximize');
    btnClose = document.getElementById('btn-close');
    btnMinimizeLoading = document.getElementById('btn-minimize-loading');
    btnMaximizeLoading = document.getElementById('btn-maximize-loading');
    btnCloseLoading = document.getElementById('btn-close-loading');
    loadingScreen = document.getElementById('loading-screen');
    loadingProgress = document.getElementById('loading-progress');
    loadingStatus = document.getElementById('loading-status');
    configSelect = document.getElementById('config-select');
    configActiveLabel = document.getElementById('config-active-label');
    regenerateConfigBtn = document.getElementById('regenerate-config-btn');
    selectConfigBtn = document.getElementById('select-config-btn');
    configPresetGrid = document.getElementById('config-preset-grid');
}

function setupWindowControls(minimizeBtn, maximizeBtn, closeBtn) {
    if (minimizeBtn) minimizeBtn.addEventListener('click', () => currentWindow?.minimize());
    if (maximizeBtn) maximizeBtn.addEventListener('click', async () => {
        if (currentWindow) {
            (await currentWindow.isMaximized()) ? currentWindow.unmaximize() : currentWindow.maximize();
        }
    });
    if (closeBtn) closeBtn.addEventListener('click', () => currentWindow?.close());
}

function pickInfo(info, snake, camel) {
    if (info == null) return undefined;
    if (info[snake] != null) return info[snake];
    if (camel && info[camel] != null) return info[camel];
    return undefined;
}

function numInfo(info, snake, camel, fallback = 0) {
    const v = pickInfo(info, snake, camel);
    const n = Number(v);
    return Number.isFinite(n) ? n : fallback;
}

function strInfo(info, snake, camel, fallback = '') {
    const v = pickInfo(info, snake, camel);
    const s = v == null ? '' : String(v).trim();
    return s || fallback;
}

function esc(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

function setIfeoBadge(active) {
    if (!ifeoStatus) return;
    ifeoStatus.className = `status-badge ${active ? 'active' : 'inactive'}`;
    if (ifeoStatusText) {
        ifeoStatusText.textContent = active ? t('ifeoActive') : t('ifeoInactive');
    } else {
        ifeoStatus.innerHTML = `<span class="status-dot ${active ? 'active' : 'inactive'}"></span> ${active ? t('ifeoActive') : t('ifeoInactive')}`;
    }
}

function formatHardwareName(name, fallback) {
    const raw = (name || '').trim();
    if (!raw || raw === '—' || /^unknown/i.test(raw)) return fallback;
    return raw
        .replace(/\s+/g, ' ')
        .replace(/\s*@\s*[\d.]+\s*GHz/i, '')
        .trim();
}

function formatMemTier(tier) {
    const v = String(tier || 'mid').toLowerCase();
    if (v === 'slow') return t('memTierSlow');
    if (v === 'mid') return t('memTierMid');
    return t('memTier', { tier: v });
}

function applyHardwareInfo(info) {
    if (!info || !cpuInfo || !gpuInfo) return;

    const cpuName = formatHardwareName(strInfo(info, 'cpu_name', 'cpuName'), t('unknownCpu'));
    const gpuName = formatHardwareName(strInfo(info, 'gpu_name', 'gpuName'), t('unknownGpu'));
    const cores = numInfo(info, 'cpu_cores', 'cpuCores');
    const threads = numInfo(info, 'cpu_threads', 'cpuThreads');
    const l3 = numInfo(info, 'l3_cache_mb', 'l3CacheMb');
    const hasBigCache = Boolean(pickInfo(info, 'has_big_cache', 'hasBigCache'));
    const totalRam = numInfo(info, 'total_ram_gb', 'totalRamGb');
    const freeRam = numInfo(info, 'free_ram_gb', 'freeRamGb');
    const heapGb = numInfo(info, 'suggested_heap_gb', 'suggestedHeapGb');
    const memTierVal = strInfo(info, 'mem_tier', 'memTier', 'mid');
    const memMts = numInfo(info, 'mem_speed_mts', 'memSpeedMts');
    const largePages = Boolean(pickInfo(info, 'large_pages', 'largePages'));
    const lpMb = numInfo(info, 'large_page_size_mb', 'largePageSizeMb');

    let cpuSub = t('coresThreads', { cores, threads });
    if (l3 > 0) cpuSub += ` • ${t('l3Cache', { mb: l3 })}`;
    if (hasBigCache) cpuSub += ` • ${t('bigL3')}`;

    cpuInfo.innerHTML = `<div class="hw-main">${esc(cpuName)}</div><div class="hw-sub">${esc(cpuSub)}</div>`;
    gpuInfo.innerHTML = `<div class="hw-main">${esc(gpuName)}</div><div class="hw-sub">${esc(t('gpuSub'))}</div>`;

    const usedPct = totalRam > 0 ? Math.min(100, Math.max(0, ((totalRam - freeRam) / totalRam) * 100)) : 0;
    ramFill.style.width = `${usedPct.toFixed(0)}%`;
    ramTotal.textContent = totalRam > 0 ? `${totalRam.toFixed(1)} GB` : t('detecting');
    ramAvailable.textContent = totalRam > 0 ? t('ramAvailable', { n: freeRam.toFixed(1) }) : '';

    if (memTier) memTier.textContent = formatMemTier(memTierVal);
    if (memSpeed) {
        const lp = largePages
            ? (lpMb > 0 ? ` • ${t('largePagesSize', { mb: lpMb })}` : ` • ${t('largePagesOn')}`)
            : ` • ${t('largePagesOff')}`;
        memSpeed.textContent = memMts > 0 ? `${t('memSpeed', { mts: memMts })}${lp}` : `${t('memSpeedUnknown')}${lp}`;
    }

    if (heapSize) heapSize.textContent = heapGb > 0 ? `${Math.round(heapGb * 1024)} MB` : '—';
}

function animateLoadingScreen() {
    const messages = loadingMessages();
    if (loadingStatus) loadingStatus.textContent = t('loadingInit');
    const hwPromise = hasBackend() ? invoke('get_system_info').catch(() => null) : Promise.resolve(null);
    const totalDuration = useMockBackend ? 1200 : 5000;
    return new Promise((resolve) => {
        const messageInterval = totalDuration / messages.length;
        const progressStep = 100 / (totalDuration / 50);
        let progress = 0, messageIndex = 0;

        const messageTimer = setInterval(() => {
            if (messageIndex < messages.length) {
                loadingStatus.textContent = messages[messageIndex++];
            }
        }, messageInterval);

        const progressTimer = setInterval(() => {
            progress += progressStep;
            if (progress >= 100) {
                progress = 100;
                clearInterval(progressTimer);
                clearInterval(messageTimer);
                hwPromise.then((info) => { if (info) applyHardwareInfo(info); });
                setTimeout(() => { loadingScreen.classList.add('hidden'); resolve(); }, 200);
            }
            loadingProgress.style.width = progress + '%';
        }, 50);
    });
}

function updateClock() {
    const now = new Date();
    currentTimeEl.textContent = `${now.toISOString().split('T')[0]} // ${now.toTimeString().split(' ')[0]}`;
}

function getTimestamp() {
    const now = new Date();
    return `[${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}:${String(now.getSeconds()).padStart(2,'0')}]`;
}

function isIfeoStatusActive(statusText) {
    const lines = (statusText || '').split(/\r?\n/).map((s) => s.trim()).filter(Boolean);
    if (lines.length === 0) return false;
    if (lines.some((line) => /service\.exe: MISSING/i.test(line))) return false;
    const targets = lines.filter((line) => /\.exe: (ok|not installed|wrong)/i.test(line));
    if (targets.length === 0) return false;
    return targets.every((line) => /: ok /i.test(line));
}

function logIfeoRegistryLines(statusText, heading) {
    const text = (statusText || '').trim();
    if (!text) return;
    addLog(heading, 'info');
    for (const line of text.split(/\r?\n/)) {
        if (line.trim()) addLog(`  ${line.trim()}`, 'info');
    }
}

async function pullWrapperLogToUi(maxLines = 280) {
    if (!isTauri) return;
    try {
        const tail = await invoke('read_wrapper_log_tail', { maxLines });
        const text = tail != null ? String(tail).trim() : '';
        if (!text) return;
        addLog(t('logWrapperTail'), 'info');
        for (const line of text.split(/\r?\n/)) {
            if (line.trim()) addLog(line.trim(), 'info');
        }
    } catch (_) {}
}

function addLog(message, type = '') {
    if (!logContainer) return;
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    const timeSpan = document.createElement('span');
    timeSpan.className = 'log-time';
    timeSpan.textContent = getTimestamp();
    const arrow = document.createElement('span');
    arrow.className = 'log-arrow';
    arrow.textContent = '>';
    const textSpan = document.createElement('span');
    textSpan.className = type ? `log-text ${type}` : 'log-text';
    textSpan.textContent = message;
    entry.appendChild(timeSpan);
    entry.appendChild(arrow);
    entry.appendChild(textSpan);
    logContainer.appendChild(entry);
    logContainer.scrollTop = logContainer.scrollHeight;
}

function setLoading(button, loading) {
    button.disabled = loading;
    if (loading) {
        button.dataset.originalHTML = button.innerHTML;
        button.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px;animation:spin 0.8s linear infinite"><path d="M21 12a9 9 0 11-6.219-8.56"/></svg> ${t('loadingBtn')}`;
    } else if (button.dataset.originalHTML) {
        button.innerHTML = button.dataset.originalHTML;
    }
}

function setRefreshLoading(loading) {
    loading ? (refreshBtn.classList.add('spinning'), refreshBtn.disabled = true)
             : (refreshBtn.classList.remove('spinning'), refreshBtn.disabled = false);
}

async function syncHeapDisplay() {
    if (!heapSize || !hasBackend()) return;
    try {
        const info = await invoke('get_system_info');
        const heapGb = numInfo(info, 'suggested_heap_gb', 'suggestedHeapGb');
        if (heapGb > 0) heapSize.textContent = `${Math.round(heapGb * 1024)} MB`;
    } catch (_) {}
}

async function refreshConfigList() {
    try {
        const result = await invoke('list_configs');
        if (!configSelect) return;
        configSelect.innerHTML = '';
        const placeholder = document.createElement('option');
        placeholder.value = '';
        placeholder.textContent = t('selectProfile');
        configSelect.appendChild(placeholder);
        for (const name of result.names) {
            const opt = document.createElement('option');
            opt.value = name;
            opt.textContent = name;
            if (name === result.active) opt.selected = true;
            configSelect.appendChild(opt);
        }
        if (configActiveLabel) {
            if (result.active) {
                configActiveLabel.textContent = result.active_exists
                    ? t('configActive', { name: result.active })
                    : t('configActiveMissing', { name: result.active });
                configActiveLabel.className = result.active_exists ? 'config-active-label success' : 'config-active-label warning';
            } else {
                configActiveLabel.textContent = t('configNoActive');
                configActiveLabel.className = 'config-active-label';
            }
        }
    } catch (e) {
        console.error('Failed to load config list:', e);
    }
}

function presetLabel(name) {
    const key = `preset_${name}`;
    const label = t(key);
    return label !== key ? label : name;
}

function presetHint(name) {
    const key = `presetHint_${name}`;
    const hint = t(key);
    return hint !== key ? hint : presetLabel(name);
}

async function refreshPresetGrid() {
    if (!configPresetGrid || !hasBackend()) return;
    try {
        const [examples, saved] = await Promise.all([
            invoke('list_examples'),
            invoke('list_configs'),
        ]);
        const names = examples.names ?? [];
        const savedSet = new Set(saved.names ?? []);
        configPresetGrid.replaceChildren();
        const ordered = [
            ...PRESET_ORDER.filter((n) => names.includes(n)),
            ...names.filter((n) => !PRESET_ORDER.includes(n)),
        ];
        for (const name of ordered) {
            const chip = document.createElement('button');
            chip.type = 'button';
            chip.className = `config-preset-chip${savedSet.has(name) ? ' active' : ''}`;
            chip.textContent = presetLabel(name);
            chip.title = presetHint(name);
            chip.addEventListener('click', () => importPreset(name));
            configPresetGrid.appendChild(chip);
        }
    } catch (e) {
        console.error('Failed to load JVM presets:', e);
    }
}

async function importPreset(name) {
    if (!hasBackend()) return;
    addLog(t('logImportPreset', { name }), 'info');
    try {
        const result = await invoke('import_example_config', { name });
        addLog(result, 'success');
        await refreshConfigList();
        await refreshPresetGrid();
    } catch (e) {
        addLog(t('logImportFail', { err: e }), 'error');
    }
}

function switchAppPage(pageId) {
    if (!APP_PAGES.includes(pageId)) pageId = 'jvm';
    document.querySelectorAll('.app-page').forEach((el) => {
        const active = el.id === `page-${pageId}`;
        el.classList.toggle('active', active);
        el.classList.toggle('hidden', !active);
    });
    document.querySelectorAll('.app-nav-btn').forEach((btn) => {
        btn.classList.toggle('active', btn.dataset.page === pageId);
    });
    try {
        localStorage.setItem(PAGE_STORAGE_KEY, pageId);
    } catch (_) {}
    if (pageId === 'server-blocker') {
        showServerBlockerWarningIfNeeded().then(() => {
            serverBlockerApi?.pingVisibleIfNeeded?.();
        });
    }
}

function showServerBlockerWarningIfNeeded() {
    return new Promise((resolve) => {
        try {
            if (localStorage.getItem(SB_WARNING_STORAGE_KEY) === '1') {
                resolve();
                return;
            }
        } catch (_) {
            resolve();
            return;
        }

        const modal = document.getElementById('sb-warning-modal');
        const accept = document.getElementById('sb-warning-accept');
        const okBtn = document.getElementById('sb-warning-ok');
        if (!modal || !accept || !okBtn) {
            resolve();
            return;
        }

        const syncOk = () => {
            okBtn.disabled = !accept.checked;
        };
        accept.addEventListener('change', syncOk);
        syncOk();

        modal.classList.remove('hidden');
        modal.setAttribute('aria-hidden', 'false');

        const finish = () => {
            modal.classList.add('hidden');
            modal.setAttribute('aria-hidden', 'true');
            try {
                localStorage.setItem(SB_WARNING_STORAGE_KEY, '1');
            } catch (_) {}
            resolve();
        };

        okBtn.addEventListener('click', finish, { once: true });
    });
}

function setupAppNav() {
    const saved = (() => {
        try {
            const p = localStorage.getItem(PAGE_STORAGE_KEY);
            return APP_PAGES.includes(p) ? p : 'jvm';
        } catch (_) {
            return 'jvm';
        }
    })();
    switchAppPage(saved);
    document.querySelectorAll('.app-nav-btn').forEach((btn) => {
        btn.addEventListener('click', () => switchAppPage(btn.dataset.page));
    });
}

function setupServerBlocker() {
    return initServerBlocker({
        t,
        invoke: hasBackend() ? invoke : null,
    });
}

async function runSystemRefresh({ showLoadingSpinner = true } = {}) {
    if (!hasBackend() || !refreshBtn) return;
    if (showLoadingSpinner) setRefreshLoading(true);
    addLog(t('logDetecting'), 'info');
    try {
        const info = await invoke('get_system_info');
        applyHardwareInfo(info);

        const cpuName = strInfo(info, 'cpu_name', 'cpuName', t('unknownCpu'));
        const totalRam = numInfo(info, 'total_ram_gb', 'totalRamGb');
        const memTierVal = strInfo(info, 'mem_tier', 'memTier', 'mid');
        const heapGb = numInfo(info, 'suggested_heap_gb', 'suggestedHeapGb');
        const activeCfg = pickInfo(info, 'active_config', 'activeConfig') || 'default';
        const largePages = Boolean(pickInfo(info, 'large_pages', 'largePages'));

        addLog(t('logSystem', { cpu: cpuName, ram: totalRam.toFixed(1), tier: memTierVal }), 'success');
        addLog(t('logHeap', { heap: heapGb, cfg: activeCfg }), 'info');
        if (largePages) addLog(t('logLargePages'), 'success');

        await refreshConfigList();
        await refreshPresetGrid();
        await pullWrapperLogToUi();
    } catch (e) {
        addLog(formatDetectError(e), 'error');
    } finally {
        if (showLoadingSpinner) setRefreshLoading(false);
    }
}

function setupEventListeners() {
    setupWindowControls(btnMinimize, btnMaximize, btnClose);
    setupWindowControls(btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading);

    refreshBtn.addEventListener('click', () => runSystemRefresh({ showLoadingSpinner: true }));

    installBtn.addEventListener('click', async () => {
        setLoading(installBtn, true);
        addLog(t('logIfeoInstall'), 'info');
        try {
            const result = await invoke('install_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            setIfeoBadge(true);
            addLog(t('logIfeoOk'), 'success');
            try {
                const st = await invoke('check_status');
                logIfeoRegistryLines(st, t('logIfeoAfterInstall'));
            } catch (_) {}
            await pullWrapperLogToUi();
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(t('logIfeoFail', { err: e }), 'error');
        } finally {
            setLoading(installBtn, false);
        }
    });

    uninstallBtn.addEventListener('click', async () => {
        setLoading(uninstallBtn, true);
        addLog(t('logIfeoRemove'), 'info');
        try {
            const result = await invoke('uninstall_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            setIfeoBadge(false);
            addLog(t('logIfeoRemoved'), 'success');
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(t('logIfeoRemoveFail', { err: e }), 'error');
        } finally {
            setLoading(uninstallBtn, false);
        }
    });

    verifyBtn.addEventListener('click', async () => {
        setLoading(verifyBtn, true);
        addLog(t('logIfeoCheck'), 'info');
        try {
            const result = await invoke('check_status');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result info';
            const isActive = isIfeoStatusActive(result);
            setIfeoBadge(isActive);
            logIfeoRegistryLines(result, t('logIfeoPerTarget'));
            addLog(isActive ? t('logIfeoAllOk') : t('logIfeoMissing'), isActive ? 'success' : 'error');
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(t('logIfeoStatusFail', { err: e }), 'error');
        } finally {
            setLoading(verifyBtn, false);
        }
    });

    selectConfigBtn?.addEventListener('click', async () => {
        if (!configSelect?.value) return;
        const name = configSelect.value;
        setLoading(selectConfigBtn, true);
        try {
            const result = await invoke('select_config', { name });
            addLog(result, 'success');
            await refreshConfigList();
            await syncHeapDisplay();
        } catch (e) {
            addLog(t('logConfigFail', { err: e }), 'error');
        } finally {
            setLoading(selectConfigBtn, false);
        }
    });

    regenerateConfigBtn?.addEventListener('click', async () => {
        setLoading(regenerateConfigBtn, true);
        addLog(t('logRegen'), 'info');
        try {
            const result = await invoke('regenerate_config');
            addLog(result, 'success');
            await refreshConfigList();
            await syncHeapDisplay();
        } catch (e) {
            addLog(t('logRegenFail', { err: e }), 'error');
        } finally {
            setLoading(regenerateConfigBtn, false);
        }
    });

    setupAppNav();
}

function setupWelcomeModal() {
    return new Promise((resolve) => {
        if (localStorage.getItem(WELCOME_STORAGE_KEY) === '1') {
            resolve();
            return;
        }

        const licenseModal = document.getElementById('license-modal');
        const infoModal = document.getElementById('info-modal');
        const accept = document.getElementById('welcome-accept');
        const licenseOk = document.getElementById('license-ok');
        const infoOk = document.getElementById('info-ok');
        if (!licenseModal || !infoModal || !licenseOk || !infoOk || !accept) {
            resolve();
            return;
        }

        const syncLicenseOk = () => {
            licenseOk.disabled = !accept.checked;
        };

        accept.addEventListener('change', syncLicenseOk);
        syncLicenseOk();

        const showInfo = () => {
            licenseModal.classList.add('hidden');
            licenseModal.setAttribute('aria-hidden', 'true');
            infoModal.classList.remove('hidden');
            infoModal.setAttribute('aria-hidden', 'false');
        };

        const finish = () => {
            infoModal.classList.add('hidden');
            infoModal.setAttribute('aria-hidden', 'true');
            try {
                localStorage.setItem(WELCOME_STORAGE_KEY, '1');
            } catch (_) {}
            resolve();
        };

        licenseOk.addEventListener('click', () => {
            if (!accept.checked) return;
            showInfo();
        });

        infoOk.addEventListener('click', finish);

        licenseModal.classList.remove('hidden');
        licenseModal.setAttribute('aria-hidden', 'false');
    });
}

async function initializeApp() {
    updateClock();
    setInterval(updateClock, 1000);
    addLog(t('logStarted'), 'success');

    await setupWelcomeModal();

    if (hasBackend()) {
        try {
            const result = await invoke('check_status');
            const isActive = isIfeoStatusActive(result);
            setIfeoBadge(isActive);
            logIfeoRegistryLines(result, t('logIfeoPerTarget'));
            if (/service\.exe: MISSING/i.test(result)) {
                addLog(t('logServiceMissing'), 'error');
            }
        } catch (_) {}

        await runSystemRefresh({ showLoadingSpinner: false });
    }
}

window.__onLangChange = async () => {
    if (hasBackend()) {
        try {
            const info = await invoke('get_system_info');
            applyHardwareInfo(info);
        } catch (_) {}
    }
    await refreshConfigList();
    await refreshPresetGrid();
    serverBlockerApi?.render();
};

window.__renderServerBlocker = () => serverBlockerApi?.render();

window.addEventListener('DOMContentLoaded', async () => {
    await initTauriAPI();
    initElements();
    serverBlockerApi = await setupServerBlocker();
    setupLangSwitcher();
    await animateLoadingScreen();
    setupEventListeners();
    await initializeApp();
});
