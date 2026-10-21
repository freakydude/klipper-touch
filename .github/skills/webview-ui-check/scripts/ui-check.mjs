// UI-Smoke-Check: steuert die Tauri-WebView per Chrome DevTools Protocol und den Fake-Moonraker per JSON-RPC.
// Aufruf: node ui-check.mjs <ausgabeordner> [BxH,BxH,...]   (Standard-Viewports: 480x320,800x480)
// Voraussetzung: App mit WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 gegen den Fake gestartet (siehe run-ui-check.ps1).
import { mkdirSync, writeFileSync } from 'node:fs';

const outDir = process.argv[2];
const viewports = (process.argv[3] ?? '480x320,800x480').split(',').map((v) => v.split('x').map(Number));
const CDP_PORT = 9222;
const FAKE_URL = 'ws://127.0.0.1:7125/websocket';
const ROUTES = ['/printstate', '/move', '/temperature', '/babysteps', '/extrusion'];

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
mkdirSync(outDir, { recursive: true });

async function openCdp() {
  const targets = await (await fetch(`http://127.0.0.1:${CDP_PORT}/json`)).json();
  // Debug-Builds oeffnen zusaetzlich ein DevTools-Fenster; das ist nicht die App.
  const target = targets.find((t) => t.type === 'page' && !t.url.includes('devtools'));
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve) => (socket.onopen = resolve));
  return socket;
}

const cdp = await openCdp();
let nextId = 1;
const pending = new Map();
const problems = [];
cdp.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) {
    pending.get(message.id)(message);
    pending.delete(message.id);
    return;
  }
  const { method, params } = message;
  if (method === 'Runtime.consoleAPICalled' && ['error', 'warning', 'assert'].includes(params.type)) {
    problems.push(`console.${params.type}: ${params.args.map((a) => a.value ?? a.description ?? '').join(' ')}`.slice(0, 300));
  } else if (method === 'Runtime.exceptionThrown') {
    problems.push(`exception: ${params.exceptionDetails.exception?.description ?? params.exceptionDetails.text}`.slice(0, 300));
  } else if (method === 'Log.entryAdded' && ['error', 'warning'].includes(params.entry.level)) {
    problems.push(`log.${params.entry.level}: ${params.entry.text} ${params.entry.url ?? ''}`.slice(0, 300));
  }
};
const send = (method, params = {}) =>
  new Promise((resolve) => {
    const id = nextId++;
    pending.set(id, resolve);
    cdp.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expression) => (await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true })).result?.result?.value;

const fake = new WebSocket(FAKE_URL);
await new Promise((resolve) => (fake.onopen = resolve));
const fakePending = new Map();
let fakeId = 1000;
fake.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.id && fakePending.has(message.id)) fakePending.get(message.id)(message);
};
const rpc = (method, params = {}) =>
  new Promise((resolve) => {
    const id = String(fakeId++);
    fakePending.set(id, resolve);
    fake.send(JSON.stringify({ jsonrpc: '2.0', id, method, params }));
  });

const shot = async (name) => {
  const result = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(`${outDir}/${name}.png`, Buffer.from(result.result.data, 'base64'));
};
const go = async (path) => {
  await evaluate(`document.querySelector('a[href="${path}"]')?.click()`);
  await sleep(1200);
};
const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll('button')].find((e) => e.textContent.trim() === ${JSON.stringify(text)}); b?.click(); return !!b; })()`);
const buttons = () => evaluate(`JSON.stringify([...document.querySelectorAll('button')].map((b) => b.textContent.trim()))`);
const overflowing = () =>
  evaluate(`JSON.stringify([...document.querySelectorAll('button, a, p, td')].filter((e) => { const b = e.getBoundingClientRect(); return b.width > 0 && (b.right > innerWidth + 1 || b.bottom > innerHeight + 1); }).map((e) => e.textContent.trim().slice(0, 20)))`);
// Spiegelt scaleUI() aus +layout.svelte: 1rem = Hoehe/320*16 px.
const setViewport = async (width, height) => {
  await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
  await evaluate(`document.documentElement.style.fontSize = '${(height / 320) * 16}px'`);
  await sleep(500);
};

await send('Runtime.enable');
await send('Log.enable');
await send('Page.enable');

console.log('start path:', await evaluate('location.pathname'));
await sleep(1500);

for (const [width, height] of viewports) {
  await setViewport(width, height);
  for (const route of ROUTES) {
    await go(route);
    await shot(`vp${width}x${height}${route.replace('/', '-')}`);
    console.log(`overflow ${width}x${height} ${route}:`, await overflowing());
  }
}
await send('Emulation.clearDeviceMetricsOverride');
await go('/printstate');

await go('/temperature');
await rpc('printer.gcode.script', { script: 'M104 S200\nM140 S60' });
await sleep(4000);
await shot('heating');

await go('/printstate');
await clickText('Load');
await sleep(2500);
await shot('load-dialog');
console.log('file rows:', await evaluate(`document.querySelectorAll('img').length`), 'images, loaded:', await evaluate(`[...document.querySelectorAll('img')].filter((i) => i.complete && i.naturalWidth > 0).length`));
await clickText('Close');

await rpc('printer.print.start', { filename: 'benchy.gcode' });
await sleep(3000);
await go('/printstate');
await shot('printing');
console.log('buttons while printing:', await buttons());

await rpc('printer.print.pause');
await sleep(1500);
await shot('paused');
await rpc('printer.print.cancel');
await sleep(1500);
await shot('cancelled');
await rpc('printer.emergency_stop');
await sleep(2000);
console.log('path after emergency stop:', await evaluate('location.pathname'));
await shot('emergency-stop');

console.log(`--- problems: ${problems.length}`);
for (const problem of new Set(problems)) console.log(problem);
process.exit(problems.length === 0 ? 0 : 2);
