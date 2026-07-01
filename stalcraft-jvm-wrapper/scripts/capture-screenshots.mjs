import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const outDir = path.join(root, 'docs', 'screenshots');
const port = 4173;
const base = `http://127.0.0.1:${port}`;

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

async function shot(page, name) {
    await page.screenshot({ path: path.join(outDir, name), fullPage: false });
}

async function main() {
    await mkdir(outDir, { recursive: true });
    const preview = await startPreview();
    const browser = await chromium.launch();
    const page = await browser.newPage({ viewport: { width: 1350, height: 1030 } });

    try {
        await page.goto(base, { waitUntil: 'networkidle' });
        await page.evaluate(() => {
            localStorage.setItem('stalcraft-jvm-welcome-hidden', '1');
            localStorage.setItem('stalcraft-jvm-lang', 'ru');
        });
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForSelector('.loading-screen.hidden', { timeout: 12000 }).catch(() => {});
        await page.waitForTimeout(500);
        await shot(page, 'main-ru.png');

        await page.evaluate(() => localStorage.setItem('stalcraft-jvm-lang', 'en'));
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForSelector('.loading-screen.hidden', { timeout: 12000 }).catch(() => {});
        await page.waitForTimeout(500);
        await shot(page, 'main-en.png');

        await page.evaluate(() => {
            localStorage.removeItem('stalcraft-jvm-welcome-hidden');
            document.getElementById('welcome-modal')?.classList.remove('hidden');
        });
        await page.waitForTimeout(300);
        await shot(page, 'license-ru.png');
    } finally {
        await browser.close();
        preview.kill('SIGTERM');
    }

    console.log('Screenshots saved to', outDir);
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});
