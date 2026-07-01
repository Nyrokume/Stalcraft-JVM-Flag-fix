/** Browser-only dev mocks (Vite without Tauri webview). */

const MOCK_SYSTEM = {
    cpu_name: 'AMD Ryzen 7 5800X (dev mock)',
    gpu_name: 'NVIDIA GeForce RTX 3070',
    total_ram_gb: 32,
    free_ram_gb: 18.5,
    cpu_cores: 8,
    cpu_threads: 16,
    l3_cache_mb: 32,
    mem_speed_mts: 3200,
    mem_tier: 'fast',
    has_big_cache: true,
    large_pages: false,
    large_page_size_mb: 0,
    suggested_heap_gb: 8,
    active_config: 'default.json',
    active_config_exists: true,
};

const MOCK_STATUS = `service.exe: ok (dev mock)
stalcraft.exe: ok
stalcraftw.exe: ok
stalzone.exe: ok
stalzonew.exe: ok
java.exe: ok
javaw.exe: ok`;

export function createDevMockInvoke() {
    return async (cmd, args = {}) => {
        switch (cmd) {
            case 'get_system_info':
                return { ...MOCK_SYSTEM };
            case 'check_status':
                return MOCK_STATUS;
            case 'list_configs':
                return {
                    names: ['default.json', '8khz.json'],
                    active: 'default.json',
                    active_exists: true,
                };
            case 'read_wrapper_log_tail':
                return '[dev] mock wrapper.log\n[dev] jvm_mode=MOCK';
            case 'install_ifeo':
                return MOCK_STATUS;
            case 'uninstall_ifeo':
                return 'IFEO uninstalled (dev mock).';
            case 'select_config':
                return `Active config set to: ${args.name ?? 'default.json'}`;
            case 'regenerate_config':
                return 'default.json regenerated (dev mock).';
            default:
                throw new Error(`Unknown mock command: ${cmd}`);
        }
    };
}

export function showDevBanner() {
    if (document.getElementById('dev-browser-banner')) return;
    const el = document.createElement('div');
    el.id = 'dev-browser-banner';
    el.className = 'dev-browser-banner';
    el.textContent = 'Browser dev · mock data · npm run dev for full Tauri';
    document.body.appendChild(el);
}
