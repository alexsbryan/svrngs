// The leak check: harvest the personal values the services emitted (from
// the raw arm, the same window), then read every route of the projected
// arm and fail on any of them. With --plant it searches the raw arm itself,
// which must fail. Prints passed/failed and counts; never the values.
//   node leakcheck.mjs [--plant]
import { execFileSync } from 'node:child_process';
const view = (arm, route) => execFileSync(`./view-${arm}.sh`, [route], { maxBuffer: 512 << 20 }).toString();
const ROUTES = ['/services?since=1800', '/errors?since=1800&limit=500', '/spans?since=1800&limit=500', '/logs?since=1800&limit=500', '/metrics?since=1800&limit=500', '/spans?since=1800&limit=500&status=error'];
// Keys about a person, never about a machine (server.address names a service).
const SENSITIVE = /(^|\.)(email|card_number|card_cvv|cvv|credit_card|street_address|zip_code|user_id|user\.id|client\.address)$/i;
const markers = new Set();
const walk = (o, k) => {
  if (o && typeof o === 'object') { for (const [kk, v] of Object.entries(o)) walk(v, kk); return; }
  if (typeof o === 'string' && SENSITIVE.test(k ?? '') && o.length >= 4) markers.add(o);
  if (typeof o === 'string' && /[\w.+-]+@[\w-]+\.[\w.]+/.test(o)) for (const m of o.match(/[\w.+-]+@[\w-]+\.[\w.]+/g)) markers.add(m);
  if (typeof o === 'string' && /\b\d{13,19}\b/.test(o)) for (const m of o.match(/\b\d{13,19}\b/g)) markers.add(m);
};
const machines = new Set();
const machine = (o, k) => { if (o && typeof o === 'object') { for (const [kk, v] of Object.entries(o)) machine(v, kk); return; } if (typeof o === 'string' && /^(server\.address|net\.peer\.name|net\.host\.name|host\.name)$/.test(k ?? '')) machines.add(o); };
for (const r of ROUTES) { const j = JSON.parse(view('R', r)); walk(j); machine(j); }
// A value the raw arm also shows as a machine's own address is that
// machine's, not a person's: a service's container IP is a client.address
// in one span and a server.address in the next (rerun F4, 2026-11-25:
// 10.89.1.22, flagged in P where it passed as server.address).
for (const m of machines) markers.delete(m);
const arm = process.argv.includes('--plant') ? 'R' : 'P';
const text = ROUTES.map((r) => view(arm, r)).join('\n');
const hits = [...markers].filter((m) => text.includes(m));
// --debug keeps each hit with where it sits in the searched text, in a
// git-ignored file, for the operator to judge; the console never shows it.
if (process.argv.includes('--debug') && hits.length) {
  const { appendFileSync } = await import('node:fs');
  for (const m of hits) { const i = text.indexOf(m); appendFileSync('out/leak-debug.txt', `${new Date().toISOString()} ${arm} value=${JSON.stringify(m)} context=${JSON.stringify(text.slice(Math.max(0, i - 200), i + m.length + 40))}\n`); }
}
const kinds = (xs) => ({ emails: xs.filter((m) => m.includes('@')).length, card_like: xs.filter((m) => /^\d{13,19}$/.test(m)).length, other: xs.filter((m) => !m.includes('@') && !/^\d{13,19}$/.test(m)).length });
console.log(JSON.stringify({ arm, markers: markers.size, marker_kinds: kinds([...markers]), bytes_read: text.length, leaked: hits.length, leaked_kinds: kinds(hits) }));
console.log(markers.size === 0 ? 'could-not-judge  no personal values were emitted to look for' : hits.length ? `failed     ${hits.length} of ${markers.size} personal values in the ${arm} arm` : `passed     none of ${markers.size} personal values in the ${arm} arm`);
process.exitCode = markers.size && !hits.length ? 0 : 1;
