// Whether a non-extractable Ed25519 key can be made, kept in IndexedDB,
// read back, handed to a worker (structured clone), sign there, and never
// export its bytes — in each engine Playwright runs here.
import { launch } from '../../../../web/tests/page.mjs';
import http from 'node:http';
const page_html = `<!doctype html><script>
window.probe = async () => {
  const out = {};
  try {
    const k = await crypto.subtle.generateKey({ name: 'Ed25519' }, false, ['sign', 'verify']);
    out.made = true;
    const db = await new Promise((d, f) => { const r = indexedDB.open('p', 1); r.onupgradeneeded = () => r.result.createObjectStore('kept'); r.onsuccess = () => d(r.result); r.onerror = () => f(r.error); });
    await new Promise((d, f) => { const t = db.transaction('kept', 'readwrite'); t.objectStore('kept').put(k.privateKey, 'holder'); t.oncomplete = d; t.onerror = () => f(t.error); });
    const back = await new Promise((d) => { const g = db.transaction('kept').objectStore('kept').get('holder'); g.onsuccess = () => d(g.result); });
    out.keptAndRead = back instanceof CryptoKey;
    out.signsAfterRead = (await crypto.subtle.sign('Ed25519', back, new Uint8Array([1,2,3]))).byteLength;
    out.exportRaw = await crypto.subtle.exportKey('pkcs8', back).then(() => 'EXPORTED', (e) => 'refused: ' + e.name);
    out.exportJwk = await crypto.subtle.exportKey('jwk', back).then((j) => 'EXPORTED ' + (j.d ? 'with d' : ''), (e) => 'refused: ' + e.name);
    out.pub = (await crypto.subtle.exportKey('raw', k.publicKey)).byteLength;
    const w = new Worker(URL.createObjectURL(new Blob([\`onmessage = async (e) => postMessage((await crypto.subtle.sign('Ed25519', e.data, new Uint8Array([9]))).byteLength)\`])));
    out.signsInWorker = await new Promise((d) => { w.onmessage = (e) => d(e.data); w.onerror = (e) => d('error ' + e.message); w.postMessage(back); });
  } catch (e) { out.error = String(e); }
  return out;
};
</script>`;
const server = http.createServer((q, r) => { r.setHeader('content-type', 'text/html'); r.end(page_html); }).listen(0);
await new Promise((d) => server.on('listening', d));
for (const engine of ['chromium', 'webkit']) {
  const { context, close } = await launch(engine);
  const p = await context.newPage();
  await p.goto(`http://localhost:${server.address().port}/`);
  console.log(engine, context.browser()?.version(), JSON.stringify(await p.evaluate(() => window.probe())));
  await close();
}
server.close();
