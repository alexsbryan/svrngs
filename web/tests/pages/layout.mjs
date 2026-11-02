// The sites' layout, for node: the pages' modules import each other as they
// are served, `/ui/`, `/vendor/` and `/page/` side by side on each page
// (scripts/build-web.sh; home.rs `served`), while in the source tree `ui/`
// is crates/svrngs-node/src/home/ui/ and `vendor/` is web/vendor/.
// A test registers this (`register('./pages/layout.mjs', import.meta.url)`)
// before it imports a page's module, and each relative import resolves where
// the file is kept. The bytes are the ones served, `?v=__VERSION__` kept.

const UI = '/crates/svrngs-node/src/home/ui/';
const KEPT = [
  ['/crates/svrngs-node/src/home/vendor/', '/web/vendor/'],
  ['/web/ui/', UI],
  // question.js is served beside app.js on the phone, kept with the home page.
  ['/web/question.js', '/crates/svrngs-node/src/home/question.js'],
];

export async function resolve(spec, ctx, next) {
  if (!spec.startsWith('.') || !ctx.parentURL?.startsWith('file:')) return next(spec, ctx);
  const url = new URL(spec, ctx.parentURL);
  for (const [served, kept] of KEPT) url.pathname = url.pathname.replace(served, kept);
  return next(url.href, ctx);
}
