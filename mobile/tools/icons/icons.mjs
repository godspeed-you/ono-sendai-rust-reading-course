#!/usr/bin/env node
// Regenerates every native icon and launch image from the vector source in resources/*.svg.
// Source of truth: the vectors below (the course's `>_` mark, the same mark as assets/favicon.svg).
// Outputs are committed native resources; this is only re-run when the artwork changes:
//   (cd mobile/tools/icons && npm ci) && node mobile/tools/icons/icons.mjs
// Its dependencies (sharp, @capacitor/assets) live in tools/icons/package.json, separate from the
// audited runtime/build dependencies in mobile/package.json.
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';

const HERE = dirname(fileURLToPath(import.meta.url));
const MOBILE = join(HERE, '..', '..');
const RES = join(MOBILE, 'resources');
mkdirSync(RES, { recursive: true });

const BG = '#0b1f2a';
const FG = '#5eead4';
const glyph = (scale, size) => `
  <g transform="translate(${size / 2} ${size / 2}) scale(${scale}) translate(-16.5 -16.5)" fill="none" stroke="${FG}"
     stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
    <path d="M9 11l6 5-6 5"/><path d="M17 22h7"/>
  </g>`;
const svg = (size, body) => `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">${body}</svg>`;
const bg = (size) => `<rect width="${size}" height="${size}" fill="${BG}"/>`;

const files = {
  // Full-bleed square (iOS applies its own mask; no transparency allowed).
  'icon-only.svg': svg(1024, bg(1024) + glyph(30, 1024)),
  // Android adaptive icon layers: the glyph stays inside the central 66% safe zone.
  'icon-foreground.svg': svg(1024, glyph(24, 1024)),
  'icon-background.svg': svg(1024, bg(1024)),
  // Launch screen: the mark on the course's dark colour, no text, no network.
  'splash.svg': svg(2732, bg(2732) + glyph(14, 2732)),
  'splash-dark.svg': svg(2732, bg(2732) + glyph(14, 2732)),
};
for (const [name, content] of Object.entries(files)) {
  const svgPath = join(RES, name);
  writeFileSync(svgPath, content);
  await sharp(Buffer.from(content)).png().toFile(svgPath.replace(/\.svg$/, '.png'));
}
console.log('rendered', Object.keys(files).length, 'sources');
const r = spawnSync(join(HERE, 'node_modules/.bin/capacitor-assets'), ['generate', '--android', '--ios',
  '--iconBackgroundColor', BG, '--iconBackgroundColorDark', BG, '--splashBackgroundColor', BG, '--splashBackgroundColorDark', BG],
  { cwd: MOBILE, stdio: 'inherit' });
process.exit(r.status ?? 1);
