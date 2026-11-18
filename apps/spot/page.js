// SPDX-License-Identifier: AGPL-3.0-or-later
// spot's chat page (page.html): its own file, so a policy of script-src 'self' runs it.
import { TAGS, parse } from './markdown.js';

const $ = (id) => document.getElementById(id);
const store = {
  get: () => {
    try {
      return JSON.parse(localStorage.getItem('spot-chat') ?? '[]');
    } catch {
      return [];
    }
  },
  set: (msgs) => {
    try {
      localStorage.setItem('spot-chat', JSON.stringify(msgs));
    } catch {}
  },
};
let msgs = store.get();
let running = null; // the AbortController of the answer in flight

function draw() {
  $('log').replaceChildren();
  if (msgs.length === 0) $('log').append(emptyNote);
  for (const m of msgs) $('log').append(bubble(m));
  $('log').scrollTop = $('log').scrollHeight;
}
const emptyNote = $('empty');
function bubble(m) {
  const el = document.createElement('div');
  el.className = `msg ${m.role}`;
  if (m.thinking) {
    const d = document.createElement('details');
    const s = document.createElement('summary');
    s.textContent = m.content ? 'Thought' : 'Thinking…';
    d.append(s, m.thinking);
    el.append(d);
  }
  if (m.role === 'assistant') el.append(markdown(m.content ?? ''));
  else el.append(m.content ?? '');
  return el;
}

// An answer's Markdown, built node by node: text is only ever text.
function markdown(text) {
  const box = document.createElement('div');
  box.className = 'md';
  box.append(...parse(text).map(build));
  return box;
}
function build(n) {
  if (typeof n === 'string') return document.createTextNode(n);
  if (!TAGS.has(n.tag)) return document.createTextNode('');
  const el = document.createElement(n.tag);
  if (n.tag === 'a') {
    el.href = n.href;
    el.target = '_blank';
    el.rel = 'noopener noreferrer';
  }
  if (n.align) el.style.textAlign = n.align;
  if (n.start) el.start = n.start;
  el.append(...n.children.map(build));
  if (n.tag === 'table') {
    const wide = document.createElement('div');
    wide.className = 'wide';
    wide.append(el);
    return wide;
  }
  if (n.tag === 'pre') {
    const copy = document.createElement('button');
    copy.type = 'button';
    copy.className = 'copy';
    copy.textContent = 'Copy';
    copy.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(el.textContent);
        copy.textContent = 'Copied';
      } catch {
        copy.textContent = 'Could not copy';
      }
      setTimeout(() => (copy.textContent = 'Copy'), 1500);
    });
    const box = document.createElement('div');
    box.className = 'code';
    box.append(copy, el);
    return box;
  }
  return el;
}

async function standing() {
  try {
    const r = await fetch('spot.json', { cache: 'no-store' });
    if (!r.ok) throw new Error(`${r.status}`);
    const s = await r.json();
    document.title = s.name;
    $('title').textContent = s.name;
    const was = $('model').value;
    $('model').replaceChildren(...s.models.map((id) => new Option(id, id)));
    if (s.models.includes(was)) $('model').value = was;
    $('model').hidden = s.models.length < 2;
    const parts = [];
    if (s.until)
      parts.push(
        `until ${new Date(s.until).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })}`,
      );
    if (s.budget)
      parts.push(
        `${(s.budget - s.spent).toLocaleString()} of ${s.budget.toLocaleString()} tokens left`,
      );
    if (s.yours) parts.push(`you've used ${s.yours.toLocaleString()}`);
    $('standing').textContent = s.ended ?? (parts.join(' · ') || 'lent until its lender stops it');
    $('standing').classList.toggle('over', !!s.ended);
    emptyNote.textContent = `Your messages go to ${s.provider} under the account of whoever lends this to you. They see who used how many tokens, not what you wrote. Your chat stays in this browser.`;
  } catch (e) {
    $('standing').textContent = `The lender's computer did not answer (${e.message}).`;
    $('standing').classList.add('over');
  }
}

async function send(text) {
  msgs.push({ role: 'user', content: text });
  const answer = { role: 'assistant', content: '', thinking: '' };
  msgs.push(answer);
  draw();
  running = new AbortController();
  $('send').textContent = 'Stop';
  try {
    const r = await fetch('v1/chat/completions', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        model: $('model').value,
        stream: true,
        messages: msgs
          .slice(0, -1)
          .filter((m) => m.role !== 'error')
          .map(({ role, content }) => ({ role, content })),
      }),
      signal: running.signal,
    });
    if (!r.ok) {
      const said = await r.json().catch(() => null);
      throw new Error(said?.error?.message ?? `the lend answered ${r.status}`);
    }
    // Read the stream as it comes; if a hop between gathers it, it reads the same, at once.
    const reader = r.body.pipeThrough(new TextDecoderStream()).getReader();
    let buffer = '';
    let current = $('log').lastElementChild;
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      buffer += value;
      const events = buffer.split('\n\n');
      buffer = events.pop();
      for (const ev of events) {
        const data = ev.replace(/^data: ?/m, '').trim();
        if (!data || data === '[DONE]') continue;
        const delta = JSON.parse(data).choices?.[0]?.delta ?? {};
        answer.content += delta.content ?? '';
        answer.thinking += delta.reasoning_content ?? delta.reasoning ?? '';
      }
      const fresh = bubble(answer);
      current.replaceWith(fresh);
      current = fresh;
      fresh.scrollIntoView({ block: 'end' });
    }
  } catch (e) {
    if (e.name !== 'AbortError') {
      if (!answer.content) msgs.pop();
      msgs.push({ role: 'error', content: e.message });
    }
  } finally {
    if (!answer.thinking) delete answer.thinking;
    running = null;
    $('send').textContent = 'Send';
    store.set(msgs);
    draw();
    standing();
  }
}

$('ask').addEventListener('submit', (e) => {
  e.preventDefault();
  if (running) return running.abort();
  const text = $('text').value.trim();
  if (!text) return;
  $('text').value = '';
  $('text').style.height = '';
  send(text);
});
$('text').addEventListener('keydown', (e) => {
  const touch = matchMedia('(pointer: coarse)').matches;
  if (e.key === 'Enter' && !e.shiftKey && !touch && !e.isComposing) {
    e.preventDefault();
    $('ask').requestSubmit();
  }
});
$('text').addEventListener('input', () => {
  $('text').style.height = '';
  $('text').style.height = `${$('text').scrollHeight}px`;
});
$('new').addEventListener('click', () => {
  running?.abort();
  msgs = [];
  store.set(msgs);
  draw();
});

draw();
standing();
