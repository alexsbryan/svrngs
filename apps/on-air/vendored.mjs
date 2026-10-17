// Node finds `@noble/curves/...` and `@noble/hashes/...` in page/vendor/, as
// the page's import map finds them, so the station, the listener and the
// page check with the one copy (vendor/README.md). Import this first, then
// what imports them dynamically: a static import is resolved before it runs.
import { register } from 'node:module';

const vendor = new URL('./page/vendor/', import.meta.url).href;
const hook = `export async function resolve(s, c, next) {
  const m = /^@noble\\/(curves|hashes)\\/(.+)$/.exec(s);
  return next(m ? ${JSON.stringify(vendor)} + 'noble-' + m[1] + '/' + m[2] : s, c);
}`;
register(`data:text/javascript,${encodeURIComponent(hook)}`);
