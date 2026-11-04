// PREDICTIONS.md §2 here, bar by bar (../judge.mjs's, bar 5's words now V1.1.3's, and bars 9 and 10), over what run.mjs read: one of passed,
// failed, could-not-judge or never-ran each, with the numbers it read.
//
//   node judge.mjs <data dir>     writes <data dir>/verdicts.json
import { readFileSync, writeFileSync } from 'node:fs';

const DATA = process.argv[2];
const S = JSON.parse(readFileSync(`${DATA}/summary.json`, 'utf8'));
const WHO = ['bo', 'cy', 'di'];
const NAME = { bo: 'Bo', cy: 'Cy', di: 'Di' };
const verdicts = [];
const say = (bar, verdict, read) => verdicts.push({ bar, verdict, read });
const within = (ms, limit) => ms !== null && ms !== undefined && ms <= limit;
const ran = (...xs) => xs.every((x) => x !== undefined && x !== null);

// 1. The doc opens.
if (!ran(S.open)) say(1, 'never-ran', null);
else {
  const read = { bo_ms: S.open.bo?.pressed?.ms ?? null, di_ms: S.open.di?.pressed?.ms ?? null, cy_ms: S.open.cy?.opened?.ms ?? null, failed: [S.open.bo, S.open.cy, S.open.di].map((o) => o?.failed).filter(Boolean) };
  say(1, within(read.bo_ms, 15_000) && within(read.di_ms, 15_000) && within(read.cy_ms, 5_000) ? 'passed' : 'failed', read);
}

// 2. Named, and everyone here.
if (!ran(S.named, S.everyone)) say(2, 'never-ran', null);
else {
  const read = { named_ms: S.named.map((a) => a.ms), everyone_ms: S.everyone.map((a) => a.ms), you: (S.read_open ?? []).map((r) => r.you), here: (S.read_open ?? []).map((r) => r.here) };
  say(2, read.named_ms.every((m) => within(m, 5_000)) && read.everyone_ms.every((m) => within(m, 5_000)) ? 'passed' : 'failed', read);
}

// 3. Live.
if (!ran(S.live)) say(3, 'never-ran', null);
else say(3, S.live.every((l) => within(l.ms_after_last_other_keystroke, 2_000)) ? 'passed' : 'failed', S.live);

// 4. Each paragraph names its last editor: every page's notes, beside the
// paragraphs they are about. Notes are drawn in the paragraphs' order, one
// per paragraph an edit touched; paired in order when there is one each.
if (!ran(S.read_typed)) say(4, 'never-ran', null);
else {
  const pages = S.read_typed.map((r, i) => {
    const paras = r.paragraphs.filter((p) => p.trim());
    const notes = r.note_lines;
    const paired = notes.length === r.paragraphs.length ? r.paragraphs.map((p, k) => [p, notes[k]]).filter(([p]) => p.trim()) : null;
    const right = paired?.every(([p, n]) => { const w = S.writers?.[p] ?? NAME[WHO.find((x) => p.startsWith(NAME[x]))]; return w && n.includes(`by ${w} `); }) ?? null;
    return { who: WHO[i], paragraphs: r.paragraphs, notes, paired: Boolean(paired), right, names: WHO.every((w) => notes.some((n) => n.includes(`by ${NAME[w]} `))), waited_ms: S.notes_wait?.[i]?.ms ?? null, filled: paras.length };
  });
  const v = pages.every((p) => p.names && p.right !== false && within(p.waited_ms, 3_000)) ? (pages.every((p) => p.right) ? 'passed' : 'could-not-judge') : 'failed';
  say(4, v, pages);
}

// 5. Goodnight.
if (!ran(S.goodnight)) say(5, 'never-ran', null);
else {
  const read = S.goodnight.map((g, i) => ({ ...g, words: S.read_night?.[i]?.night ?? null }));
  say(5, read.every((r) => within(r.ms_after_lid, 7_000) && /☾ Alex’s window is dark/.test(r.words ?? '') && /Goodnight from Alex’s\./.test(r.words ?? '')) ? 'passed' : 'failed', read);
}

// 6. Writing goes on.
if (!ran(S.kept_own, S.kept_said)) say(6, 'never-ran', null);
else {
  const read = WHO.map((w, i) => ({ who: w, own_ms: S.kept_own[i].ms, said_ms: S.kept_said[i].ms, kept: S.read_away?.[i]?.kept ?? null }));
  say(6, read.every((r) => r.own_ms !== null && r.said_ms !== null) ? 'passed' : 'failed', read);
}

// 7. It joins.
if (!ran(S.joined, S.read_joined)) say(7, 'never-ran', null);
else {
  const docs = S.read_joined.map((r) => r.paragraphs.filter((p) => p.trim()).join('\n'));
  const read = { joined: S.joined, one_doc: docs.every((d) => d === docs[0]), paragraphs: S.read_joined[0].paragraphs };
  say(7, read.one_doc && S.joined.every((j) => within(j.goodnight_gone_ms, 30_000) && within(j.all_six_ms, 30_000)) ? 'passed' : 'failed', read);
}

// 8. Nothing lost.
if (!ran(S.fresh, S.acts)) say(8, 'never-ran', null);
else {
  const persons = [...new Set(S.acts.acts.map((a) => a.person))];
  const read = { fresh_ms: S.fresh.has?.ms ?? null, acts: S.acts.acts.length, persons };
  say(8, read.fresh_ms !== null && WHO.every((w) => persons.includes(NAME[w])) ? 'passed' : 'failed', read);
}

// 9. No blank paragraph: every page holds the seven lines and nothing else.
if (!ran(S.read_joined, S.six)) say(9, 'never-ran', null);
else {
  const read = S.read_joined.map((r, i) => ({ who: WHO[i], paragraphs: r.paragraphs, blank: r.paragraphs.filter((p) => !p.trim()).length }));
  const want = [...S.six].sort().join('\n');
  say(9, read.every((r) => r.blank === 0 && [...r.paragraphs].sort().join('\n') === want) ? 'passed' : 'failed', read);
}

// 10. On each phone, each note drawn inside its own paragraph's box, naming its writer.
if (!ran(S.read_joined, S.writers)) say(10, 'never-ran', null);
else {
  const read = ['bo', 'di'].map((w) => {
    const r = S.read_joined[WHO.indexOf(w)];
    const placed = (r.placed ?? []).map((p) => ({ ...p, paragraph: r.paragraphs[p.block] ?? null }));
    const ok = placed.length === r.paragraphs.length && placed.every((p) => p.paragraph !== null && p.note.includes(`by ${S.writers[p.paragraph]} `)) && new Set(placed.map((p) => p.block)).size === placed.length;
    return { who: w, ok, placed };
  });
  say(10, read.every((r) => r.ok) ? 'passed' : 'failed', read);
}

writeFileSync(`${DATA}/verdicts.json`, JSON.stringify(verdicts, null, 1));
for (const v of verdicts) console.log(`${v.bar} ${v.verdict}`);
