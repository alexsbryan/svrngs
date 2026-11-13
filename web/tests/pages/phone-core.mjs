// The phone's runtime, web/wasm/svrngs_web.js, stood in for in Node, a
// test-side file: a law registers this (`register('./phone-core.mjs',
// import.meta.url)`, after layout.mjs) and sets `globalThis.__core` to an
// object answering as web/src/lib.rs's exports answer, and every page
// module's import of the runtime (dial.js `wasm`, app.js) reaches it here.
// The runtime is built by scripts/build-web.sh and is not in the source
// tree, so without this the import has nothing to load.
//
// As a loader hook (`resolve`, run on the loader's thread) it sends the
// runtime's specifier here; as that module (on the law's thread) each
// export the page's flows call (`grep -oh 'core\.[a-z_]*' web/page/*.js`)
// asks `__core` at the call, so each law answers with its own.

const RUNTIME = /\/wasm\/svrngs_web\.js(\?|$)/;

export async function resolve(spec, ctx, next) {
  if (!RUNTIME.test(spec)) return next(spec, ctx);
  return { url: new URL('./phone-core.mjs', import.meta.url).href, shortCircuit: true };
}

// `name` as the law's core answers it, refused by name when it has none.
const asked =
  (name) =>
  (...args) => {
    const core = globalThis.__core;
    if (typeof core?.[name] !== 'function') throw new Error(`this law's core has no ${name}`);
    return core[name](...args);
  };

export default asked('init');
export const trace_iroh = asked('trace_iroh');
export const actor = asked('actor');
export const ask = asked('ask');
export const believe = asked('believe');
export const carried_label = asked('carried_label');
export const connect_to_join = asked('connect_to_join');
export const consent = asked('consent');
export const current_version = asked('current_version');
export const endpoint = asked('endpoint');
export const is_label = asked('is_label');
export const issuer_of = asked('issuer_of');
export const label = asked('label');
export const links = asked('links');
export const older = asked('older');
export const pass = asked('pass');
export const redemption = asked('redemption');
export const relay_of = asked('relay_of');
export const relay_taken = asked('relay_taken');
export const relays_of = asked('relays_of');
export const sign_line = asked('sign_line');
