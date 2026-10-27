// build.mjs — svrngs in three figures, as data. `node build.mjs` writes them
// beside this folder. Each figure reads at two depths: plain sentences for
// anyone, and svrngs's own mechanism words in monospace (DESIGN §1 rule 1;
// the pages do the same), each naming what a reader can look up.
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import * as k from './kit.mjs';
const { C } = k;

const OUT = process.env.OUT || path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const write = (name, s) => { fs.writeFileSync(path.join(OUT, name), s); console.log('wrote', name, (s.length / 1024).toFixed(1), 'KB'); };
const P = (x, y, str, o) => k.para(x, y, str, o).s;

// A frame with its owner's name on a tab: someone's computer, someone's phone.
function owned(x, y, w, h, name) {
  return k.rect(x, y, w, h, { fill: C.paper }) +
    k.rect(x, y, k.width(name, 17) + 28, 34, { fill: C.people }) +
    k.text(x + 13, y + 23, name, { size: 17, weight: 700 });
}
// A box of a figure's own colour with a title and a mechanism line.
function tile(x, y, w, h, fill, title, plain, mono) {
  let s = k.rect(x, y, w, h, { fill });
  s += k.text(x + 14, y + 28, title, { size: 18, weight: 700 });
  const p = k.para(x + 14, y + 52, plain, { w: w - 28, size: 15 });
  s += p.s;
  if (mono) s += P(x + 14, p.end + 22, mono, { w: w - 28, size: 12, mono: true });
  return s;
}

