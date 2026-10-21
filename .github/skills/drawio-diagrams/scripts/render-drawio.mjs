// Rendert eine Seite einer .drawio-Datei mit dem draw.io-Viewer und speichert einen Screenshot (per CDP).
// Aufruf: node render-drawio.mjs <datei.drawio> <seitenindex> <viewer-static.min.js> <ausgabe.png> [debugPort]
import { readFileSync, writeFileSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const [file, pageIndex, viewerJs, out, portArg] = process.argv.slice(2);
const port = Number(portArg ?? 9333);
const escapeAttribute = (text) => text.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

const config = { highlight: '#0000ff', nav: false, resize: false, toolbar: '', page: Number(pageIndex), xml: readFileSync(file, 'utf8') };
const html = `<html><body style="margin:0"><div class="mxgraph" data-mxgraph="${escapeAttribute(JSON.stringify(config))}"></div><script src="file:///${viewerJs.replaceAll('\\', '/')}"></script></body></html>`;
const htmlPath = join(tmpdir(), 'drawio-render.html');
writeFileSync(htmlPath, html);

const edge = [process.env['ProgramFiles(x86)'] + '\\Microsoft\\Edge\\Application\\msedge.exe', process.env.ProgramFiles + '\\Microsoft\\Edge\\Application\\msedge.exe'].find((p) => {
  try {
    readFileSync(p, { flag: 'r' }).length;
    return true;
  } catch {
    return false;
  }
});
const profile = join(tmpdir(), 'drawio-render-profile');
const browser = spawn(edge, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, '--no-first-run', '--disable-sync', '--allow-file-access-from-files', '--window-size=3400,4600', 'about:blank'], { stdio: 'ignore' });

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let target;
for (let attempt = 0; attempt < 40 && !target; attempt++) {
  await sleep(500);
  try {
    target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === 'page');
  } catch {
    // Browser startet noch
  }
}
if (!target) {
  browser.kill();
  throw new Error('Browser nicht erreichbar');
}

const cdp = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve) => (cdp.onopen = resolve));
let nextId = 1;
const pending = new Map();
cdp.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) pending.get(message.id)(message);
};
const send = (method, params = {}) =>
  new Promise((resolve) => {
    const id = nextId++;
    pending.set(id, resolve);
    cdp.send(JSON.stringify({ id, method, params }));
  });

await send('Emulation.setDeviceMetricsOverride', { width: 3400, height: 4600, deviceScaleFactor: 1, mobile: false });
await send('Page.navigate', { url: `file:///${htmlPath.replaceAll('\\', '/')}` });
await sleep(6000);
const size = (await send('Runtime.evaluate', { expression: 'JSON.stringify([document.documentElement.scrollWidth, document.documentElement.scrollHeight])', returnByValue: true })).result.result.value;
const shot = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
writeFileSync(out, Buffer.from(shot.result.data, 'base64'));
console.log('page size', size, 'saved', out);
browser.kill();
process.exit(0);
