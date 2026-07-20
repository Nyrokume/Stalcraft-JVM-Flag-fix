/**
 * Refresh bundled Server Blocker catalogs from live endpoints.
 *
 * RU:  https://backend.stalcraftx.ru/address_list?login=User
 * EU/NA/SEA: unofficial-stalzone-api static address_list mirrors
 *
 * Usage: node scripts/refresh-server-catalog.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const dataDir = path.join(root, 'src', 'data');

const RU_URL = 'https://backend.stalcraftx.ru/address_list?login=User';
const ROXY_BASE =
    'https://raw.githubusercontent.com/Art3mLapa/unofficial-stalzone-api/main/static/address_list';

const ADDR_RE = /^\d{1,3}(\.\d{1,3}){3}:\d+$/;

async function getJson(url) {
    const res = await fetch(url, {
        headers: { Accept: 'application/json' },
        signal: AbortSignal.timeout(20_000),
    });
    if (!res.ok) throw new Error(`${url} → HTTP ${res.status}`);
    return res.json();
}

function tunnelList(catalog) {
    return (catalog?.pools ?? []).flatMap((p) =>
        (p.tunnels ?? []).map((t) => ({
            pool: p.name,
            region: p.region ?? '',
            name: t.name,
            address: String(t.address ?? ''),
        })),
    );
}

function tunnelKey(t) {
    return `${t.pool}::${t.name}@${t.address}`;
}

function validateCatalog(catalog, label) {
    if (!Array.isArray(catalog?.pools) || catalog.pools.length === 0) {
        throw new Error(`${label}: empty pools`);
    }
    for (const t of tunnelList(catalog)) {
        if (!t.name) throw new Error(`${label}: tunnel missing name in ${t.pool}`);
        if (!ADDR_RE.test(t.address)) {
            throw new Error(`${label}: bad address ${t.address} (${t.pool}::${t.name})`);
        }
    }
}

function stampPools(pools, region) {
    return (pools ?? []).map((p) => ({
        name: p.name,
        region,
        tunnels: (p.tunnels ?? []).map((t) => ({
            name: t.name,
            address: String(t.address),
        })),
    }));
}

function printDiff(label, before, after) {
    const a = new Set(tunnelList(before).map(tunnelKey));
    const b = new Set(tunnelList(after).map(tunnelKey));
    const added = [...b].filter((k) => !a.has(k));
    const removed = [...a].filter((k) => !b.has(k));
    console.log(
        `${label}: ${tunnelList(before).length} → ${tunnelList(after).length} tunnels (+${added.length} / -${removed.length})`,
    );
    for (const k of added) console.log(`  + ${k}`);
    for (const k of removed) console.log(`  - ${k}`);
}

const [ruLive, eu, na, sea] = await Promise.all([
    getJson(RU_URL),
    getJson(`${ROXY_BASE}/EU.json`),
    getJson(`${ROXY_BASE}/NA.json`),
    getJson(`${ROXY_BASE}/SEA.json`),
]);

const ruPath = path.join(dataDir, 'servers-ru.json');
const roxyPath = path.join(dataDir, 'servers.json');
const oldRu = JSON.parse(fs.readFileSync(ruPath, 'utf8'));
const oldRoxy = JSON.parse(fs.readFileSync(roxyPath, 'utf8'));

const ruOut = {
    mode: ruLive.mode || 'roxy',
    source: RU_URL,
    pools: (ruLive.pools ?? []).map((p) => ({
        name: p.name,
        tunnels: (p.tunnels ?? []).map((t) => ({
            name: t.name,
            address: String(t.address),
        })),
    })),
    clientToTunnelRttWeight: ruLive.clientToTunnelRttWeight ?? 1,
};

const roxyOut = {
    mode: 'roxy',
    source: `${ROXY_BASE}/{EU,NA,SEA}.json`,
    pools: [
        ...stampPools(eu.pools, 'EU'),
        ...stampPools(na.pools, 'NA'),
        ...stampPools(sea.pools, 'SEA'),
    ],
    clientToTunnelRttWeight: 1,
};

validateCatalog(ruOut, 'RU');
validateCatalog(roxyOut, 'roxy');

printDiff('RU', oldRu, ruOut);
printDiff('EU/NA/SEA', oldRoxy, roxyOut);

fs.writeFileSync(ruPath, `${JSON.stringify(ruOut, null, 2)}\n`);
fs.writeFileSync(roxyPath, `${JSON.stringify(roxyOut, null, 2)}\n`);

const ruN = tunnelList(ruOut).length;
const roxyN = tunnelList(roxyOut).length;
console.log(`OK written: RU=${ruN}, roxy=${roxyN}, merged=${ruN + roxyN}`);
console.log(`  ${ruPath}`);
console.log(`  ${roxyPath}`);
