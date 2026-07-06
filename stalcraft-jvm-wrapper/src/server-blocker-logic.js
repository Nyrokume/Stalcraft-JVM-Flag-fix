/** Pure helpers for Server Blocker — unit-testable. */

export function formatPingMs(ms) {
    if (ms == null || ms < 0) return '—';
    return `${ms}`;
}

export const PING_ACCEPTABLE_MAX_MS = 200;

export function isAcceptablePing(ms) {
    return typeof ms === 'number' && ms >= 0 && ms <= PING_ACCEPTABLE_MAX_MS;
}

export function pickBestServerId(pingMap, serverIds) {
    let bestId = null;
    let bestMs = Infinity;
    for (const id of serverIds) {
        const ms = pingMap[id];
        if (!isAcceptablePing(ms)) continue;
        if (ms < bestMs) {
            bestMs = ms;
            bestId = id;
        }
    }
    return bestId;
}

/** Blocklist: block every server except the best ping in scope. */
export function computeAutoBlock(serverIds, bestId) {
    if (!bestId || !serverIds.includes(bestId)) return [...serverIds];
    return serverIds.filter((id) => id !== bestId);
}

/** Best server id per pool (lowest ping with valid ms). */
export function pickBestPerPool(pingMap, servers) {
    const byPool = new Map();
    for (const srv of servers) {
        if (!byPool.has(srv.pool)) byPool.set(srv.pool, []);
        byPool.get(srv.pool).push(srv.id);
    }
    const bestByPool = {};
    for (const [pool, ids] of byPool) {
        const best = pickBestServerId(pingMap, ids);
        if (best) bestByPool[pool] = best;
    }
    return bestByPool;
}

/** Blocklist: keep one best server per pool in scope. */
export function computeAutoBlockPerPool(servers, bestByPool) {
    const blocked = [];
    for (const srv of servers) {
        const best = bestByPool[srv.pool];
        if (!best) continue;
        if (srv.id !== best) blocked.push(srv.id);
    }
    return blocked;
}

/** Group servers by filterRegion (RU, EU, NA, SEA). */
export function groupByRegionServers(servers) {
    const map = new Map();
    for (const srv of servers) {
        if (!map.has(srv.filterRegion)) map.set(srv.filterRegion, []);
        map.get(srv.filterRegion).push(srv);
    }
    return map;
}

/** Lowest-ping server ids, up to n. */
export function pickTopNByPing(pingMap, serverIds, n = 3) {
    return serverIds
        .filter((id) => isAcceptablePing(pingMap[id]))
        .sort((a, b) => pingMap[a] - pingMap[b])
        .slice(0, n);
}

export const AUTO_BEST_TOP_N = 3;

/**
 * Per region: best per pool → keep top N by ping → block the rest in scope.
 * preferredByPool only includes pool winners that made the top-N cut.
 */
export function pickBestPerRegionTopN(pingMap, servers, n = AUTO_BEST_TOP_N) {
    const allowed = [];
    const preferredByPool = {};
    for (const regionServers of groupByRegionServers(servers).values()) {
        const bestByPool = pickBestPerPool(pingMap, regionServers);
        const top = pickTopNByPing(pingMap, Object.values(bestByPool), n);
        for (const id of top) allowed.push(id);
        for (const [pool, id] of Object.entries(bestByPool)) {
            if (top.includes(id)) preferredByPool[pool] = id;
        }
    }
    return { allowed, preferredByPool };
}

/** Mode-aware blocked/allowed list from explicit allowed server ids. */
export function computeSelectionAllowed(mode, servers, allowedIds) {
    const allowed = new Set(allowedIds);
    if (mode === 'allowlist') return [...allowed];
    return servers.filter((s) => !allowed.has(s.id)).map((s) => s.id);
}

/**
 * Mode-aware selection after auto-best.
 * blocklist: settings.blocked = all except best per pool
 * allowlist: settings.blocked (= allowed) = only best per pool
 */
