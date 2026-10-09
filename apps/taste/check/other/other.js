// The other app (index.html): on load it asks the first taste its pass
// names to rank two films; `#unasked` sends a yes as the person with no
// word to them, then asks again — su-0's measured gap (§3). What came
// back is #out's text, and its state in #out's data-state.
const OWN = location.hostname.split('.')[0];
const baseOf = (label) => `${location.protocol}//${label}${location.host.slice(OWN.length)}/`;
const out = document.getElementById('out');
const FILMS = [{ title: 'Alien', year: 1979, ids: {} }, { title: 'Arrival', year: 2016, ids: {} }];

const kept = () => new Promise((done) => {
  const req = indexedDB.open('svrngs-lend');
  req.onerror = () => done(null);
  req.onsuccess = () => {
    const get = req.result.transaction('kept').objectStore('kept').get(`pass ${OWN}`);
    get.onsuccess = () => done(get.result ?? null);
    get.onerror = () => done(null);
  };
});

async function post(path, body) {
  const t = ((await kept())?.lends ?? []).find((l) => l.name === 'taste');
  if (!t) return { status: null, body: 'no taste in the pass' };
  const res = await fetch(`${baseOf(t.label)}${path}`, { method: 'POST', body: JSON.stringify(body) }).catch((e) => ({ status: null, text: async () => String(e) }));
  return { status: res.status, body: await res.text() };
}

function show(what, got) {
  out.dataset.state = `${what} ${got.status}`;
  out.textContent = got.body;
}

document.getElementById('unasked').onclick = async () => {
  const yes = await post('yes', {});
  out.dataset.yes = String(yes.status);
  show('unasked', await post('rank', { films: FILMS }));
};
show('rank', await post('rank', { films: FILMS }));
