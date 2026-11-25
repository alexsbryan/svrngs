// The agent's one tool: GET a route of the site's view, through the vendor
// computer's svrngs, from the lend named by the arm. Prints JSON.
//   SVRNGS_DIR=<vendor> node view.mjs <lend> <route>
import { svrngs } from '../../../../apps/pantry/svrngs.mjs';
const [lend, route = '/'] = process.argv.slice(2);
const s = svrngs();
let origin = null;
for (const g of (await s.ask('/v1/groups')).groups) origin ??= (await s.ask(`/v1/lends?namespace=${g.id}`)).lent.find((l) => l.name === lend)?.origin ?? null;
if (!origin) { console.log(JSON.stringify({ verdict: 'never-ran', why: `no group lends ${lend} now` })); process.exit(2); }
try {
  const r = await s.fetch(new URL(route.replace(/^\//, ''), origin).href, { signal: AbortSignal.timeout(30_000) });
  const t = await r.text();
  console.log(r.ok ? t : JSON.stringify({ verdict: 'never-ran', status: r.status, why: t.slice(0, 300) }));
} catch (e) { console.log(JSON.stringify({ verdict: 'never-ran', why: `the site didn't answer: ${e.message}` })); process.exit(2); }
