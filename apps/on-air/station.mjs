// The station (ON_AIR §7): an MP3 cut into pieces on frame boundaries; each
// piece put by hash with a lifetime, its line signed for the app and pushed
// on the live lane, and piece and line copied into the public directory; a
// signed `ended` last. Every answer svrngs gives is kept as it came, and a
// step it refused is said, never filled in:
//
//   node station.mjs --api <url> --group <id> --mp3 <file> --show <name>
//        --public <dir> [--keep 60] [--piece 4] [--pace 1] [--limit n]
//        [--no-ended] [--out <file.jsonl>]
//
// --pace 0 sends as fast as it can (a plant's show); --no-ended stops
// without the signed end (the off-air plant).
import './vendored.mjs';
import { mkdir, readFile, writeFile, appendFile } from 'node:fs/promises';
import { join } from 'node:path';
import { parseArgs } from 'node:util';

const air = await import('./page/air.mjs');
const { values: o } = parseArgs({
  options: {
    api: { type: 'string' }, group: { type: 'string' }, mp3: { type: 'string' }, show: { type: 'string' },
    public: { type: 'string' }, keep: { type: 'string', default: '60' }, piece: { type: 'string', default: '4' },
    pace: { type: 'string', default: '1' }, limit: { type: 'string' }, 'no-ended': { type: 'boolean' },
    out: { type: 'string' },
  },
});
for (const k of ['api', 'group', 'mp3', 'show', 'public']) if (!o[k]) throw new Error(`--${k} is needed`);
const api = o.api.replace(/\/$/, '');
const dir = join(o.public, 'air', o.show);
await mkdir(join(dir, 'lines'), { recursive: true });
await mkdir(join(dir, 'pieces'), { recursive: true });

async function say(event, fields) {
  const line = JSON.stringify({ t: Date.now(), who: 'station', show: o.show, event, ...fields });
  console.log(line);
  if (o.out) await appendFile(o.out, line + '\n');
}

// One request to this computer's svrngs: its status and words as they came,
// or why it could not be asked.
async function ask(method, path, body, type = 'application/octet-stream') {
  const t0 = Date.now();
  try {
    const r = await fetch(api + path, { method, body, headers: { 'content-type': type } });
    const text = await r.text();
    let json = null;
    try { json = JSON.parse(text); } catch {}
    return { status: r.status, json, said: text.slice(0, 300), ms: Date.now() - t0 };
  } catch (e) {
    return { status: null, said: `could not ask: ${e.cause?.code || e.message}`, ms: Date.now() - t0 };
  }
}

// Sign a line for the app; an envelope that says why when svrngs did not.
async function sign(text) {
  const r = await ask('POST', `/v1/sign?app=${air.APP}`, text, 'text/plain; charset=utf-8');
  const key = r.json?.key, sig = r.json?.sig;
  if (r.status === 200 && typeof key === 'string' && typeof sig === 'string') return { envelope: { line: text, key, sig }, r };
  return { envelope: { line: text, key: null, sig: null, unsigned: `POST /v1/sign answered ${r.status ?? 'nothing'}${r.said ? `: ${r.said}` : ''}` }, r };
}

async function onAir(n, text, extra) {
  const { envelope, r: signed } = await sign(text);
  const payload = JSON.stringify(envelope);
  const live = await ask('POST', `/v1/rail/live?namespace=${o.group}`, payload, 'text/plain; charset=utf-8');
  await writeFile(join(dir, 'lines', `${n}.json`), payload);
  await say(extra.ended ? 'ended' : 'piece', {
    n, ...extra, line: text,
    sign: { status: signed.status, said: signed.said, ms: signed.ms, signed: !!envelope.sig },
    live: { status: live.status, delivered: live.json?.delivered, peers: live.json?.peers?.map((p) => p.name ?? p.node), said: live.json ? undefined : live.said, ms: live.ms },
  });
}

const mp3 = new Uint8Array(await readFile(o.mp3));
let pieces = air.cut(mp3, Number(o.piece));
await say('cut', { bytes: mp3.length, pieces: pieces.length, seconds: pieces.map((p) => +p.seconds.toFixed(3)) });
if (o.limit) pieces = pieces.slice(0, Number(o.limit));
const pace = Number(o.pace), t0 = Date.now();
let due = 0;
for (const [n, p] of pieces.entries()) {
  // Each piece goes out when the last has had its time on air.
  const wait = t0 + due * 1000 * pace - Date.now();
  if (wait > 0) await new Promise((r) => setTimeout(r, wait));
  due += p.seconds;
  const hash = air.blake3hex(p.bytes);
  const put = await ask('PUT', `/v1/blob?group=${o.group}&keep=${o.keep}`, p.bytes);
  // The hash is the bytes' BLAKE3 either way; svrngs's answer is checked
  // against it, and its absence said.
  const held = put.status === 200 && put.json?.hash === hash ? 'held'
    : put.status === 200 ? `svrngs named ${put.json?.hash}, not the bytes' ${hash}` : `not held: PUT /v1/blob answered ${put.status ?? 'nothing'}`;
  await writeFile(join(dir, 'pieces', hash), p.bytes);
  await onAir(n, air.pieceLine(o.show, n, hash, Date.now()), {
    hash, bytes: p.bytes.length, put: { status: put.status, said: put.said, ms: put.ms, held },
  });
}
if (o['no-ended']) await say('no ended', { n: pieces.length, why: '--no-ended: the off-air plant' });
else {
  const wait = t0 + due * 1000 * pace - Date.now();
  if (wait > 0) await new Promise((r) => setTimeout(r, wait));
  await onAir(pieces.length, air.endedLine(o.show, pieces.length, Date.now()), { ended: true });
}
