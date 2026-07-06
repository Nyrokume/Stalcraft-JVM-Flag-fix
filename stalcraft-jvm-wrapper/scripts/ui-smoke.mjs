import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const port = Number(process.env.UI_PORT || 4173);
const base = process.env.UI_BASE || `http://127.0.0.1:${port}`;

function startPreview() {
    return new Promise((resolve, reject) => {
        const proc = spawn('npx', ['vite', 'preview', '--port', String(port), '--strictPort'], {
            cwd: root,
            shell: true,
            stdio: 'pipe',
        });
        let ready = false;
        const onData = (buf) => {
            const s = buf.toString();
            if (!ready && /Local:\s+http/.test(s)) {
                ready = true;
                resolve(proc);
            }
        };
        proc.stdout.on('data', onData);
        proc.stderr.on('data', onData);
        proc.on('error', reject);
        setTimeout(() => {
            if (!ready) resolve(proc);
        }, 8000);
    });
}

const preview = await startPreview();
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1350, height: 1030 } });

const results = [];

async function check(name, fn) {
    try {
        await fn();
        results.push({ name, ok: true });
    } catch (e) {
        results.push({ name, ok: false, err: e.message });
    }
}

async function dismissWelcome() {
    const licenseVisible = await page.locator('#license-modal:not(.hidden)').count();
    if (!licenseVisible) return;
    await page.locator('label:has(#welcome-accept)').click();
    await page.locator('#license-ok').click();
    await page.waitForSelector('#info-modal:not(.hidden)', { timeout: 5000 });
    await page.locator('#info-ok').click();
    await page.waitForSelector('#info-modal.hidden', { state: 'attached', timeout: 5000 });
}

