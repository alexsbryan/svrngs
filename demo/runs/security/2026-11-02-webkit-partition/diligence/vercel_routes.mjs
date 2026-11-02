// What routes does the Vercel CLI 59.10.0's own @vercel/routing-utils turn
// web/vercel.json's `headers` into, and what does the dev router's merge
// (dist/commands/dev/index.js:17277-17320: combinedHeaders[lowerKey] = value,
// in route order, for every matching route) give for /sign.html?
// usage: node vercel_routes.mjs /Users/alexsbryan/dev/svrngs/web/vercel.json
import fs from 'node:fs';
const chunk = await import('/Users/alexsbryan/.nvm/versions/node/v20.20.2/lib/node_modules/vercel/dist/chunks/chunk-Z44PATWW.js');
let ru;
for (const [k, f] of Object.entries(chunk)) {
  if (!/^require_dist/.test(k)) continue;
  try { const m = f(); if (m && m.getTransformedRoutes) { ru = m; break; } } catch {}
}
const cfg = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const { routes, error } = ru.getTransformedRoutes(cfg);
if (error) throw error;
console.log(JSON.stringify(routes, null, 1));
for (const p of ['/sign.html', '/', '/index.html']) {
  const combined = {};
  for (const r of routes) {
    if (ru.isHandler(r) || !r.headers) continue;
    if (!new RegExp(r.src, 'i').test(p)) continue;
    for (const k of Object.keys(r.headers)) combined[k.toLowerCase()] = r.headers[k];
  }
  console.log(p, '->', JSON.stringify(combined));
}
