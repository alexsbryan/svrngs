// Drive the bridge, with the fake adapter, against a running host — for the
// host's own tests (crates/svrngs/tests/removal_latency.rs).
//   node bridge/tests/harness.mjs '<steps as JSON>'
// Env: SVRNGS_HOST, SVRNGS_TOKEN_FILE, SVRNGS_OWNER, as for main.mjs. A step is
// {"do": "join"|"leave"|"ban"|"input", "speaker", "name", ...input fields}.
// Prints {"posts": [...], "replies": [...]} when every step has been handled.

import { readFileSync } from 'node:fs';

import { startBridge } from '../bridge.mjs';
import { hostClient } from '../host.mjs';
import { fakeAdapter } from './fake.mjs';

const env = process.env;
const host = hostClient({ url: env.SVRNGS_HOST, token: readFileSync(env.SVRNGS_TOKEN_FILE, 'utf8').trim() });
const chat = fakeAdapter();
const bridge = await startBridge({ adapter: chat, host, owner: env.SVRNGS_OWNER });
for (const { do: what, ...step } of JSON.parse(process.argv[2])) {
  const p = { speaker: step.speaker, name: step.name };
  if (what === 'input') await chat.input(p, step);
  else if (['join', 'leave', 'ban'].includes(what)) await chat[what](p);
  else throw new Error(`no step ${what}`);
}
await bridge.close();
process.stdout.write(`${JSON.stringify({ posts: chat.posts, replies: chat.replies })}\n`);