try {
    await page.goto(base, { waitUntil: 'networkidle' });
    await page.evaluate(() => {
        localStorage.setItem('stalcraft-jvm-lang', 'ru');
        localStorage.removeItem('stalcraft-sb-settings');
        localStorage.removeItem('stalcraft-jvm-welcome-v1');
        localStorage.removeItem('stalcraft-jvm-sb-warning-v1');
    });
    await page.reload({ waitUntil: 'networkidle' });
    await page.waitForSelector('.loading-screen.hidden', { timeout: 15000 }).catch(() => {});
    await page.waitForSelector('#license-modal:not(.hidden)', { timeout: 10000 }).catch(() => {});
    await dismissWelcome();

    await check('Welcome not shown after reload', async () => {
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForSelector('.loading-screen.hidden', { timeout: 15000 }).catch(() => {});
        await page.waitForTimeout(400);
        const licenseOpen = await page.locator('#license-modal:not(.hidden)').count();
        const infoOpen = await page.locator('#info-modal:not(.hidden)').count();
        if (licenseOpen || infoOpen) throw new Error('welcome modals shown again after reload');
    });

    await check('JVM page visible', async () => {
        if (!(await page.locator('#page-jvm').isVisible())) throw new Error('page-jvm not visible');
    });

    await check('Nav buttons', async () => {
        const count = await page.locator('.app-nav-btn').count();
        if (count !== 2) throw new Error(`expected 2 nav buttons, got ${count}`);
    });

    await check('Hardware panel', async () => {
        if (!(await page.locator('.hardware-profile').isVisible())) throw new Error('missing hardware panel');
    });

    await check('IFEO buttons', async () => {
        for (const id of ['install-btn', 'uninstall-btn', 'verify-btn']) {
            if (!(await page.locator(`#${id}`).isVisible())) throw new Error(`missing #${id}`);
        }
    });

    await check('Config controls', async () => {
        if (!(await page.locator('#config-select').isVisible())) throw new Error('missing config select');
    });

    await check('System log', async () => {
        if (!(await page.locator('#log-container').isVisible())) throw new Error('missing log');
    });

    await check('Server blocker page', async () => {
        await page.locator('#nav-server-blocker').click();
        const sbWarn = page.locator('#sb-warning-modal:not(.hidden)');
        if (await sbWarn.count()) {
            await page.locator('label:has(#sb-warning-accept)').click();
            await page.locator('#sb-warning-ok').click();
            await page.waitForFunction(
                () => document.getElementById('sb-warning-modal')?.classList.contains('hidden'),
                { timeout: 3000 },
            );
        }
        await page.waitForSelector('#page-server-blocker.active', { timeout: 3000 });
        if (!(await page.locator('.sb-topbar').isVisible())) throw new Error('missing topbar');
        const chips = await page.locator('.sb-chip').count();
        if (chips !== 4) throw new Error(`expected 4 region chips (RU+EU+NA+SEA), got ${chips}`);
        await page.locator('#sb-reset-btn').click();
        await page.waitForTimeout(120);
    });

    await check('Server cards default RU = 59', async () => {
        const cards = await page.locator('.sb-card').count();
        if (cards !== 59) throw new Error(`expected 59 RU cards by default, got ${cards}`);
    });

    await check('Region counts RU/EU/NA/SEA', async () => {
        for (const [region, expected] of [['RU', 59], ['EU', 10], ['NA', 5], ['SEA', 3]]) {
            await page.locator(`.sb-chip[data-region="${region}"]`).click();
            await page.waitForTimeout(80);
            const cards = await page.locator('.sb-card').count();
            if (cards !== expected) throw new Error(`${region}: expected ${expected}, got ${cards}`);
        }
    });

    await check('RU pool zones = 14', async () => {
        await page.locator('.sb-chip[data-region="RU"]').click();
        const zones = await page.locator('.sb-pool-zone').count();
        if (zones !== 14) throw new Error(`expected 14 RU pool zones, got ${zones}`);
    });

    await check('Block toggle updates badge', async () => {
        await page.locator('.sb-chip[data-region="RU"]').click();
        await page.locator('#sb-reset-btn').click();
        await page.waitForTimeout(100);
        await page.locator('.sb-switch').first().click();
        await page.waitForTimeout(100);
        const blocked = await page.locator('#sb-blocked-badge').textContent();
        if (blocked !== '1') throw new Error(`expected badge 1, got ${blocked}`);
    });

    await check('Show blocked filter', async () => {
        await page.locator('.sb-chip[data-region="RU"]').click();
        await page.locator('label.sb-check').click();
        await page.waitForTimeout(100);
        const cards = await page.locator('.sb-card').count();
        if (cards < 1) throw new Error('show-blocked should list blocked cards');
        await page.locator('label.sb-check').click();
    });

    await check('Search filter', async () => {
        await page.locator('.sb-chip[data-region="EU"]').click();
        await page.locator('#sb-search').fill('WAW-ROXY');
        await page.waitForTimeout(100);
        const cards = await page.locator('.sb-card').count();
        if (cards !== 3) throw new Error(`expected 3 WAW cards, got ${cards}`);
        await page.locator('#sb-search').fill('');
        await page.waitForTimeout(100);
    });

    await check('Ping progress and colored classes', async () => {
        await page.locator('.sb-chip[data-region="RU"]').click();
        await page.locator('#sb-ping-btn').click();
        await page.waitForFunction(
            () => document.getElementById('sb-topbar-viz')?.classList.contains('is-active'),
            { timeout: 8000 },
        ).catch(() => {});
        await page.waitForFunction(
            () => /Пинг \d+\/\d+|Pinging \d+\/\d+/.test(
                document.getElementById('sb-status-label')?.textContent ?? '',
            ),
            { timeout: 8000 },
        ).catch(() => {});
        await page.waitForTimeout(2500);
        const colored = await page.locator('.sb-card-ping--good, .sb-card-ping--mid').count();
        if (colored < 8) throw new Error(`expected ≥8 good/mid pings visible, got ${colored}`);
        const hidden = await page.locator('.sb-card-ping--bad, .sb-card-ping--very-bad').count();
        if (hidden > 0) throw new Error(`bad ping cards should be hidden, got ${hidden}`);
    });

    await check('Auto best EU', async () => {
        await page.locator('.sb-chip[data-region="EU"]').click();
        await page.locator('#sb-auto-best-btn').click();
        await page.waitForTimeout(1200);
        const best = await page.locator('.sb-card--best').count();
        if (best !== 2) throw new Error(`expected 2 best cards (EU pools), got ${best}`);
        const blocked = Number(await page.locator('#sb-blocked-badge').textContent());
        if (blocked !== 8) throw new Error(`expected 8 blocked in EU scope, got ${blocked}`);
    });

    await check('Blocking mock start/stop', async () => {
        await page.locator('.sb-chip[data-region="EU"]').click();
        await page.locator('#sb-auto-best-btn').click();
        await page.waitForTimeout(800);
        await page.locator('#sb-start-btn').click();
        await page.waitForTimeout(400);
        const active = await page.locator('#sb-status-label').textContent();
        if (!/активна|active/i.test(active ?? '')) throw new Error(`expected blocking active, got ${active}`);
        if (await page.locator('#sb-start-btn').isEnabled()) throw new Error('start should be disabled while active');
        await page.locator('#sb-stop-btn').click();
        await page.waitForTimeout(400);
        const idle = await page.locator('#sb-status-label').textContent();
        if (/активна|active/i.test(idle ?? '')) throw new Error(`expected idle after stop, got ${idle}`);
    });

    await check('Refresh RU button', async () => {
        if (!(await page.locator('#sb-refresh-btn').isVisible())) throw new Error('missing refresh btn');
        await page.locator('#sb-reset-btn').click();
        await page.locator('.sb-chip[data-region="RU"]').click();
        await page.locator('#sb-refresh-btn').click();
        await page.waitForTimeout(500);
        const cards = await page.locator('.sb-card').count();
        if (cards !== 59) throw new Error(`expected 59 RU cards after refresh, got ${cards}`);
    });

    await check('Region switch shows one section', async () => {
        await page.locator('#sb-search').fill('');
        await page.locator('.sb-chip[data-region="EU"]').click();
        await page.waitForTimeout(100);
        const regions = await page.locator('.sb-region').count();
        if (regions !== 1) throw new Error(`expected 1 region section for EU, got ${regions}`);
        const cards = await page.locator('.sb-card').count();
        if (cards !== 10) throw new Error(`expected 10 EU cards, got ${cards}`);
    });

    await check('Server blocker i18n EN', async () => {
        await page.locator('.custom-titlebar .lang-btn[data-lang="en"]').click();
        const text = await page.locator('.sb-topbar-title').textContent();
        if (!/Server Blocker/i.test(text)) throw new Error(`EN title: ${text}`);
    });

    await check('Back to JVM page', async () => {
        await page.locator('#nav-jvm').click();
        await page.waitForSelector('#page-jvm.active', { timeout: 3000 });
    });
} finally {
    await browser.close();
    preview.kill('SIGTERM');
}

const failed = results.filter((r) => !r.ok);
console.log(JSON.stringify(results, null, 2));
if (failed.length) {
    console.error(`${failed.length} check(s) failed`);
    process.exit(1);
}
console.log('All UI checks passed');
