// The desk's one tool, on the desk's computer: read a path of the support
// lend the customer gave, through the desk's own svrngs, as Pantry reaches a
// friend's tool (apps/pantry/svrngs.mjs, never svrngs's token).
//
//   SVRNGS_DIR=<desk data> node desk.mjs <path>     e.g. /incident
//   SVRNGS_DIR=<desk data> node desk.mjs <path> --method POST
//
// Prints the HTTP status, then the body. Silence is said, never defaulted.
import { svrngs } from '../../../../apps/pantry/svrngs.mjs';

const LEND = 'player-support';
const [path = '/', flag, method = 'GET'] = process.argv.slice(2);
const s = svrngs();
const { groups } = await s.ask('/v1/groups');
let origin = null;
for (const g of groups) {
  const { lent } = await s.ask(`/v1/lends?namespace=${g.id}`);
  origin ??= lent.find((l) => l.name === LEND)?.origin ?? null;
}
if (!origin) {
  console.log(`never-ran: no group this computer is in lends ${LEND} now`);
  process.exit(2);
}
try {
  const r = await s.fetch(new URL(path.replace(/^\//, ''), origin).href, { method: flag === '--method' ? method : 'GET', signal: AbortSignal.timeout(15_000) });
  console.log(`HTTP ${r.status}${r.headers.get('x-svrngs-refused') ? ` (svrngs refused: ${r.headers.get('x-svrngs-refused')})` : ''}`);
  console.log(await r.text());
} catch (e) {
  console.log(`never-ran: the customer's computer didn't answer: ${e.message}`);
  process.exit(2);
}
