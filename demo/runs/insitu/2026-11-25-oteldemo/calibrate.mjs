// Which baseline operation names (read from the raw arm, no fault on) the
// profile would shape. Run before the first fault; the agents never see it.
import { readFileSync } from 'node:fs';
import { codeName } from './project.mjs';
const R = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const names = new Set();
for (const v of Object.values(R)) for (const n of Object.keys(v.operations)) names.add(n);
const shaped = [...names].filter((n) => !codeName(n));
console.log(`operations: ${names.size}, shaped: ${shaped.length}`);
for (const n of shaped.sort()) console.log('  ' + JSON.stringify(n));
