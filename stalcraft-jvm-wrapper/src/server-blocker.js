/**
 * Server Blocker UI — catalog by region, ping, auto-best, firewall blocking.
 */

import serverCatalog from './data/servers.json' with { type: 'json' };
import ruCatalogBundled from './data/servers-ru.json' with { type: 'json' };
import {
    computeSelectionAllowed,
    countHiddenServers,
    formatPingMs,
    groupByPool,
    hasResolvableBlocks,
    hostSetsEqual,
    isVeryBadPing,
    mergeAutoBlockUnacceptable,
    mergeBlockedForScope,
    mergePreferredByPoolForScope,
    mockPingMs,
    pickBestPerRegionTopN,
    pingLevelClass,
    poolsWithValidPing,
    pruneSelectionToCatalog,
    PING_ACCEPTABLE_MAX_MS,
    PING_CHUNK_SIZE,
    resolveBlockedHosts,
    shouldShowServerInMenu,
} from './server-blocker-logic.js';

const SETTINGS_KEY = 'stalcraft-sb-settings';
const SETTINGS_VERSION = 5;
const CATALOG_CACHE_KEY = 'stalcraft-sb-catalog-v1';
const RU_CATALOG_URL = 'https://backend.stalcraftx.ru/address_list?login=User';
const ROXY_BASE =
    'https://raw.githubusercontent.com/Art3mLapa/unofficial-stalzone-api/main/static/address_list';
const ROXY_REGION_URLS = {
    EU: `${ROXY_BASE}/EU.json`,
    NA: `${ROXY_BASE}/NA.json`,
    SEA: `${ROXY_BASE}/SEA.json`,
};

const DEFAULT_SETTINGS = {
    v: SETTINGS_VERSION,
    mode: 'blocklist',
    blocked: [],
    region: 'RU',
    showBlockedOnly: false,
    pings: {},
    preferredByPool: null,
};

export const FILTER_REGIONS = ['RU', 'EU', 'NA', 'SEA'];

export function mergeServerCatalogs(roxy = serverCatalog, ru = ruCatalogBundled) {
    const roxyPools = (roxy?.pools ?? []).filter((p) => p.name !== 'MSK2X');
    const ruPools = (ru?.pools ?? []).map((p) => ({ ...p, region: 'RU' }));
    return {
        mode: roxy?.mode || ru?.mode || 'roxy',
        pools: [...roxyPools, ...ruPools],
        clientToTunnelRttWeight: roxy?.clientToTunnelRttWeight ?? ru?.clientToTunnelRttWeight ?? 1,
    };
}

async function fetchJsonCatalog(url, label) {
    const res = await fetch(url, {
        headers: { Accept: 'application/json' },
        signal: AbortSignal.timeout(12_000),
    });
    if (!res.ok) throw new Error(`${label} HTTP ${res.status}`);
    const data = await res.json();
    if (!Array.isArray(data?.pools) || data.pools.length === 0) {
        throw new Error(`${label} empty pools`);
    }
    return data;
}

function stampRegion(pools, region) {
    return (pools ?? []).map((p) => ({
        name: p.name,
        region,
        tunnels: (p.tunnels ?? []).map((t) => ({
            name: t.name,
            address: String(t.address ?? ''),
        })),
    }));
}

export function loadCatalogCache() {
    try {
        const raw = localStorage.getItem(CATALOG_CACHE_KEY);
        if (!raw) return null;
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed?.roxy?.pools) || !Array.isArray(parsed?.ru?.pools)) return null;
        if (!parsed.roxy.pools.length || !parsed.ru.pools.length) return null;
        return parsed;
    } catch {
        return null;
    }
}

export function saveCatalogCache(roxy, ru) {
    try {
        localStorage.setItem(
            CATALOG_CACHE_KEY,
            JSON.stringify({
                v: 1,
                savedAt: Date.now(),
                roxy,
                ru,
            }),
        );
    } catch (err) {
        console.warn('catalog cache write failed:', err);
    }
}

export async function fetchRuCatalog() {
    try {
        return await fetchJsonCatalog(RU_CATALOG_URL, 'RU');
    } catch (err) {
        console.warn('RU catalog fetch failed, using bundled list:', err);
    }
    return ruCatalogBundled;
}

/**
 * Pull RU + EU/NA/SEA from live endpoints.
 * Falls back per-region to bundled / previous values on failure.
 */
