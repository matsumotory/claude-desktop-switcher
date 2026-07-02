// Regenerate the OG card images (website/assets/og.png, ja_og.png) from an
// HTML/CSS template, so the copy baked into the images can be kept in sync with
// the LP by editing text here and re-running (no image editor, no generative AI).
//
// Design mirrors the shipped OG cards: black canvas, one rounded dark card,
// logo + wordmark, ZERO-IMPACT ISOLATION eyebrow, two-line headline, two-line
// subtext, three mode chips, URL and a macOS pill. Copy follows the LP hero
// (website/index.html / website/ja/index.html) and the copy rules in
// CLAUDE.md §5 (no em-dash / ※ / decorative symbols in user-visible text).
//
// Usage: node scripts/ogshot/gen-og.mjs
// Requires: Google Chrome, Node 21+ (global WebSocket/fetch), network for the
// Outfit webfont (same font as the LP).
import http from 'http';
import fs from 'fs';
import path from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..');
const ASSETS = path.join(REPO, 'website', 'assets');
const CHROME = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const PORT = 8193, DP = 9343;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const LOCALES = [
  {
    file: 'og.png',
    lang: 'en',
    headline: 'Separate your Claude<br>Desktop App by use case.',
    sub: 'Keep chat, Projects, Claude Cowork, Artifacts, and Claude Design apart by account, with Claude Code linked when you want it.',
    chips: ['Account only', 'Conversations & memory', 'Everything'],
    url: 'matsumotory.github.io/claude-desktop-switcher',
    headSize: 64,
  },
  {
    file: 'ja_og.png',
    lang: 'ja',
    headline: 'Claudeデスクトップアプリを<br>用途ごとに分ける。',
    sub: 'チャット・Projects・Claude Cowork・Artifacts・Claude Design を<span class="nb">アカウント別に分離できます。</span>必要なら <span class="nb">Claude Code とも連動できます。</span>',
    chips: ['アカウントだけ', '会話とメモリも', 'すべて'],
    url: 'matsumotory.github.io/claude-desktop-switcher/ja',
    headSize: 56,
  },
];

const page = ({ lang, headline, sub, chips, url, headSize }) => `<!DOCTYPE html>
<html lang="${lang}"><head><meta charset="UTF-8">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Outfit:wght@400;500;600&display=swap" rel="stylesheet">
<style>
  * { margin: 0; box-sizing: border-box; }
  body { width: 1200px; height: 630px; background: #050505;
    font-family: 'Outfit', 'Hiragino Sans', 'Hiragino Kaku Gothic ProN', sans-serif; }
  .card { position: absolute; inset: 30px; border-radius: 28px;
    border: 1px solid rgba(255,255,255,0.08);
    background: radial-gradient(120% 140% at 12% 0%, #171513 0%, #0b0a09 55%, #070606 100%);
    padding: 56px 68px 0; overflow: hidden; }
  .brand { display: flex; align-items: center; gap: 14px; }
  .brand img { width: 34px; height: 34px; object-fit: contain; }
  .brand span { color: #f5f2ee; font-size: 25px; font-weight: 600; letter-spacing: 0.01em; }
  .eyebrow { display: flex; align-items: center; gap: 14px; margin: 22px 0 0; }
  .eyebrow .rule { width: 26px; height: 1px; background: rgba(255,255,255,0.35); }
  .eyebrow span { color: #8f8b86; font-size: 14px; font-weight: 500;
    letter-spacing: 0.22em; text-transform: uppercase; }
  h1 { color: #ffffff; font-size: ${headSize}px; font-weight: 600; line-height: 1.22;
    letter-spacing: ${lang === 'ja' ? '0.01em' : '-0.01em'}; margin-top: 22px;
    line-break: strict; }
  .sub { color: #a7a29c; font-size: 24px; font-weight: 400; line-height: 1.5;
    margin-top: 20px; max-width: 980px; line-break: strict; }
  .nb { white-space: nowrap; }
  .chips { display: flex; gap: 14px; margin-top: 26px; }
  .chip { color: #d8d4cf; font-size: 16.5px; font-weight: 500;
    border: 1px solid rgba(255,255,255,0.14); border-radius: 999px;
    padding: 10px 20px; background: rgba(255,255,255,0.03); }
  .foot { position: absolute; left: 68px; right: 68px; bottom: 34px;
    display: flex; align-items: center; justify-content: space-between; }
  .foot .url { color: #7d7973; font-size: 17px; }
  .foot .pill { color: #d8d4cf; font-size: 16.5px; font-weight: 500;
    border: 1px solid rgba(255,255,255,0.14); border-radius: 999px;
    padding: 10px 20px; background: rgba(255,255,255,0.03); }
</style></head>
<body><div class="card">
  <div class="brand"><img src="http://127.0.0.1:${PORT}/logo.png" alt=""><span>Claude Desktop Switcher</span></div>
  <div class="eyebrow"><span class="rule"></span><span>Zero-Impact Isolation</span></div>
  <h1>${headline}</h1>
  <p class="sub">${sub}</p>
  <div class="chips">${chips.map((c) => `<span class="chip">${c}</span>`).join('')}</div>
  <div class="foot"><span class="url">${url}</span><span class="pill">macOS &middot; .dmg</span></div>
</div></body></html>`;

const server = http.createServer((req, res) => {
  const u = decodeURIComponent(req.url.split('?')[0]);
  if (u === '/logo.png') {
    res.setHeader('Content-Type', 'image/png');
    return res.end(fs.readFileSync(path.join(ASSETS, 'logo.png')));
  }
  const m = u.match(/^\/og-(\w+)\.html$/);
  const loc = m && LOCALES.find((l) => l.lang === m[1]);
  if (!loc) { res.statusCode = 404; return res.end('nf'); }
  res.setHeader('Content-Type', 'text/html; charset=utf-8');
  res.end(page(loc));
}).listen(PORT);

const chrome = spawn(CHROME, ['--headless=new', '--hide-scrollbars', '--no-first-run',
  '--no-default-browser-check', `--remote-debugging-port=${DP}`,
  '--user-data-dir=/tmp/csw-ogshot', 'about:blank']);

let id = 0; const pending = new Map();
async function getWs() {
  for (let i = 0; i < 60; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${DP}/json/list`);
      const p = (await r.json()).find((t) => t.type === 'page' && t.webSocketDebuggerUrl);
      if (p) return p.webSocketDebuggerUrl;
    } catch (e) { /* retry */ }
    await sleep(200);
  }
  throw new Error('no devtools');
}

async function main() {
  const ws = new WebSocket(await getWs());
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && pending.has(d.id)) { pending.get(d.id)(d.result); pending.delete(d.id); } };
  const cmd = (method, params = {}) => new Promise((res) => { const i = ++id; pending.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
  await cmd('Page.enable');
  await cmd('Emulation.setDeviceMetricsOverride', { width: 1200, height: 630, deviceScaleFactor: 1, mobile: false });

  const done = [];
  for (const loc of LOCALES) {
    await cmd('Page.navigate', { url: `http://127.0.0.1:${PORT}/og-${loc.lang}.html` });
    await sleep(1600); // webfont + layout
    const s = await cmd('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    fs.writeFileSync(path.join(ASSETS, loc.file), Buffer.from(s.data, 'base64'));
    done.push(loc.file);
  }
  console.log('regenerated:', done.join(', '));
  ws.close();
}
main().then(() => { chrome.kill(); server.close(); process.exit(0); }).catch((e) => { console.error('FAIL', e); chrome.kill(); server.close(); process.exit(1); });
