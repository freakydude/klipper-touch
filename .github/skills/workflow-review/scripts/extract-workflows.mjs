// Liest die Seite "Workflows" aus einer .drawio-Datei und gibt Lanes, Workflows, Schritte, Tags und Prioritaeten als Text aus.
// Aufruf: node extract-workflows.mjs "<datei.drawio>" [Seitenname]
import { readFileSync } from 'node:fs';

const [file, pageName = 'Workflows'] = process.argv.slice(2);
const xml = readFileSync(file, 'utf8');
const pageStart = xml.search(new RegExp(`<diagram\\b[^>]*name="${pageName}"`));
if (pageStart < 0) throw new Error(`Seite "${pageName}" nicht gefunden`);
const page = xml.slice(pageStart, xml.indexOf('</diagram>', pageStart));

const attributes = (text) => Object.fromEntries([...text.matchAll(/([\w:]+)="([^"]*)"/g)].map((m) => [m[1], m[2]]));
const plain = (value = '') =>
  value
    .replace(/&lt;br\s*\/?&gt;/gi, ' ')
    .replace(/&lt;[^&]*?&gt;/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&quot;/g, '"')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/\s+/g, ' ')
    .trim();

const cells = [...page.matchAll(/<mxCell\b([^>]*?)(\/>|>\s*(?:<mxGeometry\b([^>]*?)\/?>)?)/g)]
  .map((m) => ({ ...attributes(m[1]), geo: attributes(m[3] ?? '') }))
  .filter((cell) => cell.vertex === '1');
const byId = new Map(cells.map((cell) => [cell.id, cell]));
const y = (cell) => Number(cell.geo.y ?? 0);
const x = (cell) => Number(cell.geo.x ?? 0);
const style = (cell) => cell.style ?? '';

const laneHeaders = cells.filter((cell) => cell.parent === '1' && style(cell).includes('fillColor=#333333')).sort((a, b) => y(a) - y(b));
const containers = cells.filter((cell) => style(cell).startsWith('swimlane'));

const tagKind = (cell) =>
  style(cell).includes('#d5e8d4') ? 'Klipper' : style(cell).includes('#e1d5e7') ? 'Klipper optional' : style(cell).includes('#dae8fc') ? 'Moonraker' : 'Tag';
const stepKind = (cell) => (style(cell).includes('dashed=1') ? ' [optional]' : style(cell).includes('#f8cecc') ? ' [Problem]' : '');

for (const header of laneHeaders) {
  const index = laneHeaders.indexOf(header);
  const nextY = laneHeaders[index + 1] ? y(laneHeaders[index + 1]) : Infinity;
  const workflows = containers.filter((c) => y(c) > y(header) && y(c) < nextY).sort((a, b) => x(a) - x(b));
  console.log(`\n## ${plain(header.value)}`);

  if (workflows.length === 0) {
    const rows = new Map();
    for (const cell of cells.filter((c) => c.parent === '1' && y(c) > y(header) && y(c) < nextY && plain(c.value))) {
      rows.set(y(cell), [...(rows.get(y(cell)) ?? []), cell]);
    }
    for (const [, row] of [...rows].sort((a, b) => a[0] - b[0])) console.log('  | ' + row.sort((a, b) => x(a) - x(b)).map((c) => plain(c.value)).join(' | '));
    continue;
  }

  for (const workflow of workflows) {
    const children = cells.filter((c) => c.parent === workflow.id);
    const priority = children.find((c) => /^P[123]$/.test(plain(c.value)));
    console.log(`\n### ${plain(workflow.value)} [${priority ? plain(priority.value) : 'ohne Prioritaet'}]`);
    const steps = children.filter((c) => style(c).includes('arcSize=20')).sort((a, b) => y(a) - y(b));
    const tags = children.filter((c) => style(c).includes('fontSize=10'));
    steps.forEach((step, number) => {
      const own = tags.filter((t) => y(t) >= y(step) - 30 && y(t) <= y(step) + Number(step.geo.height) + 30 && Math.abs(y(t) + 10 - (y(step) + Number(step.geo.height) / 2)) <= 40);
      const labels = own.map((t) => `${tagKind(t)}: ${plain(t.value)}`).join('; ');
      console.log(`  ${number + 1}. ${plain(step.value)}${stepKind(step)}${labels ? `  (${labels})` : ''}`);
    });
  }
}

const questions = cells.find((c) => plain(c.value).startsWith('Offene Fragen'));
if (questions) console.log(`\n## ${plain(questions.value)}`);
