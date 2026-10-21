import type { IconName } from './icon-names';

const svgByPath = import.meta.glob<string>('./*/*.svg', { query: '?raw', import: 'default', eager: true });

/** Name (`<kategorie>-<name>`) -> SVG-Quelltext, aufgebaut aus den Dateien unter `src/lib/icons/<kategorie>/`. */
export const iconSvgByName: Record<string, string> = Object.fromEntries(
  Object.entries(svgByPath).map(([path, svg]) => [path.slice(path.lastIndexOf('/') + 1, -'.svg'.length), svg])
);

export function iconSvg(name: IconName): string {
  return iconSvgByName[name];
}
