// main.js — полный функционал включая управление конфигами
let invoke;
let isTauri = false;
let currentWindow;

let refreshBtn, installBtn, uninstallBtn, verifyBtn;
let browseBtn, browseDirBtn, gameDir, targetPath, ifeoResult;
let logContainer, currentTimeEl;
let cpuInfo, gpuInfo, ramFill, ramTotal, ramAvailable, heapSize, ifeoStatus;
let btnMinimize, btnMaximize, btnClose;
let btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading;
let loadingScreen, loadingProgress, loadingStatus;
// config UI
let configSelect, configActiveLabel, regenerateConfigBtn, selectConfigBtn;

async function initTauriAPI() {
    try {
        const tauriCore = await import('@tauri-apps/api/core');
        const tauriWindow = await import('@tauri-apps/api/window');
        invoke = tauriCore.invoke;
        currentWindow = tauriWindow.getCurrentWindow();
        isTauri = true;
    } catch (e) {
        console.error('Tauri API not available:', e);
        invoke = async (cmd) => {
            throw new Error(`Tauri command '${cmd}' not available in browser mode`);
        };
        currentWindow = null;
    }
}

function initElements() {
    refreshBtn = document.getElementById('refresh-btn');
    installBtn = document.getElementById('install-btn');
    uninstallBtn = document.getElementById('uninstall-btn');
    verifyBtn = document.getElementById('verify-btn');
    browseBtn = document.getElementById('browse-btn');
    browseDirBtn = document.getElementById('browse-dir-btn');
    gameDir = document.getElementById('game-dir');
    targetPath = document.getElementById('target-path');
    ifeoResult = document.getElementById('ifeo-result');
    logContainer = document.getElementById('log-container');
    currentTimeEl = document.getElementById('current-time');
    cpuInfo = document.getElementById('cpu-info');
    gpuInfo = document.getElementById('gpu-info');
    ramFill = document.getElementById('ram-fill');
    ramTotal = document.getElementById('ram-total');
    ramAvailable = document.getElementById('ram-available');
    heapSize = document.getElementById('heap-size');
    ifeoStatus = document.getElementById('ifeo-status');
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

const loadingMessages = [
    'Detecting hardware...', 'Analyzing CPU configuration...', 'Scanning memory modules...',
    'Detecting L3 cache size...', 'Calculating optimal JVM parameters...',
    'Loading config profiles...', 'Preparing interface...', 'Almost ready...'
];

function animateLoadingScreen() {
    return new Promise((resolve) => {
        const totalDuration = 5000;
        const messageInterval = totalDuration / loadingMessages.length;
        const progressStep = 100 / (totalDuration / 50);
        let progress = 0, messageIndex = 0;

        const messageTimer = setInterval(() => {
            if (messageIndex < loadingMessages.length) {
                loadingStatus.textContent = loadingMessages[messageIndex++];
            }
        }, messageInterval);

        const progressTimer = setInterval(() => {
            progress += progressStep;
            if (progress >= 100) {
                progress = 100;
                clearInterval(progressTimer);
                clearInterval(messageTimer);
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

function addLog(message, type = '') {
    if (!logContainer) return;
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    entry.innerHTML = `<span class="log-time">${getTimestamp()}</span><span class="log-arrow">></span><span class="log-text ${type}">${message}</span>`;
    logContainer.appendChild(entry);
    logContainer.scrollTop = logContainer.scrollHeight;
}

function setLoading(button, loading) {
    button.disabled = loading;
    if (loading) {
        button.dataset.originalHTML = button.innerHTML;
        button.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px;animation:spin 0.8s linear infinite"><path d="M21 12a9 9 0 11-6.219-8.56"/></svg> LOADING`;
    } else if (button.dataset.originalHTML) {
        button.innerHTML = button.dataset.originalHTML;
    }
}

function setRefreshLoading(loading) {
    loading ? (refreshBtn.classList.add('spinning'), refreshBtn.disabled = true)
             : (refreshBtn.classList.remove('spinning'), refreshBtn.disabled = false);
}

// ─── Config management ────────────────────────────────────────────────────────

const PRESET_STEM_BY_ID = {
    balanced: 'preset_balanced',
    latency: 'preset_latency',
    throughput: 'preset_throughput',
    nursery: 'preset_nursery',
    conservative: 'preset_conservative',
    low_ram: 'preset_low_ram',
    streaming: 'preset_streaming',
    power: 'preset_power',
};

function setConfigEditorError(msg) {
    const el = document.getElementById('config-editor-error');
    if (el) el.textContent = msg || '';
}

function fillConfigEditor(name, cfg) {
    const ta = document.getElementById('config-json-editor');
    if (!ta) return;
    ta.value = JSON.stringify(cfg, null, 2);
    ta.dataset.loadedProfile = name || '';
    setConfigEditorError('');
}

function parseConfigEditorJson() {
    const ta = document.getElementById('config-json-editor');
    if (!ta || !ta.value.trim()) throw new Error('Editor is empty');
    const raw = JSON.parse(ta.value);
    if (raw && typeof raw === 'object' && raw.config && typeof raw.config === 'object') {
        return raw.config;
    }
    return raw;
}

function updatePresetChipActive(activeName) {
    const grid = document.getElementById('config-preset-grid');
    if (!grid) return;
    for (const chip of grid.querySelectorAll('.config-preset-chip')) {
        const stem = PRESET_STEM_BY_ID[chip.dataset.preset];
        chip.classList.toggle('active', Boolean(stem && activeName === stem));
    }
}

function setPresetChipsDisabled(disabled) {
    document.querySelectorAll('.config-preset-chip').forEach((btn) => {
        btn.disabled = disabled;
    });
}

async function syncHeapDisplay() {
    if (!heapSize || !isTauri) return;
    try {
        const info = await invoke('get_system_info');
        heapSize.textContent = info.suggested_heap_gb * 1024 + ' MB';
    } catch (_) {}
}

async function refreshConfigList() {
    try {
        const result = await invoke('list_configs');
        if (!configSelect) return;
        configSelect.innerHTML = '';
        const placeholder = document.createElement('option');
        placeholder.value = '';
        placeholder.textContent = 'Select profile…';
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
                    ? `Active: ${result.active}`
                    : `Active: ${result.active} (missing — will use default)`;
                configActiveLabel.className = result.active_exists ? 'config-active-label success' : 'config-active-label warning';
            } else {
                configActiveLabel.textContent = 'No active config selected';
                configActiveLabel.className = 'config-active-label';
            }
        }
        updatePresetChipActive(result.active);
    } catch (e) {
        console.error('Failed to load config list:', e);
    }
}

function setupEventListeners() {
    setupWindowControls(btnMinimize, btnMaximize, btnClose);
    setupWindowControls(btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading);

    // ─── System refresh ───────────────────────────────────────────────────────
    refreshBtn.addEventListener('click', async () => {
        setRefreshLoading(true);
        addLog('Detecting system hardware...', 'info');
        try {
            const info = await invoke('get_system_info');

            cpuInfo.innerHTML = `<div class="hw-main">${info.cpu_name}</div><div class="hw-sub">${info.cpu_cores} Cores / ${info.cpu_threads} Threads${info.l3_cache_mb > 0 ? ` • L3 ${info.l3_cache_mb} MB` : ''}${info.has_big_cache ? ' • X3D' : ''}</div>`;
            gpuInfo.innerHTML = `<div class="hw-main">${info.gpu_name}</div><div class="hw-sub">Graphics Adapter</div>`;

            const usedPct = ((info.total_ram_gb - info.free_ram_gb) / info.total_ram_gb * 100).toFixed(0);
            ramFill.style.width = usedPct + '%';
            ramTotal.textContent = info.total_ram_gb.toFixed(2) + ' GB';
            ramAvailable.textContent = info.free_ram_gb.toFixed(2) + ' GB Available';
            heapSize.textContent = (info.suggested_heap_gb * 1024) + ' MB';

            addLog(`System: ${info.cpu_name}, ${info.total_ram_gb.toFixed(1)}GB RAM, L3=${info.l3_cache_mb}MB${info.has_big_cache ? ' (X3D)' : ''}`, 'success');
            addLog(`Heap: ${info.suggested_heap_gb}GB (${info.suggested_heap_gb * 1024}MB), config: ${info.active_config || 'default'}`, 'info');
            if (info.large_pages) addLog(`Large pages: ${info.large_page_size_mb}MB`, 'success');

            await refreshConfigList();
        } catch (e) {
            addLog(`System detection failed: ${e}`, 'error');
        } finally {
            setRefreshLoading(false);
        }
    });

    // ─── IFEO ─────────────────────────────────────────────────────────────────
    installBtn.addEventListener('click', async () => {
        setLoading(installBtn, true);
        addLog('Installing IFEO hook (service.exe)...', 'info');
        try {
            const result = await invoke('install_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            ifeoStatus.className = 'status-badge active';
            ifeoStatus.innerHTML = '<span class="status-dot active"></span> ACTIVE';
            addLog('IFEO installed successfully', 'success');
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(`IFEO install failed: ${e}`, 'error');
        } finally {
            setLoading(installBtn, false);
        }
    });

    uninstallBtn.addEventListener('click', async () => {
        setLoading(uninstallBtn, true);
        addLog('Removing IFEO hook...', 'info');
        try {
            const result = await invoke('uninstall_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            ifeoStatus.className = 'status-badge inactive';
            ifeoStatus.innerHTML = '<span class="status-dot inactive"></span> INACTIVE';
            addLog('IFEO removed', 'success');
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(`IFEO remove failed: ${e}`, 'error');
        } finally {
            setLoading(uninstallBtn, false);
        }
    });

    verifyBtn.addEventListener('click', async () => {
        setLoading(verifyBtn, true);
        addLog('Checking IFEO status...', 'info');
        try {
            const result = await invoke('check_status');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result info';
            const isActive = !result.includes('Not installed') && !result.includes('not installed');
            ifeoStatus.className = `status-badge ${isActive ? 'active' : 'inactive'}`;
            ifeoStatus.innerHTML = `<span class="status-dot ${isActive ? 'active' : 'inactive'}"></span> ${isActive ? 'ACTIVE' : 'INACTIVE'}`;
            addLog(isActive ? 'IFEO is active' : 'IFEO not installed', isActive ? 'success' : 'error');
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(`Status check failed: ${e}`, 'error');
        } finally {
            setLoading(verifyBtn, false);
        }
    });

    // ─── Browse ───────────────────────────────────────────────────────────────
    browseBtn?.addEventListener('click', async () => {
        if (!isTauri) return;
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({ multiple: false, directory: false, filters: [{ name: 'Executable', extensions: ['exe'] }] });
            if (selected) {
                const path = typeof selected === 'string' ? selected : selected[0];
                if (path) { targetPath.value = path; addLog(`Executable: ${path}`, 'info'); }
            }
        } catch (e) { addLog(`Browse failed: ${e}`, 'error'); }
    });

    browseDirBtn?.addEventListener('click', async () => {
        if (!isTauri) return;
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({ multiple: false, directory: true });
            if (selected) {
                const path = typeof selected === 'string' ? selected : selected[0];
                if (path) {
                    gameDir.value = path;
                    addLog(`Directory: ${path}`, 'info');
                    try { await invoke('save_game_dir', { gameDir: path }); } catch (_) {}
                }
            }
        } catch (e) { addLog(`Browse failed: ${e}`, 'error'); }
    });

    gameDir?.addEventListener('change', async () => {
        if (gameDir.value.trim() && isTauri) {
            try { await invoke('save_game_dir', { gameDir: gameDir.value.trim() }); } catch (_) {}
        }
    });

    // ─── Config management ────────────────────────────────────────────────────
    selectConfigBtn?.addEventListener('click', async () => {
        if (!configSelect) return;
        const name = configSelect.value;
        if (!name) return;
        setLoading(selectConfigBtn, true);
        try {
            const result = await invoke('select_config', { name });
            addLog(result, 'success');
            await refreshConfigList();
            await syncHeapDisplay();
            try {
                const active = await invoke('get_active_config');
                fillConfigEditor(active.name, active.config);
            } catch (_) {}
        } catch (e) {
            addLog(`Config select failed: ${e}`, 'error');
        } finally {
            setLoading(selectConfigBtn, false);
        }
    });

    regenerateConfigBtn?.addEventListener('click', async () => {
        setLoading(regenerateConfigBtn, true);
        addLog('Regenerating default config for current hardware...', 'info');
        try {
            const result = await invoke('regenerate_config');
            addLog(result, 'success');
            await refreshConfigList();
            await syncHeapDisplay();
            try {
                const active = await invoke('get_active_config');
                fillConfigEditor(active.name, active.config);
            } catch (_) {}
        } catch (e) {
            addLog(`Regenerate failed: ${e}`, 'error');
        } finally {
            setLoading(regenerateConfigBtn, false);
        }
    });

    document.getElementById('config-preset-grid')?.addEventListener('click', async (ev) => {
        const chip = ev.target.closest('.config-preset-chip');
        if (!chip || chip.disabled) return;
        const preset = chip.dataset.preset;
        if (!preset) return;
        setPresetChipsDisabled(true);
        addLog(`Applying preset: ${preset}…`, 'info');
        try {
            const result = await invoke('apply_config_preset', { preset });
            addLog(result, 'success');
            await refreshConfigList();
            await syncHeapDisplay();
            try {
                const active = await invoke('get_active_config');
                fillConfigEditor(active.name, active.config);
            } catch (_) {}
        } catch (e) {
            addLog(`Preset failed: ${e}`, 'error');
        } finally {
            setPresetChipsDisabled(false);
        }
    });

    document.getElementById('config-editor-load-active')?.addEventListener('click', async () => {
        if (!isTauri) return;
        try {
            const res = await invoke('get_active_config');
            fillConfigEditor(res.name, res.config);
            addLog(`Editor: loaded active profile "${res.name}"`, 'info');
        } catch (e) {
            setConfigEditorError(String(e));
            addLog(`Load active config failed: ${e}`, 'error');
        }
    });

    document.getElementById('config-editor-load-selected')?.addEventListener('click', async () => {
        if (!isTauri || !configSelect?.value) {
            setConfigEditorError('Select a profile in the list first.');
            return;
        }
        const name = configSelect.value;
        try {
            const res = await invoke('load_config_by_name', { name });
            fillConfigEditor(res.name, res.config);
            addLog(`Editor: loaded "${name}"`, 'info');
        } catch (e) {
            setConfigEditorError(String(e));
            addLog(`Load config failed: ${e}`, 'error');
        }
    });

    document.getElementById('config-editor-save')?.addEventListener('click', async () => {
        if (!isTauri || !configSelect?.value) {
            setConfigEditorError('Select a profile name in the list (Save overwrites that file).');
            return;
        }
        const name = configSelect.value;
        const btn = document.getElementById('config-editor-save');
        if (!btn) return;
        setLoading(btn, true);
        try {
            const cfg = parseConfigEditorJson();
            const result = await invoke('save_config', { name, cfg });
            addLog(result, 'success');
            await refreshConfigList();
            await syncHeapDisplay();
            try {
                const res = await invoke('load_config_by_name', { name });
                fillConfigEditor(res.name, res.config);
            } catch (_) {}
            setConfigEditorError('');
        } catch (e) {
            const msg = e instanceof SyntaxError ? `Invalid JSON: ${e.message}` : String(e);
            setConfigEditorError(msg);
            addLog(`Save config failed: ${msg}`, 'error');
        } finally {
            setLoading(btn, false);
        }
    });
}

async function initializeApp() {
    updateClock();
    setInterval(updateClock, 1000);

    if (isTauri) {
        try {
            const savedDir = await invoke('load_game_dir');
            if (savedDir && gameDir) { gameDir.value = savedDir; addLog(`Loaded saved directory: ${savedDir}`, 'info'); }
        } catch (_) {}
        try {
            await refreshConfigList();
            try {
                const active = await invoke('get_active_config');
                fillConfigEditor(active.name, active.config);
            } catch (_) {}
        } catch (_) {}
    }

    addLog('Application started', 'success');
    setTimeout(() => refreshBtn.click(), 500);

    setTimeout(async () => {
        try {
            const result = await invoke('check_status');
            const isActive = !result.includes('Not installed') && !result.includes('not installed');
            ifeoStatus.className = `status-badge ${isActive ? 'active' : 'inactive'}`;
            ifeoStatus.innerHTML = `<span class="status-dot ${isActive ? 'active' : 'inactive'}"></span> ${isActive ? 'ACTIVE' : 'INACTIVE'}`;
            addLog(`IFEO status: ${isActive ? 'active' : 'not installed'}`, isActive ? 'success' : 'info');
        } catch (_) {}
    }, 1000);
}

window.addEventListener('DOMContentLoaded', async () => {
    await initTauriAPI();
    initElements();
    await animateLoadingScreen();
    setupEventListeners();
    initializeApp();
});
