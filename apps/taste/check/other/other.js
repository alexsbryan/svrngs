// The other app (index.html): on load it asks the taste its page reads to
// rank two films; `#unasked` says a yes for itself with no word to its
// person, then asks again — su-0's measured gap (§3), which svrngs refuses
// since sup-2 and sup-3 (DESIGN §3, "What an app gets"). On a computer it
// posts the yes to the home page, the origin of the `ask` address it is
// told; on a phone it dials the taste's door itself as its pass's holder,
// with what this origin keeps (the pass, the holder's key) and the page's
// own runtime, as any app's page could. What came back is #out's text, its
// state in #out's data-state, the yes's in data-yes.
const out = document.getElementById('out');
const FILMS = [{ title: 'Alien', year: 1979, ids: {} }, { title: 'Arrival', year: 2016, ids: {} }];
const told = await fetch('/svrngs-reads').then((r) => (r.ok ? r.json() : null), () => null);
const t = told?.lends.find((l) => l.kind === 'taste');

async function rank() {
  if (!t) return { status: null, body: 'no taste is read here' };
  try {
    const res = await fetch(`${t.address}rank`, { method: 'POST', body: JSON.stringify({ films: FILMS }) });
    return { status: res.status, refused: res.headers.get('x-svrngs-refused'), body: await res.text() };
  } catch (e) {
    return { status: null, body: `unread: ${e}` };
  }
}

const kept = (own) => new Promise((done) => {
  const req = indexedDB.open('svrngs-lend');
  req.onerror = () => done(null);
  req.onsuccess = () => {
    const get = req.result.transaction('kept').objectStore('kept').get(`pass ${own}`);
    get.onsuccess = () => done(get.result ?? null);
    get.onerror = () => done(null);
  };
});

// Each stage of the yes reached, kept on #out, so one that hangs is named.
// The runtime's trace, to this page's console.
globalThis.__trace = (l) => console.debug(l);
const reached = (what) => (out.dataset.reached = `${out.dataset.reached ?? ''} ${what}`.trim());

async function yes() {
  if (!t?.ask) return { how: 'none', body: 'no taste that asks is read here' };
  const ask = new URL(t.ask);
  const [lend, app] = [ask.searchParams.get('lend'), ask.searchParams.get('app')];
  try {
    if (ask.hostname.startsWith('svrngs.')) {
      // The home page's own route, from this page: an answer no other
      // origin may read, so what the node did is in its log.
      const res = await fetch(`${ask.origin}/choices`, { method: 'POST', mode: 'no-cors', body: JSON.stringify({ lend, app, said: 'yes' }) });
      return { how: 'the home page', status: res.type === 'opaque' ? 'opaque' : res.status };
    }
    const k = await kept(location.hostname.split('.')[0]);
    reached('kept');
    // The page's runtime, which the lend's worker leaves to the network
    // when asked with a version (web/lend.js `SHELL`).
    const { default: init, holder_endpoint } = await import('/wasm/svrngs_web.js?v=other');
    await init({ module_or_path: '/wasm/svrngs_web_bg.wasm?v=other' });
    reached('runtime');
    const holder = Uint8Array.from(k.holder.match(/../g), (h) => parseInt(h, 16));
    // The lend's worker holds an endpoint of the same key on the relay;
    // one that does not come online beside it within 30 s is said so.
    const late = new Promise((_, no) => setTimeout(() => no(new Error('the holder\'s endpoint did not come online within 30 s')), 30_000));
    const endpoint = await Promise.race([holder_endpoint(holder, k.relay), late]);
    reached('online');
    const door = await Promise.race([endpoint.dial_lend(k.lends.find((l) => l.label === lend).key), late]);
    reached('dialled');
    const body = new TextEncoder().encode(JSON.stringify({ app, said: 'yes' }));
    const headers = [['x-svrngs-pass', k.pass], ['content-type', 'application/json'], ['content-length', String(body.length)]];
    const res = await door.fetch('POST', `/choices/${lend}`, headers, new Blob([body]).stream());
    return { how: 'the door, as the holder', status: res.status, refused: res.headers.get('x-svrngs-refused'), body: await res.text() };
  } catch (e) {
    return { how: 'failed', body: String(e) };
  }
}

function show(what, got) {
  out.dataset.state = `${what} ${got.status}`;
  out.dataset.refused = got.refused ?? '';
  out.textContent = got.body;
}

document.getElementById('unasked').onclick = async () => {
  const said = await yes();
  out.dataset.yes = JSON.stringify(said);
  show('unasked', await rank());
};
show('rank', await rank());
