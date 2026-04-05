// Global variables
let invoke;
let isTauri = false;
let currentWindow;

// Element references - will be initialized after DOM loads
let refreshBtn, installBtn, uninstallBtn, verifyBtn, launchBtn;
let browseBtn, browseDirBtn, gameDir, targetPath, ifeoResult, launchResult;
let logContainer, currentTimeEl;
let cpuInfo, gpuInfo, ramFill, ramTotal, ramAvailable, heapSize, ifeoStatus;

// Title bar button references
let btnMinimize, btnMaximize, btnClose;
let btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading;

// Loading screen elements
let loadingScreen, loadingProgress, loadingStatus;

// Initialize Tauri API
async function initTauriAPI() {
    try {
        const tauriCore = await import('@tauri-apps/api/core');
        const tauriWindow = await import('@tauri-apps/api/window');
        invoke = tauriCore.invoke;
        currentWindow = tauriWindow.getCurrentWindow();
        isTauri = true;
        console.log('Tauri API loaded successfully');
    } catch (e) {
        console.error('Tauri API not available:', e);
        invoke = async (cmd, args) => {
            throw new Error(`Tauri command '${cmd}' not available in browser mode`);
        };
        currentWindow = null;
    }
}

// Initialize element references
function initElements() {
    refreshBtn = document.getElementById('refresh-btn');
    installBtn = document.getElementById('install-btn');
    uninstallBtn = document.getElementById('uninstall-btn');
    verifyBtn = document.getElementById('verify-btn');
    launchBtn = document.getElementById('launch-btn');
    browseBtn = document.getElementById('browse-btn');
    browseDirBtn = document.getElementById('browse-dir-btn');
    gameDir = document.getElementById('game-dir');
    targetPath = document.getElementById('target-path');
    ifeoResult = document.getElementById('ifeo-result');
    launchResult = document.getElementById('launch-result');
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
}

// Helper function to setup window control buttons
function setupWindowControls(minimizeBtn, maximizeBtn, closeBtn) {
    if (minimizeBtn) {
        minimizeBtn.addEventListener('click', async () => {
            if (currentWindow) {
                await currentWindow.minimize();
            }
        });
    }

    if (maximizeBtn) {
        maximizeBtn.addEventListener('click', async () => {
            if (currentWindow) {
                const isMaximized = await currentWindow.isMaximized();
                if (isMaximized) {
                    await currentWindow.unmaximize();
                } else {
                    await currentWindow.maximize();
                }
            }
        });
    }

    if (closeBtn) {
        closeBtn.addEventListener('click', async () => {
            if (currentWindow) {
                await currentWindow.close();
            }
        });
    }
}

// Loading Screen Animation
const loadingMessages = [
    'Detecting hardware...',
    'Analyzing CPU configuration...',
    'Scanning memory modules...',
    'Detecting graphics adapter...',
    'Calculating optimal JVM parameters...',
    'Initializing system...',
    'Preparing interface...',
    'Almost ready...'
];

function animateLoadingScreen() {
    return new Promise((resolve) => {
        const totalDuration = 5000; // 5 seconds
        const messageInterval = totalDuration / loadingMessages.length;
        const progressStep = 100 / (totalDuration / 50); // Update every 50ms
        
        let progress = 0;
        let messageIndex = 0;
        
        // Update status messages
        const messageTimer = setInterval(() => {
            if (messageIndex < loadingMessages.length) {
                loadingStatus.textContent = loadingMessages[messageIndex];
                messageIndex++;
            }
        }, messageInterval);
        
        // Update progress bar
        const progressTimer = setInterval(() => {
            progress += progressStep;
            if (progress >= 100) {
                progress = 100;
                clearInterval(progressTimer);
                clearInterval(messageTimer);
                
                // Hide loading screen
                setTimeout(() => {
                    loadingScreen.classList.add('hidden');
                    resolve();
                }, 200);
            }
            loadingProgress.style.width = progress + '%';
        }, 50);
    });
}

// Helper functions
function updateClock() {
    const now = new Date();
    const date = now.toISOString().split('T')[0];
    const time = now.toTimeString().split(' ')[0];
    currentTimeEl.textContent = `${date} // ${time}`;
}

function getTimestamp() {
    const now = new Date();
    const h = String(now.getHours()).padStart(2, '0');
    const m = String(now.getMinutes()).padStart(2, '0');
    const s = String(now.getSeconds()).padStart(2, '0');
    return `[${h}:${m}:${s}]`;
}

