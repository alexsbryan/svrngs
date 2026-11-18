// build.mjs — the svrngs mark's files, from mark.mjs. `node build.mjs`
// writes them beside this folder: the three cuts, the lockup (plain and on
// black) and the icon.
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { box, place, shapes } from './mark.mjs';

const OUT = process.env.OUT || path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const FACE = `system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif`;
// Every file is this: a viewBox, a name a screen reader says, the drawing.
const write = (name, view, label, body) => {
  fs.writeFileSync(
    path.join(OUT, name),
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${view}" role="img" aria-label="${label}">\n<title>${label}</title>\n${body}\n</svg>\n`,
  );
  console.log('wrote', name);
};

for (const cut of ['large', 'medium', 'small']) {
  const b = box(cut);
  write(`mark-${cut}.svg`, `${b.x} ${b.y} ${b.w} ${b.h}`, 'svrngs', shapes(cut));
}

// The lockup: the name as the pages' h1 sets it, black (900) and tracked
// tight, with what it is short for under it, since "svrngs" alone reads as
// "savings".
const words = (fill) =>
  `<text x="192" y="104" font-family="${FACE}" font-size="104" font-weight="900" letter-spacing="-4.7" fill="${fill}">svrngs</text>` +
  `<text x="196" y="150" font-family="${FACE}" font-size="33" fill="${fill}">sovereign rings</text>`;
write(
  'lockup.svg',
  '0 0 560 180',
  'svrngs, sovereign rings',
  place(10, 10, 160, 'large') + words('currentColor'),
);

// The lockup on black, white as the app icon is, so a README shows it the
// same on a light page and a dark one.
write(
  'lockup-on-black.svg',
  '0 0 620 220',
  'svrngs, sovereign rings',
  `<rect width="620" height="220" rx="24" fill="black"/><g transform="translate(30 20)">` +
    place(10, 10, 160, 'large', { ink: 'white' }) +
    words('white') +
    '</g>',
);

// The favicon: the small cut, its ink white when the browser is dark, or it
// vanishes into a dark tab bar. A page links it as /favicon.svg.
write(
  'favicon.svg',
  Object.values(box('small')).join(' '),
  'svrngs',
  `<style>svg{color:black}@media (prefers-color-scheme:dark){svg{color:white}}</style>${shapes('small')}`,
);

// The app icon: white on black, where the gold reads best. A platform rounds
// the corners itself.
write(
  'icon.svg',
  '0 0 1024 1024',
  'svrngs',
  `<rect width="1024" height="1024" fill="black"/>${place(113, 113, 798, 'large', { ink: 'white' })}`,
);

// The same icon where the Arch package takes it from (packaging/arch): the
// app menu's svrngs entry shows it.
const ARCH = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '../../../packaging/arch/svrngs.svg',
);
fs.copyFileSync(path.join(OUT, 'icon.svg'), ARCH);
console.log('wrote', path.relative(process.cwd(), ARCH));