// ============================================================================
// 1 — what it is: a group kept on its own computers, and no center
// ============================================================================
function group() {
  const W = 1200, H = 910; let b = '';
  b += k.band(W, 'Your group’s things, on your group’s computers.',
    'The money you split, the nights you plan, the films you own. No accounts, no passwords, no landlord.', 1, 3);

  // A — the link
  b += k.heading(40, 180, 340, 1, 'A link in your chat');
  b += k.rect(40, 200, 340, 106, { fill: C.paper, sw: 2 });
  b += k.text(54, 222, 'the chat you already use', { size: 13, fill: C.soft });
  b += k.rect(54, 234, 312, 58, { fill: 'whitesmoke', sw: 2 });
  b += k.text(66, 257, 'ana: movie night, saturday?', { size: 16 });
  b += `<text x="66" y="280" font-family="${k.MONO}" font-size="15" fill="blue" text-decoration="underline">group.svrnme.sh/#…</text>`;
  b += P(40, 328, 'The token rides after the #, which no chat’s preview fetches. No bot, no package per chat.', { w: 340, size: 13, mono: true });
  b += k.arrow(150, 372, 92, 412, { sw: 2.5 });
  b += k.arrow(240, 372, 286, 418, { sw: 2.5 });
  b += k.phone(70, 420, 46);
  b += k.rect(76, 468, 34, 20, { fill: C.lit, sw: 2 });
  b += k.text(93, 483, 'Join', { size: 12, weight: 700, anchor: 'middle' });
  b += k.computer(232, 428, 110);
  b += k.text(242, 462, '$ curl … | sh', { size: 12, mono: true });
  b += k.text(40, 535, 'On a phone:', { size: 16, weight: 700 });
  b += P(40, 556, 'type your name, tap Join.', { w: 150, size: 16 });
  b += k.text(222, 535, 'On a computer:', { size: 16, weight: 700 });
  b += P(222, 556, 'paste one line.', { w: 150, size: 16 });
  b += P(40, 610, 'phone: the page dials the group’s computers by key, through a relay. Its key stays sealed in the browser.', { w: 340, size: 13, mono: true });
  b += k.mark(40, 672, 'Ten seconds. Nothing new to learn.', 19);
  b += k.text(40, 672, 'Ten seconds. Nothing new to learn.', { size: 19, weight: 700 });

  // B — the computers
  b += k.heading(420, 180, 400, 2, 'Kept on the group’s computers');
  const at = [[450, 214, 'ana'], [700, 214, 'ben'], [450, 372, 'cy'], [700, 372, 'dee, always on']];
  const mid = ([x, y]) => [x + 45, y + 27];
  for (let i = 0; i < at.length; i++)
    for (let j = i + 1; j < at.length; j++) {
      const [a, c] = [mid(at[i]), mid(at[j])];
      b += k.line(a[0], a[1], c[0], c[1], { sw: 2 });
    }
  for (const [x, y, name] of at) {
    b += k.computer(x, y, 90);
    b += k.rect(x + 12, y + 10, 66, 34, { fill: C.record, sw: 2 });
    b += k.text(x + 45, y + 32, 'record', { size: 12, mono: true, anchor: 'middle' });
    b += k.text(x + 45, y + 82, name, { size: 15, anchor: 'middle' });
  }
  b += k.phone(452, 488, 34);
  b += k.text(469, 568, 'eve', { size: 15, anchor: 'middle' });
  b += k.line(486, 520, 568, 520, { sw: 2.5, dash: '7 5' });
  b += k.rect(568, 500, 120, 40, { fill: C.wire });
  b += k.text(628, 526, 'relay', { size: 15, mono: true, anchor: 'middle' });
  b += k.line(688, 520, 745, 470, { sw: 2.5, dash: '7 5' });
  let p = k.para(420, 610, 'No server. Every computer keeps the whole record; one that stays on keeps the group reachable while the rest sleep.', { w: 400, size: 16 });
  b += p.s;
  b += P(420, p.end + 30, 'node: a full copy of the journal, synced over iroh (QUIC, dialled by key). relay: carries sealed bytes; sees who talks and how much, never what.', { w: 400, size: 13, mono: true });

  // C — the rings
  b += k.heading(860, 180, 300, 3, 'No center but you');
  const cx = 1010, cy = 372;
  for (const r of [150, 118, 86, 54]) b += k.circle(cx, cy, r, { fill: C.paper, sw: 2 });
  b += k.circle(cx, cy, 26, { fill: C.people, sw: 3 });
  b += k.text(cx, cy + 6, 'you', { size: 16, weight: 700, anchor: 'middle' });
  for (const [r, name] of [[54, 'house'], [86, 'friends'], [118, 'the party'], [150, 'the town']])
    b += k.text(cx, cy - r + 19, name, { size: 13, anchor: 'middle' });
  const ang = 0.62, two = [cx + 86 * Math.cos(ang), cy + 86 * Math.sin(ang)];
  b += k.arrow(cx + 22 * Math.cos(ang), cy + 22 * Math.sin(ang), two[0] - 9 * Math.cos(ang), two[1] - 9 * Math.sin(ang), { sw: 2.5, head: 10 });
  b += k.circle(two[0], two[1], 8, { fill: C.people, sw: 2.5 });
  b += k.arrow(two[0] + 9 * Math.cos(ang), two[1] + 9 * Math.sin(ang), cx + 168 * Math.cos(ang), cy + 168 * Math.sin(ang), { sw: 2.5, head: 10, dash: '6 4' });
  p = k.para(860, 560, 'An ask goes further only through someone in both rings who says yes.', { w: 300, size: 16 });
  b += p.s;
  b += P(860, p.end + 30, 'No profile. No directory. No feed. Nobody ranked.', { w: 300, size: 16, weight: 700 });
  b += P(860, p.end + 76, 'Leave whenever you like. A group can leave together, intact.', { w: 300, size: 16 });

  // What we host
  b += k.line(24, 730, W - 24, 730, { sw: 3 });
  b += k.text(40, 760, 'What we host, and you can replace each', { size: 20, weight: 900 });
  const keep = 'None of it holds your group’s things.';
  b += k.mark(W - 40 - k.width(keep, 18), 760, keep, 18);
  b += k.text(W - 40, 760, keep, { size: 18, weight: 700, anchor: 'end' });
  const hosted = [
    ['The page', 'group.svrnme.sh: static files phones open; sees which app, never what is in it'],
    ['The download', 'install.sh and a build per platform, checked against SHA256SUMS'],
    ['A relay', 'n0’s for now; svrngs relay <name> runs your own, and svrngs’s is built'],
  ];
  hosted.forEach(([t, m], i) => {
    const x = 40 + i * 380;
    b += k.rect(x, 776, 360, 66, { fill: C.wire });
    b += k.text(x + 12, 798, t, { size: 16, weight: 700 });
    b += P(x + 12, 817, m, { w: 340, size: 12, mono: true });
  });

  b += k.foot(W, H, 'NORTH_STAR 1, 3 · ten seconds; no center, nobody ranked');
  write('01-group.svg', k.svg(W, H, b, 'svrngs: your group’s things, on your group’s computers',
    'A link in a chat lets a person in from a phone or a computer in seconds. The group’s record lives on every member’s computer, with no server; a relay carries sealed bytes for phones. You stand at the center of your own rings, and nothing ranks a person. We host a page, a download and a relay, each replaceable, none holding the group’s things.'));
}

