// The harness's one way to ask a computer's svrngs (run.py `http`): the
// request's body on stdin, svrngs's answer on stdout as one JSON line, its
// bytes in base64, or why it could not be asked. Through
// apps/client/svrngs.mjs, so neither the harness nor this reads the token.
//
//   node ask.mjs <svrngs data dir> <method> <route>

import { svrngs } from '../../client/svrngs.mjs';

const [dir, method, route] = process.argv.slice(2);
const chunks = [];
if (!['GET', 'HEAD'].includes(method)) for await (const chunk of process.stdin) chunks.push(chunk);
const body = chunks.length ? Buffer.concat(chunks) : undefined;
try {
  const type = { 'content-type': 'application/octet-stream' };
  const r = await svrngs(dir).fetch(route, { method, body, headers: type });
  const bytes = Buffer.from(await r.arrayBuffer());
  console.log(JSON.stringify({ status: r.status, body: bytes.toString('base64') }));
} catch (e) {
  console.log(JSON.stringify({ status: null, said: `could not ask: ${e.message}` }));
}
