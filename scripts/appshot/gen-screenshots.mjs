// Regenerate the CSW app screenshots in website/assets/ from the real Tauri UI.
//
// The UI (crates/desktop/ui) runs as a "dumb terminal": with no window.__TAURI__
// present it falls back to a dev mock (devInvoke in main.js), so a plain headless
// Chrome renders the real screens with sample data. We drive it over the DevTools
// Protocol and capture at the same recipe as the shipped assets:
//   760 CSS width, deviceScaleFactor 2 (=> 1520px wide), dark theme.
// overview/onboarding grow the window to fit their content (the app is a fixed
// viewport with internal scroll + a pinned footer); hero is a 760x498 crop.
//
// Usage: node scripts/appshot/gen-screenshots.mjs
// Requires: Google Chrome, Node 21+ (global WebSocket/fetch).
//
// NOTE: this generates the Japanese (real app) screenshots (ja_*.png). The English
// assets (screen_*.png / hero.png) show an English UI that the app has no runtime
// mode for; they were produced by injecting an English string map. That map is not
// yet reconstructed here, so English screenshots are a TODO (see README.md).
import http from 'http';
import fs from 'fs';
import path from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..');
const UI = path.join(REPO, 'crates', 'desktop', 'ui');
const OUT = path.join(REPO, 'website', 'assets');
const CHROME = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const PORT = 8191, DP = 9341;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const types = { '.html': 'text/html; charset=utf-8', '.css': 'text/css', '.js': 'text/javascript', '.png': 'image/png', '.svg': 'image/svg+xml' };

const server = http.createServer((req, res) => {
  let u = decodeURIComponent(req.url.split('?')[0]); if (u.endsWith('/')) u += 'index.html';
  const fp = path.normalize(path.join(UI, u)); if (!fp.startsWith(UI)) { res.statusCode = 403; return res.end('x'); }
  fs.readFile(fp, (e, d) => { if (e) { res.statusCode = 404; return res.end('nf'); } res.setHeader('Content-Type', types[path.extname(fp)] || 'application/octet-stream'); res.end(d); });
}).listen(PORT);
const chrome = spawn(CHROME, ['--headless=new', '--hide-scrollbars', '--no-first-run', '--no-default-browser-check', `--remote-debugging-port=${DP}`, '--user-data-dir=/tmp/csw-appshot', 'about:blank']);

let id = 0; const pending = new Map(); const errors = [];
async function getWs() { for (let i = 0; i < 60; i++) { try { const r = await fetch(`http://127.0.0.1:${DP}/json/list`); const j = await r.json(); const p = j.find((t) => t.type === 'page' && t.webSocketDebuggerUrl); if (p) return p.webSocketDebuggerUrl; } catch (e) {} await sleep(200); } throw new Error('no devtools'); }

async function main() {
  const ws = new WebSocket(await getWs());
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && pending.has(d.id)) { pending.get(d.id)(d.result); pending.delete(d.id); } if (d.method === 'Runtime.exceptionThrown') errors.push(d.params.exceptionDetails.text); };
  const cmd = (method, params = {}) => new Promise((res) => { const i = ++id; pending.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
  const evalv = (expr) => cmd('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true });
  await cmd('Page.enable'); await cmd('Runtime.enable');
  await cmd('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: 'dark' }] });

  const load = async () => { await cmd('Emulation.setDeviceMetricsOverride', { width: 760, height: 900, deviceScaleFactor: 2, mobile: false }); await cmd('Page.navigate', { url: `http://127.0.0.1:${PORT}/index.html` }); await sleep(1300); };
  async function fit(scrollId, file) {
    const over = (await evalv(`(()=>{const s=document.getElementById('${scrollId}');return s?Math.ceil(s.scrollHeight-s.clientHeight):0;})()`)).result?.value || 0;
    await cmd('Emulation.setDeviceMetricsOverride', { width: 760, height: 900 + Math.max(0, over) + 24, deviceScaleFactor: 2, mobile: false });
    await sleep(300);
    const s = await cmd('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    fs.writeFileSync(path.join(OUT, file), Buffer.from(s.data, 'base64'));
  }
  async function crop(file, w, h) {
    await cmd('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 2, mobile: false });
    await sleep(250);
    const s = await cmd('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    fs.writeFileSync(path.join(OUT, file), Buffer.from(s.data, 'base64'));
  }
  const clickEnv = (name) => evalv(`(async()=>{const li=[...document.querySelectorAll('.profile-item')].find(e=>e.querySelector('.profile-name')?.textContent===${JSON.stringify(name)});li.click();await new Promise(r=>setTimeout(r,500));return 'ok';})()`);

  await load(); await evalv(`localStorage.removeItem('csw_onboarded'); checkFirstRun(); showEmpty();`); await sleep(400);
  await fit('emptyScroll', 'ja_screen_onboarding.png');

  await load(); await clickEnv('仕事用');
  await evalv(`(()=>{const s=[...document.querySelectorAll('.sharing-summary')].find(b=>/共有|分離/.test(b.textContent));if(s&&!s.closest('.disclosure').classList.contains('open'))s.click();})()`); await sleep(400);
  await fit('detailScroll', 'ja_screen_overview.png');

  await load(); await clickEnv('仕事用'); await sleep(300);
  await crop('ja_hero.png', 760, 498);

  console.log('regenerated ja_screen_onboarding.png, ja_screen_overview.png, ja_hero.png; errors:', errors.length ? JSON.stringify(errors) : 'none');
  ws.close();
}
main().then(() => { chrome.kill(); server.close(); process.exit(0); }).catch((e) => { console.error('FAIL', e); chrome.kill(); server.close(); process.exit(1); });