// ============================================================================
// 2 — for developers: anything on localhost, shared through the door
// ============================================================================
function door() {
  const W = 1200, H = 930; let b = '';
  b += k.band(W, 'If it runs on localhost, your friends can use it.',
    'Lend any app on a port to your group. No login, no SDK, no bundle, no port opened.', 2, 3);

  // The lender's computer
  b += owned(40, 172, 500, 420, 'Ana’s computer');
  b += k.rect(56, 226, 184, 170, { fill: C.app });
  b += k.text(68, 252, 'anything on a port', { size: 16, weight: 700 });
  [['Jellyfin', ':8096'], ['llama-server', ':8080'], ['a dev server', ':5173'], ['Minecraft', ':25565/udp']]
    .forEach(([n, port], i) => {
      b += k.text(68, 282 + i * 26, n, { size: 14 });
      b += k.text(230, 282 + i * 26, port, { size: 12, mono: true, anchor: 'end' });
    });
  b += k.arrow(258, 312, 242, 312, { sw: 3, head: 11 });
  b += P(56, 424, 'Ana presses Share on her home page.', { w: 180, size: 15 });
  b += k.text(56, 572, 'or: svrngs share 8096 --kind jellyfin', { size: 13, mono: true });

  b += k.rect(258, 214, 262, 334, { fill: C.paper, sw: 5 });
  b += k.text(274, 244, 'the door', { size: 20, weight: 900 });
  const rules = [
    [true, 'takes off any “who I am” the request claims'],
    [true, 'adds who is asking, verified:', 'X-Svrngs-Key, -Person, -User'],
    [true, 'puts in the app’s own secret, which never leaves'],
    [true, 'keeps to Ana’s choices: which paths, how many people'],
    [false, 'cuts off anyone removed from the group within seconds'],
  ];
  let y = 274;
  for (const [ok, s, mono] of rules) {
    b += ok ? k.tick(282, y - 5, 7) : k.cross(282, y - 5, 6, C.no);
    const p = k.para(298, y, s, { w: 210, size: 14 });
    b += p.s; y = p.end;
    if (mono) { b += k.text(298, y + 18, mono, { size: 11.5, mono: true }); y += 18; }
    y += 30;
  }

  // The wire
  b += k.rect(540, 326, 150, 28, { fill: C.wire });
  b += k.arrow(556, 340, 524, 340, { sw: 3, head: 11 });
  b += k.text(615, 314, 'over iroh', { size: 13, mono: true, anchor: 'middle' });
  b += P(548, 380, 'encrypted end to end; nothing to open, nothing to install', { w: 150, size: 12, mono: true });
  b += k.line(690, 340, 735, 262, { sw: 3 });
  b += k.line(690, 340, 735, 480, { sw: 3 });

  // The friends
  b += owned(735, 172, 425, 200, 'Ben’s computer');
  b += k.rect(755, 222, 385, 30, { fill: 'whitesmoke', sw: 2 });
  b += k.text(765, 242, 'http://jellyfin-3f9a….localhost:<port>/', { size: 13, mono: true });
  b += P(755, 280, 'Opens in his browser like his own app, already signed in as Ben.', { w: 385, size: 15 });
  b += k.rect(755, 318, 62, 30, { fill: C.lit, sw: 3 });
  b += k.text(786, 339, 'Play', { size: 15, weight: 700, anchor: 'middle' });
  b += k.text(830, 330, 'A game connects to', { size: 14 });
  b += k.text(830, 348, '127.0.0.1:25565, as if it ran here.', { size: 13, mono: true });

  b += owned(735, 392, 425, 200, 'Cy’s phone');
  b += k.phone(757, 442, 44) + k.rect(763, 452, 32, 40, { fill: C.app, sw: 2 });
  b += P(818, 456, 'Opens from the group’s page, each app at an address of its own.', { w: 322, size: 15 });
  b += P(818, 512, 'https://<label>.group.svrnme.sh/ · a worker holds a pass; the page that holds Cy’s key never runs an app', { w: 322, size: 12, mono: true });

  // What an app gets
  b += k.line(24, 626, W - 24, 626, { sw: 3 });
  b += k.text(40, 658, 'Your app gets three things, and needs nothing else', { size: 22, weight: 900 });
  b += tile(40, 676, 360, 124, C.app, 'Who’s asking, verified',
    'In the request’s headers. Key on X-Svrngs-Key; show X-Svrngs-Person.',
    'the door strips any a client sent');
  b += tile(420, 676, 360, 124, C.app, 'A record the group keeps',
    'It outlives the computer the app runs on, and every copy reads the same.',
    'GET, POST /svrngs/acts · or name acts in x-svrngs-append');
  b += tile(800, 676, 360, 124, C.app, 'Reach to every computer',
    'Lent, an app goes with its lender. Carried, it is a folder the group keeps, and opens on any computer that is on.',
    'apps/<kind>/index.html');
  b += P(40, 828, 'What can act as you is said by what it can do (know who you are, reach an app as you, read or write the record, decide for you), never by what kind of app it is. We build the rails and small examples to copy and bend; the finished apps are yours.', { w: 1120, size: 15 });

  b += k.foot(W, H, 'NORTH_STAR 2 · no login, no SDK, no bundle');
  write('02-door.svg', k.svg(W, H, b, 'svrngs: if it runs on localhost, your friends can use it',
    'Ana lends an app on a port. Her computer’s door takes off any identity a request claims, adds the verified person, injects the app’s secret, keeps to her limits and cuts off anyone removed. Ben opens it in his browser or plays a game at 127.0.0.1; Cy opens it on a phone at its own address. An app gets three things: who is asking, a record the group keeps, and reach to every computer.'));
}