export async function fetchAllCatalogs({
    fallbackRoxy = serverCatalog,
    fallbackRu = ruCatalogBundled,
} = {}) {
    const sources = { ru: 'bundled', eu: 'bundled', na: 'bundled', sea: 'bundled' };

    const ruPromise = fetchJsonCatalog(RU_CATALOG_URL, 'RU')
        .then((data) => {
            sources.ru = 'live';
            return {
                mode: data.mode || 'roxy',
                source: RU_CATALOG_URL,
                pools: (data.pools ?? []).map((p) => ({
                    name: p.name,
                    tunnels: (p.tunnels ?? []).map((t) => ({
                        name: t.name,
                        address: String(t.address ?? ''),
                    })),
                })),
                clientToTunnelRttWeight: data.clientToTunnelRttWeight ?? 1,
            };
        })
        .catch((err) => {
            console.warn('RU catalog fetch failed:', err);
            return fallbackRu;
        });

    const regionPromises = Object.entries(ROXY_REGION_URLS).map(async ([region, url]) => {
        try {
            const data = await fetchJsonCatalog(url, region);
            sources[region.toLowerCase()] = 'live';
            return stampRegion(data.pools, region);
        } catch (err) {
            console.warn(`${region} catalog fetch failed:`, err);
            const kept = (fallbackRoxy?.pools ?? []).filter((p) => p.region === region);
            return stampRegion(kept, region);
        }
    });

    const [ru, ...regionPools] = await Promise.all([ruPromise, ...regionPromises]);
    const roxy = {
        mode: 'roxy',
        source: `${ROXY_BASE}/{EU,NA,SEA}.json`,
        pools: regionPools.flat(),
        clientToTunnelRttWeight: 1,
    };

    const liveCount = Object.values(sources).filter((s) => s === 'live').length;
    if (liveCount > 0) {
        saveCatalogCache(roxy, ru);
    }

    return { roxy, ru, sources, liveCount };
}

function resolveFilterRegion(_poolName, catalogRegion) {
    if (catalogRegion === 'RU') return 'RU';
    return catalogRegion ?? '—';
}

export function flattenServerCatalog(catalog = mergeServerCatalogs()) {
    const pools = catalog?.pools ?? [];
    const servers = [];
    for (const pool of pools) {
        for (const tunnel of pool.tunnels ?? []) {
            const [host, portStr] = String(tunnel.address ?? ':').split(':');
            const filterRegion = resolveFilterRegion(pool.name, pool.region);
            servers.push({
                id: `${pool.name}::${tunnel.name}`,
                pool: pool.name,
                name: tunnel.name,
                address: tunnel.address,
                host,
                port: Number(portStr) || 29450,
                region: pool.region ?? '—',
                filterRegion,
            });
        }
    }
    return servers.sort((a, b) => {
        const reg = a.filterRegion.localeCompare(b.filterRegion);
        if (reg !== 0) return reg;
        const pool = a.pool.localeCompare(b.pool);
        if (pool !== 0) return pool;
        return a.name.localeCompare(b.name);
    });
}

export function getRegions(servers) {
    return FILTER_REGIONS.filter((r) => servers.some((s) => s.filterRegion === r));
}

export function loadSettings() {
    try {
        const raw = localStorage.getItem(SETTINGS_KEY);
        if (!raw) return structuredClone(DEFAULT_SETTINGS);
        const parsed = JSON.parse(raw);
        if (parsed.v !== SETTINGS_VERSION) {
            if (parsed.v === 4) {
                const region = FILTER_REGIONS.includes(parsed.region) && parsed.region !== 'ALL'
                    ? parsed.region
                    : 'RU';
                return {
                    ...DEFAULT_SETTINGS,
                    blocked: Array.isArray(parsed.blocked) ? parsed.blocked : [],
                    pings: parsed.pings && typeof parsed.pings === 'object' ? parsed.pings : {},
                    preferredByPool: parsed.preferredByPool && typeof parsed.preferredByPool === 'object'
                        ? parsed.preferredByPool
                        : null,
                    region,
                    showBlockedOnly: Boolean(parsed.showBlockedOnly),
                };
            }
            return structuredClone(DEFAULT_SETTINGS);
        }
        const region = FILTER_REGIONS.includes(parsed.region) && parsed.region !== 'ALL'
            ? parsed.region
            : 'RU';
        return {
            ...DEFAULT_SETTINGS,
            ...parsed,
            mode: 'blocklist',
            region,
            blocked: Array.isArray(parsed.blocked) ? parsed.blocked : [],
            pings: parsed.pings && typeof parsed.pings === 'object' ? parsed.pings : {},
            preferredByPool: parsed.preferredByPool && typeof parsed.preferredByPool === 'object'
                ? parsed.preferredByPool
                : null,
        };
    } catch {
        return structuredClone(DEFAULT_SETTINGS);
    }
}

export function saveSettings(settings) {
    settings.v = SETTINGS_VERSION;
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
}

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

function regionClass(region) {
    return `sb-chip--${String(region).toLowerCase()}`;
}

