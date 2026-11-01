// The sites' layout, for node: v2's modules import each other as they are
// served, `/ui/`, `/vendor/` and `/v2/` side by side on each page
// (scripts/build-web.sh --ui v2; home/v2/serve.rs), while in the source
// tree `ui/` is crates/svrngs-node/src/home/ui/ and `vendor/` is web/vendor/.
// A test registers this (`register('./v2/layout.mjs', import.meta.url)`)
// before it imports a v2 module, and each relative import resolves where
// the file is kept. The bytes are the ones served, `?v=__VERSION__` kept.

const UI = '/crates/svrngs-node/src/home/ui/';
const KEPT = [
  ['/crates/svrngs-node/src/home/vendor/', '/web/vendor/'],
  ['/web/ui/', UI],
  // The phone's frame, web/v2/app.js, is served as /app.js, beside ui/ and v2/.
  ['/web/v2/ui/', UI],
  ['/web/v2/v2/', '/web/v2/'],
];

export async function resolve(spec, ctx, next) {
  if (!spec.startsWith('.') || !ctx.parentURL?.startsWith('file:')) return next(spec, ctx);
  const url = new URL(spec, ctx.parentURL);
  for (const [served, kept] of KEPT) url.pathname = url.pathname.replace(served, kept);
  return next(url.href, ctx);
}
