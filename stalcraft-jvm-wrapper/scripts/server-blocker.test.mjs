import { readFileSync } from 'node:fs';

import { dirname, join } from 'node:path';

import { fileURLToPath } from 'node:url';

import test from 'node:test';

import assert from 'node:assert/strict';



import {

    flattenServerCatalog,

    fetchRuCatalog,

    getRegions,

    loadSettings,

    mergeServerCatalogs,

    saveSettings,

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
    poolsWithValidPing,
    resolveBlockedHosts,
    shouldShowServerInMenu,
    mergeAutoBlockUnacceptable,
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



test('flattenServerCatalog returns all tunnels', () => {

    assert.equal(servers.length, 77);

    assert.equal(servers[0].id, 'GAME-EU::GAME-EU-2');

    assert.equal(servers[0].host, '79.127.241.67');

    assert.equal(servers[0].port, 29450);

});



test('getRegions returns RU, EU, NA, SEA', () => {

    const regions = getRegions(servers);

    assert.deepEqual(regions, ['RU', 'EU', 'NA', 'SEA']);

});



test('merged catalog has 21 pools', () => {

    assert.equal(catalog.pools.length, 21);

});



test('RU pools from stalcraftx backend', () => {

    const ru = servers.filter((s) => s.filterRegion === 'RU');

    assert.equal(ru.length, 59);

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

    assert.equal(pingLevelClass(150), ' sb-card-ping--bad');

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

test('pickBestPerRegionTopN skips ping above 100ms', () => {
    const scope = [
        { id: 'A::1', pool: 'A', filterRegion: 'EU' },
        { id: 'B::1', pool: 'B', filterRegion: 'EU' },
    ];
    const pings = { 'A::1': 150, 'B::1': 80 };
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
    const pings = { 'a': 45, 'b': 150, 'c': null, 'd': undefined };
    assert.equal(shouldShowServerInMenu(pings, 'a'), true);
    assert.equal(shouldShowServerInMenu(pings, 'b'), false);
    assert.equal(shouldShowServerInMenu(pings, 'c'), false);
    assert.equal(shouldShowServerInMenu(pings, 'd'), true);
    assert.equal(shouldShowServerInMenu(pings, 'b', { pinging: true }), true);
});

test('mergeAutoBlockUnacceptable marks bad servers blocked in blocklist mode', () => {
    const scope = [
        { id: 'a', pool: 'A' },
        { id: 'b', pool: 'A' },
    ];
    const pings = { a: 40, b: 200 };
    const blocked = mergeAutoBlockUnacceptable('blocklist', scope, pings, []);
    assert.deepEqual(blocked.sort(), ['b']);
});

test('countHiddenServers tallies bad and missing', () => {
    const scope = [{ id: 'a' }, { id: 'b' }, { id: 'c' }];
    const pings = { a: 50, b: null, c: 120 };
    assert.equal(countHiddenServers(scope, pings), 2);
});

