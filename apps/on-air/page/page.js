// The stranger's page (ON_AIR §7): the station's key from the link's
// fragment, `#key=<hex>&show=<name>`; lines and pieces read from the public
// directory beside it, `../air/<show>/lines/<n>.json` and
// `../air/<show>/pieces/<hash>`; each line checked under the key and each
// piece against its line's hash before it plays, by Media Source. It says
// "ended" only on the station's signed end, and "cannot reach" apart from it:
// when the directory does not answer, or no next line comes within three
// pieces' time. Everything it decided is in `window.air`.
import { checkLine, checkPiece } from './air.mjs';

const WAIT_FOR_A_LINE_MS = 12_000;
const fragment = new URLSearchParams(location.hash.slice(1));
const key = fragment.get('key'), show = fragment.get('show');
const base = new URL(`../air/${encodeURIComponent(show)}/`, location.href);
const air = (window.air = { key, show, status: 'tuning in', played: [], refused: [], log: [] });
const statusEl = document.getElementById('status'), logEl = document.getElementById('log');
const audio = document.getElementById('audio');

function note(entry) {
  air.log.push({ t: Date.now(), ...entry });
  logEl.textContent = air.log.map((e) => `${e.n ?? ''} ${e.event}${e.why ? ` — ${e.why}` : ''}`).join('\n');
}
function status(s, why) {
  air.status = s; air.why = why;
  statusEl.textContent = why ? `${s[0].toUpperCase()}${s.slice(1)}: ${why}` : s[0].toUpperCase() + s.slice(1);
  note({ event: s, why });
}

// The player: Media Source fed each checked piece in order.
let buffer = null;
async function player() {
  if (!('MediaSource' in window) || !MediaSource.isTypeSupported('audio/mpeg')) {
    throw new Error('Media Source cannot play audio/mpeg in this browser');
  }
  const source = new MediaSource();
  audio.src = URL.createObjectURL(source);
  await new Promise((r) => source.addEventListener('sourceopen', r, { once: true }));
  buffer = source.addSourceBuffer('audio/mpeg');
  buffer.mode = 'sequence';
  return source;
}
const appended = (bytes) => new Promise((resolve, reject) => {
  buffer.addEventListener('updateend', resolve, { once: true });
  buffer.addEventListener('error', () => reject(new Error('the player refused the piece')), { once: true });
  buffer.appendBuffer(bytes);
});

async function fetchOr(url) {
  try { return await fetch(url, { cache: 'no-store' }); } catch (e) { return { ok: false, status: null, why: e.message }; }
}

async function run() {
  if (!/^[0-9a-f]{64}$/.test(key || '') || !show) return status('cannot tune in', "the link carries no station's key and show");
  let source;
  try { source = await player(); } catch (e) { return status('cannot play', e.message); }
  let n = 0, since = Date.now();
  for (;;) {
    const r = await fetchOr(new URL(`lines/${n}.json`, base));
    if (!r.ok) {
      if (Date.now() - since > WAIT_FOR_A_LINE_MS) {
        return status('cannot reach', r.status === 404 ? `no line ${n} in ${WAIT_FOR_A_LINE_MS / 1000} s` : `the public directory answered ${r.status ?? r.why}`);
      }
      await new Promise((res) => setTimeout(res, 500));
      continue;
    }
    since = Date.now();
    let envelope = null;
    try { envelope = await r.json(); } catch {}
    const line = checkLine(envelope, key);
    if (!line.ok) { air.refused.push({ n, why: line.why }); note({ n, event: 'refused', why: line.why }); n++; continue; }
    if (line.line.ended) {
      if (source.readyState === 'open' && !buffer.updating) source.endOfStream();
      return status('ended', `after ${line.line.n} pieces`);
    }
    const p = await fetchOr(new URL(`pieces/${line.line.hash}`, base));
    const bytes = p.ok ? new Uint8Array(await p.arrayBuffer()) : null;
    const piece = bytes ? checkPiece(bytes, line.line.hash) : { ok: false, why: `its piece did not come: ${p.status ?? p.why}` };
    if (!piece.ok) { air.refused.push({ n, why: piece.why }); note({ n, event: 'refused', why: piece.why }); n++; continue; }
    try { await appended(bytes); } catch (e) { air.refused.push({ n, why: e.message }); note({ n, event: 'refused', why: e.message }); n++; continue; }
    air.played.push({ n, hash: line.line.hash, at: Date.now() });
    note({ n, event: 'plays' });
    if (air.played.length === 1) {
      status('on air');
      audio.play().then(() => note({ event: 'playing' }), (e) => note({ event: 'not playing', why: e.message }));
    }
    n++;
  }
}
run().catch((e) => status('cannot play', e.message));
