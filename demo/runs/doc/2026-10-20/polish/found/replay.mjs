// node replay.mjs first-take-acts.jsonl, with DOC=<apps/doc>: the first take's acts (../../data/ is its pages' view), one line per act that changed the paragraphs.
// Replay the take's acts one by one: who wrote each, what the fragment's paragraphs are after it, and what it deleted.
import { readFileSync } from 'node:fs';
const { Y } = await import(process.env.DOC + '/vendor/doc-bundle.js');
const { b64ToBytes } = await import(process.env.DOC + '/adapter.js');
const acts = readFileSync(process.argv[2], 'utf8').split('\n').filter(Boolean).map(JSON.parse);
const doc = new Y.Doc(); const frag = doc.getXmlFragment('default');
let before = JSON.stringify([]);
for (const a of acts) {
  const u = b64ToBytes(a.update);
  const d = Y.decodeUpdate(u);
  Y.applyUpdate(doc, u);
  const now = JSON.stringify(frag.toArray().map((p) => p.toString().replace(/<\/?paragraph>/g, '')));
  const dels = [...d.ds.clients.entries()].map(([c, ds]) => `${c}:${ds.map((x) => `${x.clock}+${x.len}`).join(',')}`).join(' ');
  if (now !== before || dels) console.log(String(a.seq).padStart(3), a.person.padEnd(3), (dels ? `del[${dels}]` : '').padEnd(28), now);
  before = now;
}
