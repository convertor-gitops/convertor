import { readdir, readFile } from 'node:fs/promises';
import { resolve, relative } from 'node:path';
const root = resolve(import.meta.dirname, '../src/app');
async function files(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  return (
    await Promise.all(
      entries.map((e) => (e.isDirectory() ? files(resolve(dir, e.name)) : [resolve(dir, e.name)])),
    )
  ).flat();
}
const failures = [];
for (const file of await files(root)) {
  if (!/\.(ts|html|scss)$/.test(file)) continue;
  const path = relative(root, file),
    source = await readFile(file, 'utf8');
  if (/\bUi[A-Z]\w*/.test(source)) failures.push(`${path}: use uppercase UI prefix`);
  const consumer = /^page\/(plan-board|ui-component)\//.test(path);
  if (!consumer || file.endsWith('.spec.ts')) continue;
  if (/from\s+['"]@angular\/material\//.test(source))
    failures.push(`${path}: consume shared components, not Material directly`);
  if (/::ng-deep/.test(source))
    failures.push(`${path}: consumer must not style component internals`);
  if (/^page\/plan-board\//.test(path) && /from\s+['"][^'"]*ui-component\//.test(source))
    failures.push(`${path}: business page depends on showcase`);
  if (file.endsWith('.html') && /<(?:mat-[\w-]+|details|summary)\b/.test(source))
    failures.push(`${path}: local control bypasses shared component`);
}
if (failures.length) {
  console.error(failures.join('\n'));
  process.exitCode = 1;
} else console.log('Shared UI architecture checks passed.');