// ============================================================================
// 3 — for engineers: the record, the shape of the code, and the gates
// ============================================================================
function record() {
  const W = 1200, H = 1150; let b = '';
  b += k.band(W, 'Same acts, same state.',
    'Small enough to read in an afternoon, and checked so it stays that way.', 3, 3);

  // 1 — the record
  b += k.heading(40, 180, 1120, 1, 'The record: signed acts, and every copy folds them the same way');
  b += k.phone(50, 212, 30);
  b += P(94, 228, 'A phone signs its own acts.', { w: 100, size: 14 });
  b += k.computer(44, 306, 56);
  b += P(110, 316, 'A computer signs its own person’s.', { w: 104, size: 14 });
  b += P(40, 400, 'none writes as anyone else (not_yours)', { w: 170, size: 12, mono: true });
  b += k.arrow(194, 236, 222, 270, { sw: 2.5, head: 10 });
  b += k.arrow(206, 330, 222, 320, { sw: 2.5, head: 10 });

  b += k.rect(224, 204, 228, 236, { fill: C.record });
  b += k.text(238, 228, 'an act', { size: 16, weight: 700 });
  [['group', 'its id'], ['actor', 'a key'], ['seq', '7'], ['prev', 'id of seq 6'], ['ts', 'a stamp'],
    ['body', 'expenses/spend …'], ['id', 'BLAKE3 of all above'], ['sig', 'over the id']]
    .forEach(([f, v], i) => {
      b += k.text(238, 254 + i * 21, f, { size: 13, mono: true, weight: 700 });
      b += k.text(294, 254 + i * 21, v, { size: 13, mono: true });
    });
  b += k.text(238, 428, 'Correct voids, never erases.', { size: 13 });
  b += k.arrow(452, 320, 474, 320, { sw: 2.5, head: 10 });

  b += k.text(480, 226, 'a grow-only set, linked like git', { size: 15, weight: 700 });
  const sq = (x, y) => k.rect(x, y, 24, 24, { fill: C.record, sw: 2 });
  const back = (x1, y1, x2, y2) => k.arrow(x1, y1, x2, y2, { sw: 2, head: 8 });
  for (const x of [484, 544, 604, 664]) b += sq(x, 252);
  for (const x of [544, 604, 664]) b += back(x, 264, x - 34, 264);
  b += sq(484, 314) + sq(544, 314) + sq(604, 296) + sq(604, 336);
  b += back(544, 326, 510, 326) + back(604, 308, 570, 322) + back(604, 348, 570, 332);
  b += k.text(636, 314, '← both kept', { size: 12, mono: true });
  b += P(480, 390, 'a fork voids nothing: both held, and named. Copies converge by union; sync sends only what is missing.', { w: 240, size: 12, mono: true });
  b += k.arrow(724, 320, 746, 320, { sw: 2.5, head: 10 });

  b += k.rect(748, 214, 160, 214, { fill: C.paper, sw: 4 });
  b += k.text(762, 242, 'admit', { size: 18, mono: true, weight: 700 });
  b += k.text(762, 266, 'one total order', { size: 14 });
  b += k.text(762, 284, '(ts, actor, seq, id)', { size: 11.5, mono: true });
  b += k.text(762, 314, 'refused, each named:', { size: 12 });
  ['WrongPrevious', 'Ungranted', 'CarriedForAnother'].forEach((n, i) => {
    b += k.rect(760, 324 + i * 30, 136, 24, { fill: C.no, sw: 2 });
    b += k.text(768, 341 + i * 30, n, { size: 11.5, mono: true });
  });
  b += k.arrow(908, 320, 930, 320, { sw: 2.5, head: 10 });

  ['who is in the group', 'Money’s balances', 'who came, in Events', 'what the house’s rules say']
    .forEach((n, i) => {
      b += k.rect(932, 214 + i * 46, 228, 36, { fill: C.paper, sw: 2 });
      b += k.rect(932, 214 + i * 46, 10, 36, { fill: C.record, sw: 2 });
      b += k.text(954, 238 + i * 46, n, { size: 15 });
    });
  b += P(932, 410, 'permute or duplicate the acts: nothing changes (a property test)', { w: 228, size: 12, mono: true });
  const auth = 'An entry’s author is its signer. Replicas only carry.';
  b += k.mark(40, 478, auth, 18);
  b += k.text(40, 478, auth, { size: 18, weight: 700 });

  // 2 — the shape of the code
  b += k.heading(40, 546, 540, 2, 'Ownership is a tree; I/O lives at the edge');
  const layers = [
    ['svrngs', 'the command: new, join, share, leave, relay', C.paper],
    ['svrngs-node', 'adapters: iroh sync, the door, the APIs', C.paper],
    ['svrngs-runtime', 'the one owner of each journal; one writer', C.paper],
    ['svrngs-core', 'pure: time and keys come in as arguments', C.record],
  ];
  layers.forEach(([n, d, fill], i) => {
    const x = 40 + i * 16, yy = 566 + i * 34, w = 540 - i * 32, h = 226 - i * 42;
    b += k.rect(x, yy, w, h, { fill, sw: 3 });
    b += k.text(x + 12, yy + 23, n, { size: 14, mono: true, weight: 700 });
    b += k.text(x + 12 + k.width(n, 14, true) + 12, yy + 23, d, { size: 13 });
  });
  b += P(100, 716, 'no_std: the compiler refuses files, clocks, the network, threads and HashMap here. A group’s state is a pure function of its acts.', { w: 410, size: 13 });
  b += P(40, 818, 'svrngs-web, the phone’s page in wasm, builds on the same core.', { w: 540, size: 12, mono: true });

  // 3 — the fork
  b += k.heading(620, 546, 540, 3, 'Take what exists: fork it, own it');
  b += k.rect(620, 566, 210, 70, { fill: C.paper, sw: 3, dash: '8 6' });
  b += k.text(634, 594, 'commonwealth-ai', { size: 14, mono: true, weight: 700 });
  b += k.text(634, 618, 'its ring code', { size: 14 });
  b += k.rect(950, 566, 210, 70, { fill: C.paper, sw: 4 });
  b += k.text(964, 594, 'svrngs', { size: 14, mono: true, weight: 700 });
  b += k.text(964, 618, 'owns it from then on', { size: 14 });
  b += k.arrow(830, 590, 948, 590, { sw: 3, head: 11 });
  b += k.text(889, 582, 'forked', { size: 12, mono: true, anchor: 'middle' });
  b += k.arrow(950, 618, 832, 618, { sw: 2, head: 9, dash: '6 5' });
  b += k.cross(891, 618, 7, C.no);
  b += k.text(889, 662, 'links nothing back', { size: 12, mono: true, anchor: 'middle' });
  let y = 690;
  for (const [s, mono] of [
    ['Behavior kept first, then changed, each in its own commit.'],
    ['Every forked file names its original:', 'ported from commonwealth-ai <path>@<commit>'],
    ['Build what the next demo needs; the rest waits for its trigger.'],
    ['A change of direction rewrites the design. It never adds an exception.'],
  ]) {
    b += k.rect(620, y - 11, 9, 9, { fill: C.ink, sw: 0 });
    b += k.text(640, y, s, { size: 15 });
    if (mono) { y += 20; b += k.text(640, y, mono, { size: 12, mono: true }); }
    y += 34;
  }

  // 4 — the gates
  b += k.heading(40, 908, 1120, 4, 'Every check failed on a planted input before it passed');
  b += k.text(1160, 908, 'scripts/check.sh', { size: 15, mono: true, anchor: 'end' });
  const gates = [
    ['principles', 'two pinned documents, word for word'],
    ['lean', 'a line budget; a raise is the operator’s, after a cut'],
    ['vocabulary', 'the infra never learns what a film or an expense is'],
    ['canaries', 'Money, Events, and the whole web in Chromium and WebKit'],
    ['the ceiling', '25 people, 10,000 acts: an idle round under 0.1 s'],
    ['trust floor', 'every page’s policy, every listener; then fmt, clippy, tests'],
  ];
  gates.forEach(([t, d], i) => {
    const x = 40 + i * 189;
    b += k.rect(x, 928, 176, 112, { fill: C.paper });
    b += k.rect(x, 928, 176, 10, { fill: C.ok, sw: 3 });
    b += k.text(x + 12, 962, t, { size: 16, weight: 700 });
    b += P(x + 12, 984, d, { w: 160, size: 13 });
  });
  let x = 40;
  b += k.text(x, 1074, 'Four verdicts:', { size: 16, weight: 700 }); x += 124;
  for (const [n, fill, dash] of [['passed', C.ok], ['failed', C.no], ['could not judge', C.maybe], ['never ran', C.paper, '5 4']]) {
    const w = k.width(n, 13, true) + 20;
    b += k.rect(x, 1056, w, 26, { fill, sw: 2, dash });
    b += k.text(x + 10, 1074, n, { size: 13, mono: true });
    x += w + 10;
  }
  b += k.text(x + 8, 1074, 'An absence is reported, never defaulted.', { size: 15 });

  b += k.foot(W, H, 'NORTH_STAR 2 · could you read all of it? did you have to?');
  write('03-record.svg', k.svg(W, H, b, 'svrngs: same acts, same state',
    'Phones and computers sign their own acts. Each act names its previous one inside its hash; copies converge by union, and admission puts them in one total order, refusing what breaks the rules by name. Folds over that order give who is in the group and each app’s state, the same on every copy. The code nests from a pure core outward to the command, forked once from commonwealth-ai and linking nothing back, and every check in scripts/check.sh failed on a planted input before it passed.'));
}

group();
door();
record();
