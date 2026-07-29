import { readFileSync } from 'node:fs';

import { dirname, join } from 'node:path';

import { fileURLToPath } from 'node:url';

import test from 'node:test';

import assert from 'node:assert/strict';



import {
    flattenServerCatalog,
    fetchRuCatalog,
    fetchAllCatalogs,
    getRegions,
    loadSettings,
    loadCatalogCache,
    mergeServerCatalogs,
    saveSettings,
    saveCatalogCache,
} from '../src/server-blocker.js';

import {

    computeAutoBlock,

    computeAutoBlockPerPool,

    computeSelectionPerPool,
    computeSelectionAllowed,
    pickBestPerRegionTopN,
    pickTopNByPing,
    AUTO_BEST_TOP_N,

    formatPingMs,

    mockPingMs,

    pickBestPerPool,

    pickBestServerId,

    pingLevelClass,
    groupByPool,
    hasBlockingSelection,
    hasResolvableBlocks,
    hostSetsEqual,
    poolsWithValidPing,
    pruneSelectionToCatalog,
    resolveBlockedHosts,
    shouldShowServerInMenu,
    mergeAutoBlockUnacceptable,
    mergeBlockedForScope,
    mergePreferredByPoolForScope,
    countHiddenServers,
    isUnacceptablePing,
} from '../src/server-blocker-logic.js';



const root = join(dirname(fileURLToPath(import.meta.url)), '..');

const roxyCatalog = JSON.parse(

    readFileSync(join(root, 'src/data/servers.json'), 'utf8'),

);

const ruCatalog = JSON.parse(

    readFileSync(join(root, 'src/data/servers-ru.json'), 'utf8'),

);

const catalog = mergeServerCatalogs(roxyCatalog, ruCatalog);

const servers = flattenServerCatalog(catalog);

const expectedRu = (ruCatalog.pools ?? []).reduce((n, p) => n + (p.tunnels?.length ?? 0), 0);
const expectedRoxy = (roxyCatalog.pools ?? [])
    .filter((p) => p.name !== 'MSK2X')
    .reduce((n, p) => n + (p.tunnels?.length ?? 0), 0);
const expectedTotal = expectedRu + expectedRoxy;



test('flattenServerCatalog returns all tunnels', () => {

    assert.equal(servers.length, expectedTotal);
    assert.ok(servers.length >= 60, `catalog too small: ${servers.length}`);

    assert.equal(servers[0].id, 'GAME-EU::GAME-EU-2');

    assert.equal(servers[0].host, '79.127.241.67');

    assert.equal(servers[0].port, 29450);

});



test('getRegions returns RU, EU, NA, SEA', () => {

    const regions = getRegions(servers);

    assert.deepEqual(regions, ['RU', 'EU', 'NA', 'SEA']);

});



test('merged catalog has roxy + RU pools', () => {

    const roxyPools = (roxyCatalog.pools ?? []).filter((p) => p.name !== 'MSK2X').length;
    const ruPools = (ruCatalog.pools ?? []).length;
    assert.equal(catalog.pools.length, roxyPools + ruPools);

});



test('RU pools from stalcraftx backend', () => {

    const ru = servers.filter((s) => s.filterRegion === 'RU');

    assert.equal(ru.length, expectedRu);

    assert.ok(ru.some((s) => s.pool === 'MSK1'));

    assert.ok(ru.some((s) => s.pool === 'SPB3'));

    assert.ok(ru.every((s) => s.filterRegion === 'RU'));

});



test('settings round-trip via localStorage mock', () => {

    const store = {};

    const original = globalThis.localStorage;

    globalThis.localStorage = {

        getItem: (k) => store[k] ?? null,

        setItem: (k, v) => { store[k] = v; },

        removeItem: (k) => { delete store[k]; },

    };

    try {

        saveSettings({

            v: 5,

            mode: 'blocklist',

            blocked: ['GAME-EU::GAME-EU-2'],

            region: 'EU',

            pings: {},

            preferredByPool: { 'GAME-EU': 'GAME-EU::GAME-EU-2' },

        });

        const loaded = loadSettings();

        assert.equal(loaded.mode, 'blocklist');

        assert.deepEqual(loaded.blocked, ['GAME-EU::GAME-EU-2']);

        assert.equal(loaded.region, 'EU');

        assert.deepEqual(loaded.preferredByPool, { 'GAME-EU': 'GAME-EU::GAME-EU-2' });

    } finally {

        globalThis.localStorage = original;

    }

});