export function computeSelectionPerPool(mode, servers, bestByPool) {
    if (mode === 'allowlist') {
        return Object.values(bestByPool);
    }
    return computeAutoBlockPerPool(servers, bestByPool);
}

export const PING_VERY_BAD_MS = 300;

export function isVeryBadPing(ms) {
    return typeof ms === 'number' && ms > PING_VERY_BAD_MS;
}

/** Ping attempted but host unreachable / timed out. */
export function isMissingPing(ms) {
    return ms === null;
}

/** Measured ping over acceptable threshold or unreachable. */
export function isUnacceptablePing(ms) {
    if (ms === null) return true;
    if (typeof ms !== 'number') return false;
    return !isAcceptablePing(ms);
}

/**
 * Whether a server card should appear in the list.
 * Unpinged servers stay visible until a measurement exists.
 * During an active ping pass, servers awaiting measurement stay visible.
 */
export function shouldShowServerInMenu(pingMap, serverId, { pinging = false } = {}) {
    if (pinging) return true;
    if (!(serverId in pingMap)) return true;
    const ms = pingMap[serverId];
    if (ms === undefined) return true;
    if (isMissingPing(ms)) return false;
    if (isUnacceptablePing(ms)) return false;
    return true;
}

/** Blocklist: mark bad/null as blocked. Allowlist: remove them from allowed set. */
export function mergeAutoBlockUnacceptable(mode, servers, pingMap, currentBlocked) {
    const set = new Set(currentBlocked);
    for (const srv of servers) {
        if (!(srv.id in pingMap)) continue;
        const ms = pingMap[srv.id];
        if (!isUnacceptablePing(ms)) continue;
        if (mode === 'blocklist') set.add(srv.id);
        else set.delete(srv.id);
    }
    return [...set];
}

export function countHiddenServers(servers, pingMap, { pinging = false } = {}) {
    return servers.filter((s) => !shouldShowServerInMenu(pingMap, s.id, { pinging })).length;
}

/** CSS modifier for ping latency: good ≤60ms, mid ≤200ms, else bad. */
export function pingLevelClass(ms) {
    if (ms == null || ms < 0) return '';
    if (isVeryBadPing(ms)) return ' sb-card-ping--very-bad';
    if (ms <= 60) return ' sb-card-ping--good';
    if (ms <= PING_ACCEPTABLE_MAX_MS) return ' sb-card-ping--mid';
    return ' sb-card-ping--bad';
}

/** Unique IPv4 hosts to firewall-block for the current mode + selection. */
export function resolveBlockedHosts(servers, mode, selectedIds) {
    const selected = new Set(selectedIds);
    const hosts = mode === 'allowlist'
        ? servers.filter((s) => !selected.has(s.id)).map((s) => s.host)
        : servers.filter((s) => selected.has(s.id)).map((s) => s.host);
    return [...new Set(hosts.filter(Boolean))];
}

export function hasBlockingSelection(servers, mode, selectedIds) {
    if (!selectedIds.length) return false;
    if (mode === 'allowlist') {
        return selectedIds.length < servers.length;
    }
    return true;
}

/** Group servers by pool name, preserving catalog order. */
export function groupByPool(servers, poolOrder = []) {
    const map = new Map();
    for (const srv of servers) {
        if (!map.has(srv.pool)) map.set(srv.pool, []);
        map.get(srv.pool).push(srv);
    }
    const order = poolOrder.length
        ? poolOrder.filter((p) => map.has(p))
        : [...map.keys()].sort();
    return order.map((pool) => ({ pool, servers: map.get(pool) }));
}

/** Count pools in scope that have at least one server with valid ping. */
export function poolsWithValidPing(pingMap, servers) {
    const pools = [...new Set(servers.map((s) => s.pool))];
    return pools.filter((pool) =>
        servers.some((s) => s.pool === pool && isAcceptablePing(pingMap[s.id])),
    ).length;
}

export const PING_CHUNK_SIZE = 16;

export function mockPingMs(host, port) {
    let h = 0;
    const s = `${host}:${port}`;
    for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) >>> 0;
    return 35 + (h % 160);
}
