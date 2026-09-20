// Source: https://material-theme.com/docs/reference/color-palette/
// Usage: node scripts/import-ui-palettes.mjs /path/to/color-palette.html
import { readFileSync, writeFileSync } from 'node:fs';
const html = readFileSync(process.argv[2], 'utf8');
const light = new Set([
  'lighter',
  'skyblue',
  'sandybeach',
  'github',
  'onelight',
  'solarlight',
  'lightowl',
  'prismLight',
  'horizonBright',
]);
const keys = {
  background: 'Background',
  foreground: 'Foreground',
  muted: 'Text',
  selection: 'Selection Background',
  onSelection: 'Selection Foreground',
  button: 'Buttons',
  surface: 'Second Background',
  disabled: 'Disabled',
  contrast: 'Contrast',
  active: 'Active',
  border: 'Border',
  highlight: 'Highlight',
  accent: 'Accent Color',
  error: 'Error Color',
  green: 'Green Color',
  yellow: 'Yellow Color',
  blue: 'Blue Color',
  purple: 'Purple Color',
};
const palettes = [
  ...html.matchAll(/<li class="card themed single ([^"]+)"[^>]*>([\s\S]*?)<\/li>/g),
].map(([, id, body]) => {
  const name = body.match(/<span class="card-title">(.*?)<\/span>/)[1];
  const raw = Object.fromEntries(
    [...body.matchAll(/<p[^>]*>([^<:]+):\s*(#[0-9a-fA-F]+)<\/p>/g)].map(([, key, value]) => [
      key,
      value,
    ]),
  );
  const colors = Object.fromEntries(
    Object.entries(keys).map(([key, source]) => {
      if (!raw[source]) throw new Error(id + ': missing ' + source);
      return [key, raw[source]];
    }),
  );
  return { id, name, mode: light.has(id) ? 'light' : 'dark', colors };
});
if (palettes.length < 29) throw new Error('Incomplete palette source');
writeFileSync(
  new URL('../src/app/common/model/ui-palettes.json', import.meta.url),
  JSON.stringify(palettes, null, 2) + '\n',
);
console.log('Imported ' + palettes.length + ' official palettes');
