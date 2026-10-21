// Pflegt das Icon-Set unter src/lib/icons:
//   node scripts/icons/build-icon-index.mjs          schreibt icon-names.ts und overview.html neu
//   node scripts/icons/build-icon-index.mjs --check  prueft Namen, Aufbau der SVGs und ob icon-names.ts aktuell ist (Exit-Code 1 bei Fehlern)
import { readdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const iconsDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'src', 'lib', 'icons');
const checkOnly = process.argv.includes('--check');

const WRAPPER_OPEN =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">';
const NAME_PATTERN = /^[a-z]+-[a-z0-9]+(-[a-z0-9]+)*$/;

const categories = readdirSync(iconsDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();

const icons = categories.flatMap((category) =>
  readdirSync(join(iconsDir, category))
    .filter((file) => file.endsWith('.svg'))
    .sort()
    .map((file) => ({ category, name: file.slice(0, -4), svg: readFileSync(join(iconsDir, category, file), 'utf8') }))
);

const problems = [];
const seen = new Map();
for (const { category, name, svg } of icons) {
  const where = `${category}/${name}.svg`;
  if (!NAME_PATTERN.test(name)) problems.push(`${where}: Name entspricht nicht <kategorie>-<name> in kebab-case`);
  if (!name.startsWith(`${category}-`)) problems.push(`${where}: Name muss mit "${category}-" beginnen`);
  if (!svg.startsWith(WRAPPER_OPEN)) problems.push(`${where}: Wurzelelement weicht vom Standard ab (viewBox, stroke, linecap ...)`);
  if (!svg.trimEnd().endsWith('</svg>')) problems.push(`${where}: schliessendes </svg> fehlt`);
  if (/#[0-9a-fA-F]{3,8}\b|rgb\(|hsl\(|style=|<script|\son\w+=/.test(svg)) problems.push(`${where}: feste Farben, style oder Skripte sind nicht erlaubt (nur currentColor)`);
  const body = svg.slice(WRAPPER_OPEN.length);
  if (seen.has(body)) problems.push(`${where}: identisch mit ${seen.get(body)}`);
  seen.set(body, where);
}

const namesFile = join(iconsDir, 'icon-names.ts');
const namesSource =
  `// Automatisch erzeugt von scripts/icons/build-icon-index.mjs - nicht von Hand aendern.\n` +
  `export const iconNames = [\n${icons.map((icon) => `  '${icon.name}'`).join(',\n')}\n] as const;\n\n` +
  `export type IconName = (typeof iconNames)[number];\n`;

if (checkOnly) {
  const current = existsSync(namesFile) ? readFileSync(namesFile, 'utf8') : '';
  if (current.replace(/\r\n/g, '\n') !== namesSource) problems.push('icon-names.ts ist nicht aktuell (Skript ohne --check ausfuehren)');
  for (const problem of problems) console.error(`FEHLER ${problem}`);
  console.log(`${icons.length} Icons in ${categories.length} Kategorien geprueft, ${problems.length} Fehler.`);
  process.exit(problems.length === 0 ? 0 : 1);
}

writeFileSync(namesFile, namesSource);

const sheet = (foreground, background) =>
  categories
    .map((category) => {
      const items = icons
        .filter((icon) => icon.category === category)
        .map((icon) => `<figure><div class="icon">${icon.svg}</div><figcaption>${icon.name.slice(category.length + 1)}</figcaption></figure>`)
        .join('');
      return `<section style="color:${foreground};background:${background}"><h2>${category}</h2><div class="grid">${items}</div></section>`;
    })
    .join('');

const html = `<!doctype html>
<html lang="de"><head><meta charset="utf-8"><title>Icon-Set Klipper-Touch</title>
<style>
body{margin:0;font-family:system-ui,sans-serif}
h1{margin:16px 24px;font-size:20px}
section{padding:12px 24px 20px}
h2{margin:0 0 8px;font-size:14px;text-transform:uppercase;letter-spacing:.08em;opacity:.7}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(104px,1fr));gap:8px}
figure{margin:0;text-align:center}
.icon svg{width:48px;height:48px}
figcaption{font-size:11px;opacity:.8;word-break:break-all}
</style></head><body>
<h1>Icon-Set Klipper-Touch (${icons.length} Icons) - erzeugt von scripts/icons/build-icon-index.mjs</h1>
${sheet('#000', '#fff')}
${sheet('#fff', '#222')}
</body></html>
`;
writeFileSync(join(iconsDir, 'overview.html'), html);
console.log(`${icons.length} Icons in ${categories.length} Kategorien; icon-names.ts und overview.html geschrieben${problems.length ? `; ${problems.length} Hinweise:\n${problems.join('\n')}` : ''}`);
