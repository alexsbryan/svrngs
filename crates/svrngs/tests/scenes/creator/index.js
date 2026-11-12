// Scene fixture adapted from the guide's guestbook. Identity comes from svrngs.
const $ = (id) => document.getElementById(id);
const say = (words) => {
  $('said').textContent = words;
};

async function show() {
  const r = await fetch('svrngs/acts');
  const read = await r.json();
  if (!r.ok) return say(`Cannot read the rota: ${read.error ?? r.status}`);
  $('you').textContent = `You are ${read.you}. Everyone in this group can see what you save.`;
  const latest = new Map();
  for (const o of read.ops) {
    if (o.payload?.kind === 'availability' && !o.voided) latest.set(o.person_key, o);
  }
  $('rota').replaceChildren(
    ...[...latest.values()].map((o) => {
      const li = document.createElement('li');
      li.textContent = `${o.person}: ${o.payload.when}`;
      return li;
    }),
  );
}

$('availability').onsubmit = async (event) => {
  event.preventDefault();
  const when = $('when').value.trim();
  if (!when) return;
  const r = await fetch('svrngs/acts', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ op: { kind: 'availability', when } }),
  });
  if (!r.ok) return say(`Not saved: ${(await r.json()).error ?? r.status}`);
  $('when').value = '';
  say('Saved for this group.');
  await show();
};
show().catch(() => say('Cannot read the rota just now. Your draft is still here.'));
