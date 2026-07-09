/** Browser-only dev mocks (Vite without Tauri webview). */

import { mockPingMs } from './server-blocker-logic.js';

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
stalzone.exe: ok
stalzonew.exe: ok
stalcraft.exe: ok
stalcraftw.exe: ok
java.exe: ok
javaw.exe: ok`;

const MOCK_BLOCKING = { active: false };

const MOCK_EXAMPLES = [
    'weak',
    'medium',
    'max',
    'throughput_v110',
    'x3d_v110',
    'balanced_mid',
    'slow_ddr',
    'removed_fast_ddr',
    '8khz',
];
const MOCK_IMPORTED = new Set(['default', '8khz']);

export function createDevMockInvoke() {
    return async (cmd, args = {}) => {
        switch (cmd) {
            case 'get_system_info':
                return { ...MOCK_SYSTEM };
            case 'check_status':
                return MOCK_STATUS;
            case 'list_configs':
                return {
                    names: [...MOCK_IMPORTED],
                    active: 'default',
                    active_exists: true,
                };
            case 'list_examples':
                return { names: [...MOCK_EXAMPLES] };
            case 'import_example_config': {
                const name = args.name ?? '';
                if (!MOCK_EXAMPLES.includes(name)) {
                    throw new Error(`example not found: ${name}`);
                }
                MOCK_IMPORTED.add(name);
                return `Imported preset: ${name}.json`;
            }
            case 'read_wrapper_log_tail':
                return '[dev] mock wrapper.log\n[dev] jvm_mode=MOCK';
            case 'install_ifeo':
                return MOCK_STATUS;
            case 'uninstall_ifeo':
                return 'IFEO uninstalled (dev mock).';
            case 'select_config':
                return `Active config set to: ${args.name ?? 'default'}`;
            case 'regenerate_config':
                return 'default.json regenerated (dev mock).';
            case 'ping_servers':
                return (args.targets ?? []).map((t) => ({
                    id: t.id,
                    ms: mockPingMs(t.host, t.port),
                }));
            case 'start_server_blocking':
                MOCK_BLOCKING.active = true;
                MOCK_BLOCKING.ips = (args.ips ?? []).length;
                return `Blocked ${MOCK_BLOCKING.ips} IP(s) (dev mock)`;
            case 'stop_server_blocking':
                MOCK_BLOCKING.active = false;
                MOCK_BLOCKING.ips = 0;
                return 'Removed firewall rules (dev mock)';
            case 'server_blocking_active':
                return {
                    active: MOCK_BLOCKING.active,
                    rule_count: MOCK_BLOCKING.active ? (MOCK_BLOCKING.ips ?? 0) * 2 : 0,
                };
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