function addLog(message, type = '') {
    if (!logContainer) return;
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    entry.innerHTML = `
        <span class="log-time">${getTimestamp()}</span>
        <span class="log-arrow">></span>
        <span class="log-text ${type}">${message}</span>
    `;
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
    if (loading) {
        refreshBtn.classList.add('spinning');
        refreshBtn.disabled = true;
    } else {
        refreshBtn.classList.remove('spinning');
        refreshBtn.disabled = false;
    }
}

// Setup all event listeners
function setupEventListeners() {
    // Setup window controls
    setupWindowControls(btnMinimize, btnMaximize, btnClose);
    setupWindowControls(btnMinimizeLoading, btnMaximizeLoading, btnCloseLoading);

    // System info refresh
    refreshBtn.addEventListener('click', async () => {
        setRefreshLoading(true);
        addLog('Detecting system hardware...', 'info');

        try {
            const info = await invoke('get_system_info');
            console.log('Real system info:', info);

            cpuInfo.innerHTML = `
                <div class="hw-main">${info.cpu_name}</div>
                <div class="hw-sub">${info.cpu_cores} Cores / ${info.cpu_cores * 2} Threads</div>
            `;

            gpuInfo.innerHTML = `
                <div class="hw-main">${info.gpu_name}</div>
                <div class="hw-sub">Graphics Adapter</div>
            `;

            const usedPercent = ((info.total_ram_gb - info.free_ram_gb) / info.total_ram_gb * 100).toFixed(0);
            ramFill.style.width = usedPercent + '%';
            ramTotal.textContent = info.total_ram_gb.toFixed(2) + ' GB';
            ramAvailable.textContent = info.free_ram_gb.toFixed(2) + ' GB Available';

            const heapMB = info.suggested_heap_gb * 1024;
            heapSize.textContent = heapMB + ' MB';

            addLog(`System detected: ${info.cpu_name}, ${info.gpu_name}, ${info.total_ram_gb.toFixed(2)}GB RAM (${info.free_ram_gb.toFixed(2)}GB free)`, 'success');
            addLog(`Optimal heap size: ${heapMB}MB (${info.suggested_heap_gb}GB)`, 'info');

            if (info.large_pages) {
                addLog(`Large pages enabled: ${info.large_page_size_mb}MB`, 'success');
            } else {
                addLog('Large pages: not supported', 'info');
            }
        } catch (error) {
            addLog(`System detection failed: ${error}`, 'error');
            console.error('System detection error:', error);
        } finally {
            setRefreshLoading(false);
        }
    });

    // IFEO Install
    installBtn.addEventListener('click', async () => {
        setLoading(installBtn, true);
        addLog('Installing IFEO registry hook...', 'info');

        try {
            const result = await invoke('install_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            ifeoStatus.className = 'status-badge active';
            ifeoStatus.innerHTML = '<span class="status-dot active"></span> ACTIVE';
            addLog('IFEO installation successful', 'success');
            addLog('Wrapper will now intercept game launches', 'info');
        } catch (error) {
            ifeoResult.textContent = error;
            ifeoResult.className = 'ifeo-result error';
            addLog(`IFEO installation failed: ${error}`, 'error');
            console.error('IFEO install error:', error);
        } finally {
            setLoading(installBtn, false);
        }
    });

    // IFEO Uninstall
    uninstallBtn.addEventListener('click', async () => {
        setLoading(uninstallBtn, true);
        addLog('Removing IFEO registry hook...', 'info');

        try {
            const result = await invoke('uninstall_ifeo');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result success';
            ifeoStatus.className = 'status-badge inactive';
            ifeoStatus.innerHTML = '<span class="status-dot inactive"></span> INACTIVE';
            addLog('IFEO removal successful', 'success');
        } catch (error) {
            ifeoResult.textContent = error;
            ifeoResult.className = 'ifeo-result error';
            addLog(`IFEO removal failed: ${error}`, 'error');
            console.error('IFEO uninstall error:', error);
        } finally {
            setLoading(uninstallBtn, false);
        }
    });

    // IFEO Verify
    verifyBtn.addEventListener('click', async () => {
        setLoading(verifyBtn, true);
        addLog('Checking IFEO installation status...', 'info');

        try {
            const result = await invoke('check_status');
            ifeoResult.textContent = result;
            ifeoResult.className = 'ifeo-result info';

            const isActive = !result.includes('Not installed') && !result.includes('not installed');
            if (isActive) {
                ifeoStatus.className = 'status-badge active';
                ifeoStatus.innerHTML = '<span class="status-dot active"></span> ACTIVE';
                addLog('IFEO wrapper is active', 'success');
            } else {
                ifeoStatus.className = 'status-badge inactive';
                ifeoStatus.innerHTML = '<span class="status-dot inactive"></span> INACTIVE';
                addLog('IFEO wrapper is not installed', 'error');
            }
        } catch (error) {
            ifeoResult.textContent = error;
            ifeoResult.className = 'ifeo-result error';
            addLog(`Status check failed: ${error}`, 'error');
            console.error('IFEO status error:', error);
        } finally {
            setLoading(verifyBtn, false);
        }
    });

    // Launch game
    launchBtn.addEventListener('click', async () => {
        const dir = gameDir.value.trim();
        const exe = targetPath.value.trim();

        if (!dir && !exe) {
            launchResult.textContent = 'Please select game directory or executable';
            launchResult.className = 'launch-result error';
            addLog('Launch failed: no target specified', 'error');
            return;
        }

        let fullPath;
        if (dir && exe) {
            if (exe.includes(':\\') || exe.includes('/')) {
                fullPath = exe;
            } else {
                fullPath = `${dir}\\${exe}`;
            }
        } else if (dir) {
            fullPath = dir;
        } else {
            fullPath = exe;
        }

        fullPath = fullPath.replace(/\//g, '\\');

        const isIFEoActive = ifeoStatus.classList.contains('active');
        if (!isIFEoActive) {
            addLog('WARNING: IFEO is not active. Game may not launch with optimizations.', 'error');
        }

        setLoading(launchBtn, true);
        addLog(`Launching game: ${fullPath}`, 'info');

        try {
            const result = await invoke('launch_game', {
                target: fullPath,
                args: []
            });

            launchResult.textContent = result;
            launchResult.className = 'launch-result success';
            addLog('Game process started successfully', 'success');
        } catch (error) {
            launchResult.textContent = error;
            launchResult.className = 'launch-result error';
            addLog(`Game launch failed: ${error}`, 'error');
            console.error('Game launch error:', error);
        } finally {
            setLoading(launchBtn, false);
        }
    });

    // Browse for executable
    browseBtn.addEventListener('click', async () => {
        try {
            if (isTauri) {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({
                    multiple: false,
                    directory: false,
                    filters: [{
                        name: 'Game Executable',
                        extensions: ['exe']
                    }]
                });

                if (selected) {
                    const path = typeof selected === 'string' ? selected :
                                (Array.isArray(selected) ? selected[0] : null);
                    if (path) {
                        targetPath.value = path;
                        addLog(`Selected game executable: ${path}`, 'info');
                    }
                }
            }
        } catch (error) {
            addLog(`File selection failed: ${error}`, 'error');
            console.error('Browse error:', error);
        }
    });

    // Browse for directory
    browseDirBtn.addEventListener('click', async () => {
        try {
            if (isTauri) {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({
                    multiple: false,
                    directory: true
                });

                if (selected) {
                    const path = typeof selected === 'string' ? selected :
                                (Array.isArray(selected) ? selected[0] : null);
                    if (path) {
                        gameDir.value = path;
                        addLog(`Selected game directory: ${path}`, 'info');
                    }
                }
            }
        } catch (error) {
            addLog(`Directory selection failed: ${error}`, 'error');
            console.error('Browse directory error:', error);
        }
    });
}

// Main initialization function
function initializeApp() {
    // Initialize clock
    updateClock();
    setInterval(updateClock, 1000);

    // Initial system info load
    addLog('Application started successfully', 'success');

    // Auto-detect system on startup
    setTimeout(() => {
        refreshBtn.click();
    }, 500);

    // IFEO status check on startup
    setTimeout(async () => {
        try {
            const result = await invoke('check_status');
            const isActive = !result.includes('Not installed') && !result.includes('not installed');
            
            if (isActive) {
                ifeoStatus.className = 'status-badge active';
                ifeoStatus.innerHTML = '<span class="status-dot active"></span> ACTIVE';
                addLog('IFEO is active on startup', 'success');
            } else {
                ifeoStatus.className = 'status-badge inactive';
                ifeoStatus.innerHTML = '<span class="status-dot inactive"></span> INACTIVE';
                addLog('IFEO is not installed on startup', 'info');
            }
        } catch (error) {
            console.error('Startup IFEO check failed:', error);
        }
    }, 1000);
}

// Start everything when DOM is loaded
window.addEventListener('DOMContentLoaded', async () => {
    // Initialize Tauri API
    await initTauriAPI();
    
    // Initialize element references
    initElements();
    
    // Start loading animation
    await animateLoadingScreen();
    
    console.log('Loading screen completed');
    
    // Setup event listeners
    setupEventListeners();
    
    // Initialize the app
    initializeApp();
});