function regionLabel(region, t) {
    if (region === 'RU') return t('sbRegionRu');
    if (region === 'EU') return t('sbRegionEu');
    if (region === 'NA') return t('sbRegionNa');
    if (region === 'SEA') return t('sbRegionSea');
    return region;
}

export async function initServerBlocker({ t, invoke = null }) {
    const cached = loadCatalogCache();
    const initial = await fetchAllCatalogs({
        fallbackRoxy: cached?.roxy ?? serverCatalog,
        fallbackRu: cached?.ru ?? ruCatalogBundled,
    });
    let liveRoxy = initial.roxy;
    let liveRu = initial.ru;
    let ruPoolOrder = (liveRu?.pools ?? []).map((p) => p.name);
    let servers = flattenServerCatalog(mergeServerCatalogs(liveRoxy, liveRu));
    let settings = pruneSelectionToCatalog(servers, loadSettings());
    saveSettings(settings);
    let pinging = false;
    let pingProgress = null;
    let blockingActive = false;
    let blockingBusy = false;
    let appliedHosts = [];
    let wavePhase = 0;
    let waveRaf = null;

    const els = {
        pools: document.getElementById('sb-pools'),
        search: document.getElementById('sb-search'),
        showBlocked: document.getElementById('sb-show-blocked'),
        regionChips: document.getElementById('sb-region-chips'),
        blockedBadge: document.getElementById('sb-blocked-badge'),
        statusLabel: document.getElementById('sb-status-label'),
        progressViz: document.getElementById('sb-topbar-viz'),
        progressFill: document.getElementById('sb-viz-fill'),
        progressShimmer: document.getElementById('sb-viz-shimmer'),
        progressWave: document.getElementById('sb-viz-wave'),
        progressLabel: document.getElementById('sb-viz-label'),
        startBtn: document.getElementById('sb-start-btn'),
        stopBtn: document.getElementById('sb-stop-btn'),
        resetBtn: document.getElementById('sb-reset-btn'),
        pingBtn: document.getElementById('sb-ping-btn'),
        autoBestBtn: document.getElementById('sb-auto-best-btn'),
        refreshBtn: document.getElementById('sb-refresh-btn'),
    };

    if (!els.pools) return { render: () => {} };

    function isSelected(id) {
        return settings.blocked.includes(id);
    }

    function isDenied(id) {
        return isSelected(id);
    }

    function serversInRegion(region = settings.region) {
        return servers.filter((s) => s.filterRegion === region);
    }

    function filteredServers() {
        const q = (els.search?.value ?? '').trim().toLowerCase();
        return serversInRegion().filter((srv) => {
            if (!shouldShowServerInMenu(settings.pings, srv.id, { pinging })) return false;
            if (settings.showBlockedOnly && !isDenied(srv.id)) return false;
            if (!q) return true;
            const hay = `${srv.name} ${srv.pool} ${srv.address} ${srv.filterRegion}`.toLowerCase();
            return hay.includes(q);
        });
    }

    function applyAutoBlockBad(scope) {
        const result = mergeAutoBlockUnacceptable(
            'blocklist',
            scope,
            settings.pings,
            settings.blocked,
            settings.preferredByPool,
        );
        settings.blocked = result.blocked;
        settings.preferredByPool = result.preferredByPool;
        saveSettings(settings);
    }

    function poolOrderInList(regionServers) {
        const order = [];
        for (const s of regionServers) {
            if (!order.includes(s.pool)) order.push(s.pool);
        }
        return order;
    }

    function groupedByRegion(list) {
        const map = new Map();
        for (const srv of list) {
            if (!map.has(srv.filterRegion)) {
                map.set(srv.filterRegion, []);
            }
            map.get(srv.filterRegion).push(srv);
        }
        return FILTER_REGIONS
            .filter((r) => map.has(r))
            .map((r) => {
                const regionServers = map.get(r);
                const poolOrder = r === 'RU' ? ruPoolOrder : poolOrderInList(regionServers);
                const zones = groupByPool(regionServers, poolOrder);
                return { region: r, servers: regionServers, zones };
            });
    }

    function blockedHostsForFirewall() {
        return resolveBlockedHosts(servers, 'blocklist', settings.blocked);
    }

    function pingLabel(srv) {
        const ms = settings.pings[srv.id];
        if (pinging && !(srv.id in settings.pings)) return '…';
        if (ms === null) return '—';
        return formatPingMs(ms);
    }

    function pingClassFor(id) {
        const ms = settings.pings[id];
        if (pinging && !(id in settings.pings)) return ' is-pending';
        if (ms === null) return ' sb-card-ping--missing';
        return pingLevelClass(ms);
    }

    let blockingRuleCount = 0;
    let statusMessage = '';

    async function syncBlockingStatus() {
        if (!invoke) {
            blockingActive = false;
            blockingRuleCount = 0;
            return;
        }
        try {
            const st = await invoke('server_blocking_active');
            if (typeof st === 'boolean') {
                blockingActive = st;
                blockingRuleCount = st ? 1 : 0;
            } else {
                blockingActive = Boolean(st?.active);
                blockingRuleCount = Number(st?.rule_count ?? 0);
            }
        } catch {
            blockingActive = false;
            blockingRuleCount = 0;
        }
    }

    function countValidPings(serverList) {
        return serverList.filter((s) => typeof settings.pings[s.id] === 'number').length;
    }

    async function runPing(serverList) {
        if (!serverList.length) return 0;
        pinging = true;
        pingProgress = { done: 0, total: serverList.length };
        statusMessage = '';
        renderStatus();
        renderServerList();
        renderProgressViz(serverList);

        let okCount = 0;
        try {
            for (let i = 0; i < serverList.length; i += PING_CHUNK_SIZE) {
                const chunk = serverList.slice(i, i + PING_CHUNK_SIZE);
                const targets = chunk.map((s) => ({ id: s.id, host: s.host, port: s.port }));
                let results;
                if (invoke) {
                    results = await invoke('ping_servers', { targets, timeout_ms: 1200 });
                } else {
                    results = targets.map((target) => ({
                        id: target.id,
                        ms: mockPingMs(target.host, target.port),
                    }));
                }
                for (const row of results ?? []) {
                    settings.pings[row.id] = row.ms ?? null;
                    if (typeof row.ms === 'number') okCount += 1;
                }
                pingProgress.done = Math.min(i + chunk.length, serverList.length);
                renderServerList();
                renderStatus();
                renderProgressViz(serverList);
                await new Promise((r) => requestAnimationFrame(r));
            }
            saveSettings(settings);
            applyAutoBlockBad(serverList);
        } catch (err) {
            console.error('ping_servers failed:', err);
            statusMessage = t('sbPingFailed', { err: String(err) });
        } finally {
            pinging = false;
            pingProgress = null;
            renderServerList();
            renderStatus();
        }
        return okCount;
    }

    async function pingVisibleIfNeeded() {
        if (import.meta.env.MODE === 'test') return;
        const scope = serversInRegion();
        const withPing = scope.filter((s) => {
            const ms = settings.pings[s.id];
            return typeof ms === 'number' && ms <= PING_ACCEPTABLE_MAX_MS;
        });
        if (withPing.length >= Math.min(8, scope.length)) return;
        await runPing(scope);
    }

    function applyAutoBest(scope) {
        const { allowed, preferredByPool } = pickBestPerRegionTopN(settings.pings, scope);
        if (!allowed.length) return false;
        const scopeIds = scope.map((s) => s.id);
        const nextBlocked = computeSelectionAllowed('blocklist', scope, allowed);
        settings.blocked = mergeBlockedForScope(settings.blocked, scopeIds, nextBlocked);
        settings.preferredByPool = mergePreferredByPoolForScope(
            settings.preferredByPool,
            scope,
            preferredByPool,
        );
        saveSettings(settings);
        return true;
    }

    function selectionDirty() {
        if (!blockingActive) return false;
        return !hostSetsEqual(appliedHosts, blockedHostsForFirewall());
    }

    function markSelectionChanged() {
        if (selectionDirty()) {
            statusMessage = t('sbRulesOutOfDate');
        }
    }

    function renderBadge() {
        const regionServers = serversInRegion();
        const denied = regionServers.filter((s) => isDenied(s.id)).length;
        const globalDenied = servers.filter((s) => isDenied(s.id)).length;
        if (!els.blockedBadge) return;
        els.blockedBadge.textContent = String(denied);
        els.blockedBadge.classList.toggle('has-blocked', denied > 0 || globalDenied > 0);
        const bestCount = settings.preferredByPool
            ? Object.keys(settings.preferredByPool).length
            : 0;
        const parts = [];
        if (bestCount > 0) parts.push(`${bestCount} ${t('sbBestPoolsShort')}`);
        if (globalDenied !== denied) parts.push(`${globalDenied} ${t('sbBlockedShort')} ${t('sbGlobalShort')}`);
        const hint = parts.length
            ? `${t('sbStatusConfigured')} · ${parts.join(' · ')}`
            : denied > 0
                ? t('sbStatusConfigured')
                : t('sbStatusIdle');
        els.blockedBadge.title = hint;
    }

    function renderProgressViz(scope = servers) {
        if (!els.progressViz) return;
        const active = pinging || blockingBusy;
        els.progressViz.classList.toggle('is-active', active);
        els.progressViz.classList.toggle('is-blocking', blockingBusy && !pinging);
        if (!active) {
            els.progressViz.setAttribute('aria-hidden', 'true');
            stopWaveLoop();
            return;
        }
        els.progressViz.removeAttribute('aria-hidden');
        const total = pingProgress?.total ?? 1;
        const done = pingProgress?.done ?? 0;
        const pct = pinging
            ? Math.min(100, Math.round((done / Math.max(total, 1)) * 100))
            : 72;
        if (els.progressFill) els.progressFill.style.width = `${pct}%`;
        const track = els.progressViz.querySelector('.sb-viz-track');
        track?.setAttribute('aria-valuenow', String(pct));
        if (els.progressLabel) {
            els.progressLabel.textContent = pinging
                ? t('sbPingProgress', { done, total })
                : t('sbBlockingBusy');
        }
        startWaveLoop(scope);
    }

    function stopWaveLoop() {
        if (waveRaf) cancelAnimationFrame(waveRaf);
        waveRaf = null;
    }

    function drawProgressWave(scope) {
        const canvas = els.progressWave;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;
        const w = canvas.width;
        const h = canvas.height;
        ctx.clearRect(0, 0, w, h);

        const measured = [];
        for (const s of scope) {
            const ms = settings.pings[s.id];
            if (typeof ms === 'number') measured.push(ms);
        }

        const points = measured.length >= 6
            ? measured.slice(-28)
            : Array.from({ length: 28 }, (_, i) => {
                const t = wavePhase + i * 0.35;
                return 55 + Math.sin(t) * 22 + Math.sin(t * 2.3) * 10;
            });

        const max = Math.max(...points, 120);
        const min = Math.min(...points, 20);
        const span = Math.max(max - min, 1);
        const stroke = blockingBusy && !pinging ? '#c45c4a' : '#c96442';

        ctx.strokeStyle = stroke;
        ctx.lineWidth = 1.5;
        ctx.lineJoin = 'round';
        ctx.beginPath();
        points.forEach((val, i) => {
            const x = (i / Math.max(points.length - 1, 1)) * (w - 8) + 4;
            const y = h - 4 - ((val - min) / span) * (h - 8);
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        });
        ctx.stroke();

        ctx.fillStyle = blockingBusy && !pinging ? 'rgba(181,74,58,0.12)' : 'rgba(201,100,66,0.12)';
        ctx.lineTo(w - 4, h);
        ctx.lineTo(4, h);
        ctx.closePath();
        ctx.fill();
    }

    function startWaveLoop(scope) {
        stopWaveLoop();
        const tick = () => {
            wavePhase += 0.12;
            drawProgressWave(scope);
            waveRaf = requestAnimationFrame(tick);
        };
        waveRaf = requestAnimationFrame(tick);
    }

    function renderStatus() {
        if (!els.statusLabel) return;
        if (pinging && pingProgress) {
            els.statusLabel.textContent = t('sbPingProgress', pingProgress);
            els.statusLabel.className = 'sb-status-label is-busy';
            renderProgressViz();
            return;
        }
        if (blockingBusy) {
            els.statusLabel.textContent = t('sbBlockingBusy');
            els.statusLabel.className = 'sb-status-label is-busy';
            renderProgressViz();
            return;
        }
        renderProgressViz();
        if (blockingActive) {
            const rules = blockingRuleCount > 0 ? ` · ${blockingRuleCount}` : '';
            const dirty = selectionDirty() ? ` · ${t('sbRulesOutOfDateShort')}` : '';
            els.statusLabel.textContent = `${t('sbBlockingActive')}${rules}${dirty}`;
            els.statusLabel.className = 'sb-status-label is-active';
            return;
        }
        if (statusMessage) {
            els.statusLabel.textContent = statusMessage;
            els.statusLabel.className = 'sb-status-label is-preview';
            return;
        }
        if (!invoke) {
            els.statusLabel.textContent = t('sbPreviewBanner');
            els.statusLabel.className = 'sb-status-label is-preview';
            return;
        }
        els.statusLabel.textContent = t('sbStatusIdle');
        els.statusLabel.className = 'sb-status-label';
    }

    function updateBlockingButtons() {
        const canResolve = hasResolvableBlocks(servers, 'blocklist', settings.blocked);
        const dirty = selectionDirty();
        const busy = blockingBusy || pinging;
        if (els.startBtn) {
            // While active: Start becomes "Apply" when selection diverges from firewall.
            const enableStart = Boolean(invoke)
                && !busy
                && canResolve
                && (!blockingActive || dirty);
            els.startBtn.disabled = !enableStart;
            const label = els.startBtn.querySelector('[data-i18n]');
            if (label) {
                label.textContent = blockingActive && dirty
                    ? t('sbApplyChanges')
                    : t('sbStartBlock');
            }
        }
        if (els.stopBtn) {
            els.stopBtn.disabled = busy || !invoke || !blockingActive;
        }
        if (els.resetBtn) els.resetBtn.disabled = busy;
        if (els.refreshBtn) els.refreshBtn.disabled = busy;
        if (els.pingBtn) els.pingBtn.disabled = busy;
        if (els.autoBestBtn) els.autoBestBtn.disabled = busy;
    }

    function renderRegionChips() {
        if (!els.regionChips) return;
        const chips = FILTER_REGIONS.map((r) => ({
            id: r,
            label: regionLabel(r, t),
        }));
        els.regionChips.innerHTML = chips.map((c) => `
            <button type="button" class="sb-chip ${regionClass(c.id)}${settings.region === c.id ? ' active' : ''}"
                data-region="${escapeHtml(c.id)}">${escapeHtml(c.label)}</button>
        `).join('');
    }

    function renderServerCard(srv) {
        const denied = isDenied(srv.id);
        const accessKey = denied ? 'sbBlocked' : 'sbAllowed';
        const isBest = !denied && settings.preferredByPool?.[srv.pool] === srv.id;
        const ping = pingLabel(srv);
        const pingClass = pingClassFor(srv.id);
        const bestClass = isBest ? ' sb-card--best' : '';
        const veryBad = isVeryBadPing(settings.pings[srv.id]);
        const highLatClass = veryBad ? ' sb-card--high-latency' : '';
        const hasMs = typeof settings.pings[srv.id] === 'number';
        const pingHtml = hasMs
            ? `${escapeHtml(ping)}<span class="sb-ping-unit">ms</span>`
            : escapeHtml(ping);
        return `
            <article class="sb-card${denied ? ' sb-card--blocked' : ''}${bestClass}${highLatClass}" data-server-id="${escapeHtml(srv.id)}">
                <div class="sb-card-top">
                    <span class="sb-card-pool-tag">${escapeHtml(srv.pool)}</span>
                    <span class="sb-card-ping${pingClass}" data-ping-for="${escapeHtml(srv.id)}">${pingHtml}</span>
                </div>
                ${veryBad ? `<p class="sb-ping-alert" role="alert">${escapeHtml(t('sbPingVeryBad'))}</p>` : ''}
                <h4 class="sb-card-name">${escapeHtml(srv.name)}${isBest ? ` <span class="sb-best-tag">${escapeHtml(t('sbBestTag'))}</span>` : ''}</h4>
                <code class="sb-card-addr">${escapeHtml(srv.address)}</code>
                <div class="sb-card-foot">
                    <span class="sb-card-access${denied ? ' is-blocked' : ''}">${t(accessKey)}</span>
                    <div class="sb-card-power">
                        <span class="sb-power-label">${escapeHtml(t('sbPower'))}</span>
                        <span class="sb-switch-off">OFF</span>
                        <label class="sb-switch" title="${escapeHtml(t('sbToggleHint'))}">
                            <input type="checkbox" class="sb-switch-input" data-block-toggle="${escapeHtml(srv.id)}"
                                ${isSelected(srv.id) ? 'checked' : ''}>
                            <span class="sb-switch-track"><span class="sb-switch-thumb"></span></span>
                        </label>
                        <span class="sb-switch-on">ON</span>
                    </div>
                </div>
            </article>`;
    }

    function renderServerList() {
        const list = filteredServers();
        const groups = groupedByRegion(list);
        if (!groups.length) {
            els.pools.innerHTML = `<div class="sb-empty">${escapeHtml(t('sbEmpty'))}</div>`;
            return;
        }
        els.pools.innerHTML = groups.map((g) => {
            const regionAll = serversInRegion(g.region);
            const deniedInRegion = regionAll.filter((s) => isDenied(s.id)).length;
            const cardsHtml = g.zones.map((z) => {
                    const poolAll = regionAll.filter((s) => s.pool === z.pool);
                    const deniedInZone = poolAll.filter((s) => isDenied(s.id)).length;
                    return `
                        <div class="sb-pool-zone" data-pool="${escapeHtml(z.pool)}">
                            <header class="sb-pool-zone-head">
                                <h4 class="sb-pool-zone-title">${escapeHtml(z.pool)}</h4>
                                <span class="sb-pool-zone-meta">${poolAll.length} · ${deniedInZone} ${t('sbBlockedShort')}</span>
                            </header>
                            <div class="sb-card-grid">
                                ${z.servers.map(renderServerCard).join('')}
                            </div>
                        </div>`;
                }).join('');
            return `
                <section class="sb-region${g.region === 'RU' ? ' sb-region--ru' : ''}" data-region="${escapeHtml(g.region)}">
                    <header class="sb-region-head">
                        <div class="sb-region-title-wrap">
                            <h3 class="sb-region-title ${regionClass(g.region)}">${escapeHtml(regionLabel(g.region, t))}</h3>
                            <span class="sb-region-code">${escapeHtml(g.region)}</span>
                        </div>
                        <span class="sb-region-meta">${regionAll.length} · ${deniedInRegion} ${t('sbBlockedShort')}</span>
                    </header>
                    ${cardsHtml}
                </section>`;
        }).join('');

        els.pools.querySelectorAll('[data-block-toggle]').forEach((input) => {
            input.addEventListener('change', () => {
                const id = input.dataset.blockToggle;
                const set = new Set(settings.blocked);
                if (input.checked) set.add(id);
                else set.delete(id);
                settings.blocked = [...set];
                const pool = id.split('::')[0];
                if (settings.preferredByPool?.[pool] === id && !input.checked) {
                    const next = { ...settings.preferredByPool };
                    delete next[pool];
                    settings.preferredByPool = Object.keys(next).length ? next : null;
                }
                saveSettings(settings);
                markSelectionChanged();
                renderBadge();
                updateBlockingButtons();
                renderStatus();
                renderServerList();
            });
        });
    }

    function setActionLoading(_loading) {
        updateBlockingButtons();
    }

    function render() {
        renderBadge();
        renderStatus();
        renderRegionChips();
        renderServerList();
        updateBlockingButtons();
        if (els.showBlocked) els.showBlocked.checked = settings.showBlockedOnly;
    }

    els.search?.addEventListener('input', () => renderServerList());
    els.regionChips?.addEventListener('click', (e) => {
        const btn = e.target.closest('.sb-chip[data-region]');
        if (!btn) return;
        settings.region = btn.dataset.region;
        saveSettings(settings);
        render();
    });
    els.showBlocked?.addEventListener('change', () => {
        settings.showBlockedOnly = els.showBlocked.checked;
        saveSettings(settings);
        renderServerList();
    });

    els.resetBtn?.addEventListener('click', async () => {
        if (blockingBusy || pinging) return;
        const ok = typeof window.__confirmAction === 'function'
            ? await window.__confirmAction(t('confirmSbReset'))
            : window.confirm(t('confirmSbReset'));
        if (!ok) return;
        settings = structuredClone(DEFAULT_SETTINGS);
        saveSettings(settings);
        if (els.search) els.search.value = '';
        statusMessage = '';
        if (invoke && (blockingActive || blockingRuleCount > 0)) {
            blockingBusy = true;
            updateBlockingButtons();
            renderStatus();
            try {
                await invoke('stop_server_blocking');
                blockingActive = false;
                appliedHosts = [];
                statusMessage = t('sbResetCleared');
                window.__showToast?.(t('toastSbReset'), 'success');
            } catch (err) {
                console.error('reset stop_server_blocking failed:', err);
                statusMessage = t('sbBlockFailed', { err: String(err) });
                window.__showToast?.(t('toastActionFailed'), 'error');
            } finally {
                blockingBusy = false;
                await syncBlockingStatus();
            }
        } else {
            appliedHosts = [];
            statusMessage = t('sbResetClearedLocal');
            window.__showToast?.(t('toastSbReset'), 'success');
        }
        render();
    });

    els.pingBtn?.addEventListener('click', async () => {
        if (blockingBusy || pinging) return;
        setActionLoading(true);
        const scope = serversInRegion();
        const ok = await runPing(scope);
        markSelectionChanged();
        if (ok > 0) {
            const hidden = countHiddenServers(scope, settings.pings);
            statusMessage = hidden > 0
                ? t('sbPingDoneHidden', { ok, total: scope.length, hidden })
                : t('sbPingDone', { ok, total: scope.length });
        } else if (!statusMessage) {
            statusMessage = t('sbAutoBestNoPing');
        }
        setActionLoading(false);
        render();
    });

    els.autoBestBtn?.addEventListener('click', async () => {
        if (blockingBusy || pinging) return;
        const scope = serversInRegion();
        setActionLoading(true);
        const poolTotal = new Set(scope.map((s) => s.pool)).size;
        if (poolsWithValidPing(settings.pings, scope) < poolTotal) {
            await runPing(scope);
        }
        applyAutoBlockBad(scope);
        const ok = applyAutoBest(scope);
        markSelectionChanged();
        if (!ok) {
            statusMessage = t('sbAutoBestNoPing');
        } else {
            const preferredCount = Object.keys(settings.preferredByPool ?? {}).length;
            const hidden = countHiddenServers(scope, settings.pings);
            statusMessage = hidden > 0
                ? t('sbAutoBestDoneHidden', {
                    pools: preferredCount,
                    hidden,
                })
                : t('sbAutoBestDone', {
                    pools: preferredCount,
                });
        }
        setActionLoading(false);
        render();
    });

    els.startBtn?.addEventListener('click', async () => {
        if (!invoke || blockingBusy || pinging) return;
        const ips = blockedHostsForFirewall();
        if (!ips.length) {
            statusMessage = t('sbNoHostsToBlock');
            renderStatus();
            updateBlockingButtons();
            return;
        }
        blockingBusy = true;
        statusMessage = '';
        renderStatus();
        updateBlockingButtons();
        try {
            const msg = await invoke('start_server_blocking', { ips });
            appliedHosts = [...ips];
            statusMessage = t('sbBlockSuccess');
            window.__showToast?.(t('toastSbApplied'), 'success');
            console.info(msg);
        } catch (err) {
            console.error('start_server_blocking failed:', err);
            statusMessage = t('sbBlockFailed', { err: String(err) });
            window.__showToast?.(t('toastActionFailed'), 'error');
        } finally {
            blockingBusy = false;
            await syncBlockingStatus();
            if (blockingActive) appliedHosts = [...ips];
            render();
        }
    });

    els.stopBtn?.addEventListener('click', async () => {
        if (!invoke || blockingBusy || pinging) return;
        blockingBusy = true;
        statusMessage = '';
        renderStatus();
        updateBlockingButtons();
        try {
            const msg = await invoke('stop_server_blocking');
            appliedHosts = [];
            statusMessage = t('sbBlockStopped');
            window.__showToast?.(t('toastSbStopped'), 'success');
            console.info(msg);
        } catch (err) {
            console.error('stop_server_blocking failed:', err);
            statusMessage = t('sbBlockFailed', { err: String(err) });
            window.__showToast?.(t('toastActionFailed'), 'error');
        } finally {
            blockingBusy = false;
            await syncBlockingStatus();
            if (!blockingActive) appliedHosts = [];
            render();
        }
    });

    els.refreshBtn?.addEventListener('click', async () => {
        if (blockingBusy || pinging) return;
        if (els.refreshBtn) els.refreshBtn.disabled = true;
        statusMessage = t('sbRefreshBusy');
        renderStatus();
        try {
            const fresh = await fetchAllCatalogs({
                fallbackRoxy: liveRoxy,
                fallbackRu: liveRu,
            });
            liveRoxy = fresh.roxy;
            liveRu = fresh.ru;
            ruPoolOrder = (liveRu?.pools ?? []).map((p) => p.name);
            servers = flattenServerCatalog(mergeServerCatalogs(liveRoxy, liveRu));
            settings = pruneSelectionToCatalog(servers, settings);
            saveSettings(settings);
            const hosts = blockedHostsForFirewall();
            const liveHint = fresh.liveCount > 0
                ? t('sbRefreshLive', { n: fresh.liveCount })
                : t('sbRefreshFallback');
            if (blockingActive && invoke) {
                if (!hosts.length) {
                    blockingBusy = true;
                    try {
                        await invoke('stop_server_blocking');
                        appliedHosts = [];
                        statusMessage = `${liveHint} · ${t('sbRefreshStoppedEmpty')}`;
                    } catch (err) {
                        statusMessage = t('sbBlockFailed', { err: String(err) });
                    } finally {
                        blockingBusy = false;
                        await syncBlockingStatus();
                    }
                } else if (!hostSetsEqual(hosts, appliedHosts)) {
                    blockingBusy = true;
                    try {
                        await invoke('start_server_blocking', { ips: hosts });
                        appliedHosts = [...hosts];
                        statusMessage = `${liveHint} · ${t('sbRefreshReapplied')}`;
                    } catch (err) {
                        statusMessage = t('sbBlockFailed', { err: String(err) });
                    } finally {
                        blockingBusy = false;
                        await syncBlockingStatus();
                        if (blockingActive) appliedHosts = [...hosts];
                    }
                } else {
                    statusMessage = `${liveHint} · ${t('sbRefreshDone')}`;
                }
            } else {
                statusMessage = `${liveHint} · ${t('sbRefreshDone')}`;
            }
        } catch (err) {
            console.error('catalog refresh failed:', err);
            statusMessage = t('sbRefreshFailed', { err: String(err) });
        }
        render();
        if (els.refreshBtn) els.refreshBtn.disabled = false;
    });

    await syncBlockingStatus();
    if (blockingActive) appliedHosts = blockedHostsForFirewall();
    render();
    return {
        render,
        pingVisibleIfNeeded,
        getServerCount: () => servers.length,
    };
}