test('pickBestServerId chooses lowest acceptable ping', () => {

    const pings = { a: 120, b: 45, c: null, d: 95 };

    assert.equal(pickBestServerId(pings, ['a', 'b', 'c', 'd']), 'b');

});



test('pickBestPerPool picks one best per pool', () => {

    const scope = [

        { id: 'A::1', pool: 'A' },

        { id: 'A::2', pool: 'A' },

        { id: 'B::1', pool: 'B' },

        { id: 'B::2', pool: 'B' },

    ];

    const pings = { 'A::1': 80, 'A::2': 40, 'B::1': 90, 'B::2': 70 };

    assert.deepEqual(pickBestPerPool(pings, scope), { A: 'A::2', B: 'B::2' });

});



test('computeAutoBlock blocks all except best', () => {

    const ids = ['a', 'b', 'c'];

    assert.deepEqual(computeAutoBlock(ids, 'b'), ['a', 'c']);

});



test('computeAutoBlockPerPool blocks non-best in each pool', () => {

    const scope = [

        { id: 'A::1', pool: 'A' },

        { id: 'A::2', pool: 'A' },

        { id: 'B::1', pool: 'B' },

    ];

    const bestByPool = { A: 'A::2', B: 'B::1' };

    assert.deepEqual(computeAutoBlockPerPool(scope, bestByPool), ['A::1']);

});



test('pingLevelClass thresholds', () => {

    assert.equal(pingLevelClass(null), '');

    assert.equal(pingLevelClass(45), ' sb-card-ping--good');

    assert.equal(pingLevelClass(80), ' sb-card-ping--mid');

    assert.equal(pingLevelClass(150), ' sb-card-ping--mid');

    assert.equal(pingLevelClass(250), ' sb-card-ping--bad');

    assert.equal(pingLevelClass(301), ' sb-card-ping--very-bad');

});



test('formatPingMs handles missing values', () => {

    assert.equal(formatPingMs(null), '—');

    assert.equal(formatPingMs(87), '87');

});



test('groupByPool preserves pool order', () => {
    const list = [
        { id: 'b::1', pool: 'B' },
        { id: 'a::1', pool: 'A' },
        { id: 'a::2', pool: 'A' },
    ];
    const grouped = groupByPool(list, ['A', 'B']);
    assert.deepEqual(grouped.map((g) => g.pool), ['A', 'B']);
    assert.equal(grouped[0].servers.length, 2);
});

test('resolveBlockedHosts for blocklist and allowlist', () => {
    const list = [
        { id: 'a', host: '1.1.1.1' },
        { id: 'b', host: '2.2.2.2' },
        { id: 'c', host: '1.1.1.1' },
    ];
    assert.deepEqual(resolveBlockedHosts(list, 'blocklist', ['a']), ['1.1.1.1']);
    assert.deepEqual(resolveBlockedHosts(list, 'allowlist', ['a']), ['2.2.2.2', '1.1.1.1']);
    assert.equal(hasBlockingSelection(list, 'allowlist', ['a', 'b', 'c']), false);
    assert.equal(hasBlockingSelection(list, 'allowlist', ['a']), true);
});

test('mockPingMs is deterministic', () => {

    assert.equal(mockPingMs('1.2.3.4', 29450), mockPingMs('1.2.3.4', 29450));

    assert.ok(mockPingMs('x', 1) >= 35);

});

