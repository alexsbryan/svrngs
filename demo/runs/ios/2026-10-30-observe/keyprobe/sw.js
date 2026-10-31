self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));
const sign = async (k) => { try { await crypto.subtle.sign(k.algorithm.name === 'HMAC' ? 'HMAC' : 'Ed25519', k, new Uint8Array(4)); return 'signs'; } catch (e) { return 'sign failed ' + e; } };
self.addEventListener('message', (e) => e.waitUntil((async () => {
  const d = e.data ?? {};
  let said = `keys: ${Object.keys(d)}`;
  if (d.key) said += `, a ${d.key.constructor.name} ${d.key.algorithm?.name}, ${await sign(d.key)}`;
  if (d.fromDb) {
    said = await new Promise((ok) => { const r = indexedDB.open('probe', 1); r.onupgradeneeded = () => r.result.createObjectStore('k');
      r.onsuccess = () => { const g = r.result.transaction('k').objectStore('k').get('nx'); g.onsuccess = async () => ok(g.result ? `read a ${g.result.constructor.name}, ${await sign(g.result)}` : 'read nothing'); g.onerror = () => ok('read failed ' + g.error); };
      r.onerror = () => ok('open failed ' + r.error); });
  }
  e.ports[0]?.postMessage(said);
})()));
self.addEventListener('messageerror', async () => { for (const c of await self.clients.matchAll()) c.postMessage({ messageerror: true }); });
