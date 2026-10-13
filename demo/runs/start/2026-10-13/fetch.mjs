// Bo's Node program in the start-measure run (run.py): Node's own `fetch` at
// the lend's address, whole and streamed, and whether Node's name lookup
// (dns.lookup, the system's getaddrinfo, which fetch uses) gives its host.
// One JSON line.
//
//   node fetch.mjs <origin>
import { lookup } from 'node:dns/promises';

const [origin] = process.argv.slice(2);
const out = { origin, node: process.version };
try {
  out.resolved = (await lookup(new URL(origin).hostname, { all: true })).map((a) => a.address);
} catch (e) {
  out.resolved = null;
  out.resolve_error = `${e.code ?? e.name}: ${e.message}`;
}
const ask = (stream) => ({
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({ model: 'any', messages: [{ role: 'user', content: 'Name three rivers in Spain.' }],
    max_tokens: 32, temperature: 0, seed: 1, stream }),
});
const why = (e) => ({ reached: false, error: `${e.name}: ${e.message}`, cause: e.cause ? `${e.cause.code ?? ''} ${e.cause.message ?? e.cause}` : null });
const url = new URL('v1/chat/completions', origin);
let t0 = performance.now();
try {
  const r = await fetch(url, ask(false));
  const body = await r.json();
  out.whole = { reached: true, status: r.status, ms: Math.round(performance.now() - t0), object: body.object,
    refused: r.headers.get('x-svrngs-refused'), ...(r.ok ? {} : { said: body }) };
} catch (e) {
  out.whole = why(e);
}
t0 = performance.now();
try {
  const r = await fetch(url, ask(true));
  const text = await r.text();
  out.streamed = { reached: true, status: r.status, events: text.split('\n').filter((l) => l.startsWith('data:')).length,
    ms: Math.round(performance.now() - t0), refused: r.headers.get('x-svrngs-refused'), ...(r.ok ? {} : { said: text.slice(0, 300) }) };
} catch (e) {
  out.streamed = why(e);
}
console.log(JSON.stringify(out));