test('computeSelectionPerPool blocklist blocks non-best', () => {
    const scope = [
        { id: 'A::1', pool: 'A' },
        { id: 'A::2', pool: 'A' },
        { id: 'B::1', pool: 'B' },
    ];
    const bestByPool = { A: 'A::2', B: 'B::1' };
    assert.deepEqual(computeSelectionPerPool('blocklist', scope, bestByPool), ['A::1']);
});

test('computeSelectionPerPool allowlist allows only best', () => {
    const scope = [
        { id: 'A::1', pool: 'A' },
        { id: 'A::2', pool: 'A' },
        { id: 'B::1', pool: 'B' },
    ];
    const bestByPool = { A: 'A::2', B: 'B::1' };
    assert.deepEqual(computeSelectionPerPool('allowlist', scope, bestByPool), ['A::2', 'B::1']);
});

test('pickBestPerRegionTopN keeps top 3 pool winners per region', () => {
    const scope = [
        { id: 'A::1', pool: 'A', filterRegion: 'EU' },
        { id: 'A::2', pool: 'A', filterRegion: 'EU' },
        { id: 'B::1', pool: 'B', filterRegion: 'EU' },
        { id: 'B::2', pool: 'B', filterRegion: 'EU' },
        { id: 'C::1', pool: 'C', filterRegion: 'EU' },
        { id: 'C::2', pool: 'C', filterRegion: 'EU' },
        { id: 'C::3', pool: 'C', filterRegion: 'EU' },
        { id: 'D::1', pool: 'D', filterRegion: 'EU' },
    ];
    const pings = {
        'A::1': 90, 'A::2': 40,
        'B::1': 80, 'B::2': 35,
        'C::1': 70, 'C::2': 30, 'C::3': 200,
        'D::1': 25,
    };
    const { allowed, preferredByPool } = pickBestPerRegionTopN(pings, scope, 3);
    assert.deepEqual(allowed, ['D::1', 'C::2', 'B::2']);
    assert.deepEqual(preferredByPool, { B: 'B::2', C: 'C::2', D: 'D::1' });
    assert.deepEqual(
        computeSelectionAllowed('blocklist', scope, allowed),
        ['A::1', 'A::2', 'B::1', 'C::1', 'C::3'],
    );
});

test('pickBestPerRegionTopN skips ping above 200ms', () => {
    const scope = [
        { id: 'A::1', pool: 'A', filterRegion: 'EU' },
        { id: 'B::1', pool: 'B', filterRegion: 'EU' },
    ];
    const pings = { 'A::1': 250, 'B::1': 80 };
    const { allowed } = pickBestPerRegionTopN(pings, scope, 3);
    assert.deepEqual(allowed, ['B::1']);
});

test('mergeServerCatalogs excludes MSK2X from roxy', () => {
    const roxyWithMsk2x = {
        pools: [
            { name: 'MSK2X', tunnels: [{ name: 'x', address: '1.1.1.1:29450' }] },
            { name: 'GAME-EU', region: 'EU', tunnels: [] },
        ],
    };
    const merged = mergeServerCatalogs(roxyWithMsk2x, { pools: [] });
    assert.ok(!merged.pools.some((p) => p.name === 'MSK2X'));
});

test('loadSettings resets on corrupt JSON and version mismatch', () => {
    const store = {};
    const original = globalThis.localStorage;
    globalThis.localStorage = {
        getItem: (k) => store[k] ?? null,
        setItem: (k, v) => { store[k] = v; },
        removeItem: (k) => { delete store[k]; },
    };
    try {
        store['stalcraft-sb-settings'] = '{not json';
        assert.equal(loadSettings().mode, 'blocklist');
        store['stalcraft-sb-settings'] = JSON.stringify({ v: 1, mode: 'allowlist' });
        assert.equal(loadSettings().mode, 'blocklist');
        assert.equal(loadSettings().region, 'RU');
        store['stalcraft-sb-settings'] = JSON.stringify({ v: 4, mode: 'allowlist', region: 'ALL', blocked: [] });
        const migrated = loadSettings();
        assert.equal(migrated.mode, 'blocklist');
        assert.equal(migrated.region, 'RU');
    } finally {
        globalThis.localStorage = original;
    }
});

