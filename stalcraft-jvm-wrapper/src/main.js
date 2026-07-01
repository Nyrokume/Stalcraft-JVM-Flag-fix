// main.js — slim GUI (Go cli parity)

let invoke;
let isTauri = false;
let currentWindow;

let refreshBtn, installBtn, uninstallBtn, verifyBtn, ifeoResult;
let logContainer, currentTimeEl;
let cpuInfo, gpuInfo, ramFill, ramTotal, ramAvailable, heapSize, ifeoStatus;
let memTier, memSpeed;
let btnMinimize, btnMaximize, btnClose;
let btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading;
let loadingScreen, loadingProgress, loadingStatus;
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
    'Detecting hardware...', 'Reading SMBIOS memory speed...', 'Scanning memory modules...',
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

function isIfeoStatusActive(statusText) {
    const lines = (statusText || '').split(/\r?\n/).map((s) => s.trim()).filter(Boolean);
    if (lines.length === 0) return false;
    return !lines.some((line) => /: not installed$/i.test(line));
}

function logIfeoRegistryLines(statusText, heading) {
    const t = (statusText || '').trim();
    if (!t) return;
    addLog(heading, 'info');
    for (const line of t.split(/\r?\n/)) {
        if (line.trim()) addLog(`  ${line.trim()}`, 'info');
    }
}

async function pullWrapperLogToUi(maxLines = 280) {
    if (!isTauri) return;
    try {
        const tail = await invoke('read_wrapper_log_tail', { maxLines });
        const text = tail != null ? String(tail).trim() : '';
        if (!text) return;
        addLog('── logs/wrapper.log (tail) ──', 'info');
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
        button.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px;animation:spin 0.8s linear infinite"><path d="M21 12a9 9 0 11-6.219-8.56"/></svg> LOADING`;
    } else if (button.dataset.originalHTML) {
        button.innerHTML = button.dataset.originalHTML;
    }
}

function setRefreshLoading(loading) {
    loading ? (refreshBtn.classList.add('spinning'), refreshBtn.disabled = true)
             : (refreshBtn.classList.remove('spinning'), refreshBtn.disabled = false);
}

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
    } catch (e) {
        console.error('Failed to load config list:', e);
    }
}

async function runSystemRefresh({ showLoadingSpinner = true } = {}) {
    if (!isTauri || !refreshBtn) return;
    if (showLoadingSpinner) setRefreshLoading(true);
    addLog('Detecting system hardware...', 'info');
    try {
        const info = await invoke('get_system_info');

        const esc = (s) => String(s)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;');
        cpuInfo.innerHTML = `<div class="hw-main">${esc(info.cpu_name)}</div><div class="hw-sub">${info.cpu_cores} Cores / ${info.cpu_threads} Threads${info.l3_cache_mb > 0 ? ` • L3 ${info.l3_cache_mb} MB` : ''}</div>`;
        gpuInfo.innerHTML = `<div class="hw-main">${esc(info.gpu_name)}</div><div class="hw-sub">Graphics Adapter</div>`;

        const usedPct = info.total_ram_gb > 0
            ? ((info.total_ram_gb - info.free_ram_gb) / info.total_ram_gb * 100).toFixed(0)
            : '0';
        ramFill.style.width = usedPct + '%';
        ramTotal.textContent = info.total_ram_gb.toFixed(2) + ' GB';
        ramAvailable.textContent = info.free_ram_gb.toFixed(2) + ' GB Available';
        heapSize.textContent = (info.suggested_heap_gb * 1024) + ' MB';

        if (memTier) memTier.textContent = `Tier: ${info.mem_tier}`;
        if (memSpeed) {
            memSpeed.textContent = info.mem_speed_mts > 0
                ? `${info.mem_speed_mts} MT/s configured`
                : 'Speed unknown (mid tier fallback)';
        }

        addLog(`System: ${info.cpu_name}, ${info.total_ram_gb.toFixed(1)}GB RAM, mem=${info.mem_tier}`, 'success');
        addLog(`Heap: ${info.suggested_heap_gb}GB, config: ${info.active_config || 'default'}`, 'info');
        if (info.large_pages) addLog('Large pages: available', 'success');

        await refreshConfigList();
        await pullWrapperLogToUi();
    } catch (e) {
        addLog(`System detection failed: ${e}`, 'error');
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
        addLog('Installing IFEO hook...', 'info');
        try {
            const result = await invoke('install_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            ifeoStatus.className = 'status-badge active';
            ifeoStatus.innerHTML = '<span class="status-dot active"></span> ACTIVE';
            addLog('IFEO installed successfully', 'success');
            try {
                const st = await invoke('check_status');
                logIfeoRegistryLines(st, 'IFEO registry (verify after install):');
            } catch (_) {}
            await pullWrapperLogToUi();
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
            const isActive = isIfeoStatusActive(result);
            ifeoStatus.className = `status-badge ${isActive ? 'active' : 'inactive'}`;
            ifeoStatus.innerHTML = `<span class="status-dot ${isActive ? 'active' : 'inactive'}"></span> ${isActive ? 'ACTIVE' : 'INACTIVE'}`;
            logIfeoRegistryLines(result, 'IFEO registry (per target):');
            addLog(isActive ? 'IFEO: all targets configured' : 'IFEO: at least one target missing', isActive ? 'success' : 'error');
        } catch (e) {
            ifeoResult.textContent = e;
            ifeoResult.className = 'ifeo-result error';
            addLog(`Status check failed: ${e}`, 'error');
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
            const active = await invoke('get_active_config');
            fillConfigEditor(active.name, active.config);
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
            const active = await invoke('get_active_config');
            fillConfigEditor(active.name, active.config);
        } catch (e) {
            addLog(`Regenerate failed: ${e}`, 'error');
        } finally {
            setLoading(regenerateConfigBtn, false);
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
            const res = await invoke('load_config_by_name', { name });
            fillConfigEditor(res.name, res.config);
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
    addLog('Application started', 'success');

    if (isTauri) {
        try {
            const result = await invoke('check_status');
            const isActive = isIfeoStatusActive(result);
            ifeoStatus.className = `status-badge ${isActive ? 'active' : 'inactive'}`;
            ifeoStatus.innerHTML = `<span class="status-dot ${isActive ? 'active' : 'inactive'}"></span> ${isActive ? 'ACTIVE' : 'INACTIVE'}`;
            logIfeoRegistryLines(result, 'IFEO registry (per target):');
        } catch (_) {}

        await runSystemRefresh({ showLoadingSpinner: false });
        try {
            const active = await invoke('get_active_config');
            fillConfigEditor(active.name, active.config);
        } catch (_) {}
    }
}

window.addEventListener('DOMContentLoaded', async () => {
    await initTauriAPI();
    initElements();
    await animateLoadingScreen();
    setupEventListeners();
    await initializeApp();
});
