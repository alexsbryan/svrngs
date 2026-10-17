// The listener (ON_AIR §7), on a member's computer: drains the live lane,
// gets each piece by its hash from svrngs, checks it, and appends it to a
// file a person can play. Each payload is kept with who the drain says sent
// it, or that it names no one; a line is the station's when its signature
// checks under the station's key, or when the drain names the station's key
// as its sender. Anything else is refused, saying why.
//
//   node listen.mjs --api <url> --group <id> --station <person> --out <dir>
//        [--for <seconds>]
//
// It stops on the station's `ended`, or after --for seconds (default 300),
// saying which.
import './vendored.mjs';
import { mkdir, appendFile } from 'node:fs/promises';
import { join } from 'node:path';
import { parseArgs } from 'node:util';

const air = await import('./page/air.mjs');
const { values: o } = parseArgs({
  options: {
    api: { type: 'string' }, group: { type: 'string' }, station: { type: 'string' }, out: { type: 'string' },
    for: { type: 'string', default: '300' },
  },
});
for (const k of ['api', 'group', 'station', 'out']) if (!o[k]) throw new Error(`--${k} is needed`);
const api = o.api.replace(/\/$/, '');
await mkdir(o.out, { recursive: true });

async function say(event, fields) {
  const line = JSON.stringify({ t: Date.now(), who: 'listen', event, ...fields });
  console.log(line);
  await appendFile(join(o.out, 'listen.jsonl'), line + '\n');
}

async function get(path) {
  try {
    const r = await fetch(api + path);
    return { status: r.status, bytes: new Uint8Array(await r.arrayBuffer()) };
  } catch (e) {
    return { status: null, bytes: new Uint8Array(0), said: `could not ask: ${e.cause?.code || e.message}` };
  }
}
const text = (b) => new TextDecoder().decode(b).slice(0, 300);

// The station's keys, from the group's people: its computer's, which signs
// for the app (CHARTER.md), and its person's first.
const people = await get(`/v1/rail/people?namespace=${o.group}`);
const rows = people.status === 200 ? JSON.parse(new TextDecoder().decode(people.bytes)).people : [];
const station = rows.filter((r) => r.person === o.station);
if (station.length !== 1) {
  await say('no station', { why: `the group names ${station.length} computers of ${o.station}`, people: people.status });
  process.exit(2);
}
const { key, person_key } = station[0];
await say('listening', { station: o.station, key, person_key });

// A payload as the drain gives it: `{payload, key, person_key}` since
// onair-live, a bare string before it, kept as the drain named it.
function unwrap(p) {
  if (typeof p === 'string') return { text: p, sender: null };
  return { text: p?.payload ?? null, sender: { key: p?.key ?? null, person_key: p?.person_key ?? null } };
}

const until = Date.now() + Number(o.for) * 1000;
let ended = null;
while (!ended && Date.now() < until) {
  const drained = await get(`/v1/rail/live?namespace=${o.group}`);
  if (drained.status !== 200) {
    await say('drain refused', { status: drained.status, said: drained.said ?? text(drained.bytes) });
    await new Promise((r) => setTimeout(r, 1000));
    continue;
  }
  const { payloads = [], dropped = 0 } = JSON.parse(new TextDecoder().decode(drained.bytes));
  if (dropped) await say('dropped', { dropped });
  for (const p of payloads) {
    const arrived = Date.now();
    const { text: body, sender } = unwrap(p);
    let envelope = null;
    try { envelope = JSON.parse(body); } catch {}
    const signed = air.checkLine(envelope, key);
    const named = sender?.key === key;
    const said = { sender: sender ?? 'the drain names no sender', payload: String(body).slice(0, 200) };
    if (!signed.ok && !named) {
      await say('refused', { ...said, why: `${signed.why}; ${sender ? `sent by ${String(sender.key).slice(0, 8)}…, not the station` : 'and the drain names no sender'}` });
      continue;
    }
    const line = signed.ok ? signed.line : (() => { try { return JSON.parse(envelope.line); } catch { return null; } })();
    if (!line) { await say('refused', { ...said, why: 'from the station, but not a line' }); continue; }
    const by = signed.ok ? 'its signature' : 'the drain naming the station';
    if (line.ended) {
      ended = { n: line.n, at: line.at, by };
      await say('ended', { ...said, ...ended });
      break;
    }
    const got = await get(`/v1/blob/${line.hash}?group=${o.group}`);
    if (got.status !== 200) {
      await say('not held', { ...said, n: line.n, hash: line.hash, by, status: got.status, said: got.said ?? text(got.bytes) });
      continue;
    }
    const checked = air.checkPiece(got.bytes, line.hash);
    if (!checked.ok) { await say('refused piece', { ...said, n: line.n, hash: line.hash, by, why: checked.why }); continue; }
    await appendFile(join(o.out, 'heard.mp3'), got.bytes);
    await say('held', { ...said, n: line.n, hash: line.hash, by, bytes: got.bytes.length, ms: Date.now() - arrived, line_at: line.at });
  }
  await new Promise((r) => setTimeout(r, 200));
}
if (!ended) await say('stopped', { why: `no ended from the station in ${o.for} s` });