test('fetchRuCatalog falls back to bundled on fetch failure', async () => {
    const original = globalThis.fetch;
    globalThis.fetch = async () => {
        throw new Error('offline');
    };
    try {
        const data = await fetchRuCatalog();
        assert.ok(Array.isArray(data?.pools));
        assert.ok(data.pools.length > 0);
    } finally {
        globalThis.fetch = original;
    }
});

test('fetchAllCatalogs merges live regions and caches', async () => {
    const original = globalThis.fetch;
    const store = {};
    const ls = globalThis.localStorage;
    globalThis.localStorage = {
        getItem: (k) => store[k] ?? null,
        setItem: (k, v) => { store[k] = v; },
        removeItem: (k) => { delete store[k]; },
    };
    globalThis.fetch = async (url) => {
        const u = String(url);
        if (u.includes('stalcraftx.ru')) {
            return {
                ok: true,
                json: async () => ({
                    mode: 'roxy',
                    pools: [{ name: 'MSK1', tunnels: [{ name: 'MSK1-1', address: '1.1.1.1:29450' }] }],
                }),
            };
        }
        if (u.includes('/EU.json')) {
            return {
                ok: true,
                json: async () => ({
                    pools: [{ name: 'GAME-EU', tunnels: [{ name: 'GAME-EU-2', address: '2.2.2.2:29450' }] }],
                }),
            };
        }
        if (u.includes('/NA.json')) {
            return {
                ok: true,
                json: async () => ({
                    pools: [{ name: 'GAME-NA', tunnels: [{ name: 'GAME-NA-3', address: '3.3.3.3:29450' }] }],
                }),
            };
        }
        if (u.includes('/SEA.json')) {
            return {
                ok: true,
                json: async () => ({
                    pools: [{ name: 'GAME-SEA', tunnels: [{ name: 'GAME-SEA-2', address: '4.4.4.4:29450' }] }],
                }),
            };
        }
        throw new Error(`unexpected url ${u}`);
    };
    try {
        const { roxy, ru, liveCount, sources } = await fetchAllCatalogs({
            fallbackRoxy: { pools: [] },
            fallbackRu: { pools: [] },
        });
        assert.equal(liveCount, 4);
        assert.equal(sources.ru, 'live');
        assert.equal(ru.pools[0].name, 'MSK1');
        assert.equal(roxy.pools.length, 3);
        assert.ok(roxy.pools.every((p) => ['EU', 'NA', 'SEA'].includes(p.region)));
        const cached = loadCatalogCache();
        assert.ok(cached);
        assert.equal(cached.ru.pools[0].name, 'MSK1');
        saveCatalogCache(roxy, ru);
        assert.ok(loadCatalogCache());
    } finally {
        globalThis.fetch = original;
        globalThis.localStorage = ls;
    }
});

test('fetchAllCatalogs falls back per region on failure', async () => {
    const original = globalThis.fetch;
    globalThis.fetch = async () => {
        throw new Error('offline');
    };
    try {
        const fallbackRu = {
            pools: [{ name: 'MSK1', tunnels: [{ name: 'MSK1-1', address: '9.9.9.9:29450' }] }],
        };
        const fallbackRoxy = {
            pools: [
                { name: 'GAME-EU', region: 'EU', tunnels: [{ name: 'GAME-EU-2', address: '8.8.8.8:29450' }] },
            ],
        };
        const { ru, roxy, liveCount } = await fetchAllCatalogs({ fallbackRoxy, fallbackRu });
        assert.equal(liveCount, 0);
        assert.equal(ru.pools[0].name, 'MSK1');
        assert.equal(roxy.pools[0].name, 'GAME-EU');
        assert.equal(roxy.pools[0].region, 'EU');
    } finally {
        globalThis.fetch = original;
    }
});

test('poolsWithValidPing counts pools with at least one numeric ping', () => {
    const scope = [
        { id: 'A::1', pool: 'A' },
        { id: 'A::2', pool: 'A' },
        { id: 'B::1', pool: 'B' },
    ];
    const pings = { 'A::1': 50, 'A::2': null, 'B::1': null };
    assert.equal(poolsWithValidPing(pings, scope), 1);
    pings['B::1'] = 80;
    assert.equal(poolsWithValidPing(pings, scope), 2);
});

test('shouldShowServerInMenu hides bad and missing after ping', () => {
    const pings = { 'a': 45, 'b': 250, 'c': null, 'd': undefined };
    assert.equal(shouldShowServerInMenu(pings, 'a'), true);
    assert.equal(shouldShowServerInMenu(pings, 'b'), false);
    assert.equal(shouldShowServerInMenu(pings, 'c'), false);
    assert.equal(shouldShowServerInMenu(pings, 'd'), true);
    assert.equal(shouldShowServerInMenu(pings, 'b', { pinging: true }), true);
    assert.equal(shouldShowServerInMenu({ e: 180 }, 'e'), true);
});

test('mergeAutoBlockUnacceptable marks bad servers blocked in blocklist mode', () => {
    const scope = [
        { id: 'a', pool: 'A' },
        { id: 'b', pool: 'A' },
    ];
    const pings = { a: 40, b: 250 };
    const result = mergeAutoBlockUnacceptable('blocklist', scope, pings, [], { A: 'b' });
    assert.deepEqual(result.blocked.sort(), ['b']);
    assert.equal(result.preferredByPool, null);
});

test('mergeBlockedForScope preserves other regions', () => {
    const prev = ['EU::1', 'RU::old'];
    const scopeIds = ['RU::old', 'RU::new', 'RU::keep'];
    const next = ['RU::new'];
    assert.deepEqual(
        mergeBlockedForScope(prev, scopeIds, next).sort(),
        ['EU::1', 'RU::new'].sort(),
    );
});

test('mergePreferredByPoolForScope keeps other pools', () => {
    const prev = { MSK1: 'MSK1::1', 'GAME-EU': 'GAME-EU::2' };
    const scope = [{ id: 'MSK1::2', pool: 'MSK1' }];
    const next = { MSK1: 'MSK1::2' };
    assert.deepEqual(
        mergePreferredByPoolForScope(prev, scope, next),
        { MSK1: 'MSK1::2', 'GAME-EU': 'GAME-EU::2' },
    );
});

test('pruneSelectionToCatalog drops orphans', () => {
    const servers = [
        { id: 'A::1', pool: 'A' },
        { id: 'B::1', pool: 'B' },
    ];
    const pruned = pruneSelectionToCatalog(servers, {
        blocked: ['A::1', 'GONE::1'],
        pings: { 'A::1': 40, 'GONE::1': 99 },
        preferredByPool: { A: 'A::1', X: 'X::1' },
    });
    assert.deepEqual(pruned.blocked, ['A::1']);
    assert.deepEqual(pruned.pings, { 'A::1': 40 });
    assert.deepEqual(pruned.preferredByPool, { A: 'A::1' });
});

test('hasResolvableBlocks ignores orphan-only selection', () => {
    const servers = [{ id: 'A::1', host: '1.1.1.1', pool: 'A' }];
    assert.equal(hasResolvableBlocks(servers, 'blocklist', ['MISSING::1']), false);
    assert.equal(hasResolvableBlocks(servers, 'blocklist', ['A::1']), true);
});

test('hostSetsEqual is order-independent', () => {
    assert.equal(hostSetsEqual(['2.2.2.2', '1.1.1.1'], ['1.1.1.1', '2.2.2.2']), true);
    assert.equal(hostSetsEqual(['1.1.1.1'], ['1.1.1.1', '2.2.2.2']), false);
});

test('countHiddenServers tallies bad and missing', () => {
    const scope = [{ id: 'a' }, { id: 'b' }, { id: 'c' }];
    const pings = { a: 50, b: null, c: 180 };
    assert.equal(countHiddenServers(scope, pings), 1);
});

