// build.mjs — svrngs in three figures, as data. `node build.mjs` writes them
// beside this folder. One point per figure, in as few words as make it;
// build-v1.mjs keeps the fuller set. Mechanism words go in monospace, as
// the pages say them (DESIGN §1 rule 1).
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import * as k from './kit.mjs';
const { C } = k;

const OUT = process.env.OUT || path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const write = (name, s) => { fs.writeFileSync(path.join(OUT, name), s); console.log('wrote', name, (s.length / 1024).toFixed(1), 'KB'); };
const P = (x, y, str, o) => k.para(x, y, str, o).s;
const lit = (x, y, str, size, anchor = 'start') => {
  const x0 = anchor === 'end' ? x - k.width(str, size) : x;
  return k.mark(x0, y, str, size) + k.text(x, y, str, { size, weight: 700, anchor });
};

// A frame with its owner's name on a tab: someone's computer, someone's phone.
function owned(x, y, w, h, name) {
  return k.rect(x, y, w, h, { fill: C.paper }) +
    k.rect(x, y, k.width(name, 17) + 28, 34, { fill: C.people }) +
    k.text(x + 13, y + 23, name, { size: 17, weight: 700 });
}

// ============================================================================
// 1 — a group, kept on its own computers, with no center
// ============================================================================
function group() {
  const W = 1200, H = 700; let b = '';
  b += k.band(W, 'Your group’s things, on your group’s computers.', 'No accounts. No passwords. No landlord.', 1, 3);

  // A — the link
  b += k.heading(40, 180, 340, 1, 'A link in your chat');
  b += k.rect(40, 206, 340, 66, { fill: 'whitesmoke', sw: 2 });
  b += k.text(56, 232, 'ana: movie night saturday?', { size: 17 });
  b += `<text x="56" y="258" font-family="${k.MONO}" font-size="16" fill="blue" text-decoration="underline">group.svrnme.sh/#…</text>`;
  b += k.arrow(150, 292, 96, 336, { sw: 2.5 });
  b += k.arrow(250, 292, 290, 340, { sw: 2.5 });
  b += k.phone(72, 346, 46);
  b += k.rect(78, 394, 34, 20, { fill: C.lit, sw: 2 });
  b += k.text(95, 409, 'Join', { size: 12, weight: 700, anchor: 'middle' });
  b += k.computer(236, 352, 110);
  b += k.text(246, 386, '$ curl … | sh', { size: 12, mono: true });
  b += k.text(40, 464, 'Phone: your name, Join.', { size: 16 });
  b += k.text(232, 464, 'Computer: one line.', { size: 16 });
  b += lit(40, 520, 'Ten seconds.', 22);

  // B — the computers
  b += k.heading(420, 180, 400, 2, 'Kept on your computers');
  const at = [[450, 210, 'ana'], [700, 210, 'ben'], [450, 360, 'cy'], [700, 360, 'dee, always on']];
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
  b += k.phone(452, 470, 34);
  b += k.text(469, 550, 'eve', { size: 15, anchor: 'middle' });
  b += k.line(486, 502, 568, 502, { sw: 2.5, dash: '7 5' });
  b += k.rect(568, 482, 120, 40, { fill: C.wire });
  b += k.text(628, 508, 'relay', { size: 15, mono: true, anchor: 'middle' });
  b += k.line(688, 502, 745, 458, { sw: 2.5, dash: '7 5' });
  b += k.text(568, 542, 'sees who, never what', { size: 12, mono: true });
  b += P(420, 590, 'No server. Every computer holds the whole record.', { w: 400, size: 17 });

  // C — the rings
  b += k.heading(860, 180, 300, 3, 'No center but you');
  const cx = 1010, cy = 366;
  for (const r of [150, 118, 86, 54]) b += k.circle(cx, cy, r, { fill: C.paper, sw: 2 });
  b += k.circle(cx, cy, 26, { fill: C.people, sw: 3 });
  b += k.text(cx, cy + 6, 'you', { size: 16, weight: 700, anchor: 'middle' });
  for (const [r, name] of [[54, 'house'], [86, 'friends'], [118, 'the party'], [150, 'the town']])
    b += k.text(cx, cy - r + 19, name, { size: 13, anchor: 'middle' });
  const ang = 0.62, two = [cx + 86 * Math.cos(ang), cy + 86 * Math.sin(ang)];
  b += k.arrow(cx + 22 * Math.cos(ang), cy + 22 * Math.sin(ang), two[0] - 9 * Math.cos(ang), two[1] - 9 * Math.sin(ang), { sw: 2.5, head: 10 });
  b += k.circle(two[0], two[1], 8, { fill: C.people, sw: 2.5 });
  b += k.arrow(two[0] + 9 * Math.cos(ang), two[1] + 9 * Math.sin(ang), cx + 168 * Math.cos(ang), cy + 168 * Math.sin(ang), { sw: 2.5, head: 10, dash: '6 4' });
  b += P(860, 556, 'Reach grows only through someone who says yes.', { w: 300, size: 17 });
  b += k.text(860, 612, 'No profile. No feed. Nobody ranked.', { size: 17, weight: 700 });

  b += k.foot(W, H);
  write('01-group.svg', k.svg(W, H, b, 'svrngs: your group’s things, on your group’s computers',
    'A link in a chat lets a person in from a phone or a computer in seconds. Every member’s computer holds the whole record, with no server; a relay carries sealed bytes for phones. You stand at the center of your own rings, and nothing ranks a person.'));
}

// ============================================================================
// 2 — anything on localhost, shared through the door
// ============================================================================
function door() {
  const W = 1200, H = 720; let b = '';
  b += k.band(W, 'If it runs on localhost, your friends can use it.', 'No login. No SDK. No port opened.', 2, 3);

  // The lender's computer
  b += owned(40, 172, 500, 330, 'Ana’s computer');
  b += k.rect(60, 226, 184, 140, { fill: C.app });
  b += k.text(72, 252, 'any app on a port', { size: 16, weight: 700 });
  [['Jellyfin', ':8096'], ['a model', ':8080'], ['Minecraft', ':25565']].forEach(([n, port], i) => {
    b += k.text(72, 284 + i * 26, n, { size: 15 });
    b += k.text(234, 284 + i * 26, port, { size: 13, mono: true, anchor: 'end' });
  });
  b += k.rect(60, 398, 74, 32, { fill: C.lit, sw: 3 });
  b += k.text(97, 420, 'Share', { size: 16, weight: 700, anchor: 'middle' });
  b += k.text(146, 420, 'one press', { size: 15 });
  b += k.arrow(268, 300, 246, 300, { sw: 3, head: 11 });

  b += k.rect(268, 214, 252, 202, { fill: C.paper, sw: 5 });
  b += k.text(286, 246, 'the door', { size: 22, weight: 900 });
  [[true, 'verifies who’s asking'], [true, 'keeps the app’s secret'], [false, 'shuts out anyone removed']]
    .forEach(([ok, s], i) => {
      const y = 290 + i * 44;
      b += ok ? k.tick(294, y - 5, 7) : k.cross(294, y - 5, 6, C.no);
      b += P(312, y, s, { w: 196, size: 16 });
    });

  // The wire
  b += k.rect(540, 326, 150, 28, { fill: C.wire });
  b += k.arrow(556, 340, 524, 340, { sw: 3, head: 11 });
  b += k.text(615, 380, 'encrypted', { size: 13, mono: true, anchor: 'middle' });
  b += k.line(690, 340, 735, 262, { sw: 3 });
  b += k.line(690, 340, 735, 420, { sw: 3 });

  // The friends
  b += owned(735, 172, 425, 160, 'Ben’s computer');
  b += k.rect(755, 222, 385, 30, { fill: 'whitesmoke', sw: 2 });
  b += k.text(765, 242, 'http://jellyfin-3f9a.localhost/', { size: 13, mono: true });
  b += k.rect(755, 274, 62, 30, { fill: C.lit, sw: 3 });
  b += k.text(786, 295, 'Play', { size: 15, weight: 700, anchor: 'middle' });
  b += k.text(830, 295, '→ 127.0.0.1:25565', { size: 14, mono: true });

  b += owned(735, 352, 425, 150, 'Cy’s phone');
  b += k.phone(757, 398, 40) + k.rect(763, 408, 28, 36, { fill: C.app, sw: 2 });
  b += k.text(818, 432, 'Each app at its own address.', { size: 16 });

  // What an app gets
  b += k.line(24, 536, W - 24, 536, { sw: 3 });
  b += k.text(40, 568, 'Your app gets three things', { size: 22, weight: 900 });
  [['Who’s asking', 'X-Svrngs-Key'], ['A shared record', '/svrngs/acts'], ['Every member’s computer', 'lent, or carried by the group']]
    .forEach(([t, m], i) => {
      const x = 40 + i * 380;
      b += k.rect(x, 586, 360, 64, { fill: C.app });
      b += k.text(x + 14, 612, t, { size: 18, weight: 700 });
      b += k.text(x + 14, 636, m, { size: 13, mono: true });
    });

  b += k.foot(W, H);
  write('02-door.svg', k.svg(W, H, b, 'svrngs: if it runs on localhost, your friends can use it',
    'Ana shares an app on a port. Her computer’s door says who is asking, keeps the app’s secret in, and shuts out anyone removed. Ben opens it in his browser or plays at 127.0.0.1; Cy opens it on a phone. An app gets three things: who is asking, a shared record, and every member’s computer.'));
}

// ============================================================================
// 3 — the record, the shape of the code, and every piece we run replaceable
// ============================================================================
function record() {
  const W = 1200, H = 1000; let b = '';
  b += k.band(W, 'Same acts, same state.',
    'Small enough to read in an afternoon. Batteries included, every one replaceable.', 3, 3);

  // 1 — the record
  b += k.heading(40, 180, 1120, 1, 'Every copy folds the same signed acts');
  b += k.phone(56, 222, 30);
  b += k.computer(48, 312, 64);
  b += k.text(40, 390, 'Each device signs its own.', { size: 14 });
  b += k.arrow(150, 270, 196, 290, { sw: 2.5, head: 10 });

  b += k.rect(200, 214, 220, 162, { fill: C.record });
  b += k.text(214, 238, 'an act', { size: 16, weight: 700 });
  [['actor', 'a key'], ['prev', 'its last act'], ['body', 'expenses/spend'], ['sig', 'over the hash']]
    .forEach(([f, v], i) => {
      b += k.text(214, 266 + i * 23, f, { size: 13, mono: true, weight: 700 });
      b += k.text(270, 266 + i * 23, v, { size: 13, mono: true });
    });
  b += k.arrow(420, 296, 448, 296, { sw: 2.5, head: 10 });

  const sq = (x, y) => k.rect(x, y, 24, 24, { fill: C.record, sw: 2 });
  const back = (x1, y1, x2, y2) => k.arrow(x1, y1, x2, y2, { sw: 2, head: 8 });
  for (const x of [462, 522, 582]) b += sq(x, 236);
  for (const x of [522, 582]) b += back(x, 248, x - 34, 248);
  b += sq(462, 304) + sq(522, 304) + sq(582, 284) + sq(582, 326);
  b += back(522, 316, 488, 316) + back(582, 296, 548, 312) + back(582, 338, 548, 322);
  b += k.text(458, 390, 'linked like git', { size: 13, mono: true });
  b += k.arrow(640, 296, 668, 296, { sw: 2.5, head: 10 });

  b += k.rect(670, 214, 190, 162, { fill: C.paper, sw: 4 });
  b += k.text(686, 242, 'admit', { size: 18, mono: true, weight: 700 });
  b += k.text(686, 266, 'one order', { size: 15 });
  ['WrongPrevious', 'Ungranted'].forEach((n, i) => {
    b += k.rect(684, 288 + i * 34, 160, 26, { fill: C.no, sw: 2 });
    b += k.text(694, 306 + i * 34, n, { size: 12.5, mono: true });
  });
  b += k.arrow(860, 296, 888, 296, { sw: 2.5, head: 10 });

  ['who’s in', 'Money’s balances', 'the house’s rules'].forEach((n, i) => {
    b += k.rect(890, 222 + i * 52, 270, 40, { fill: C.paper, sw: 2 });
    b += k.rect(890, 222 + i * 52, 10, 40, { fill: C.record, sw: 2 });
    b += k.text(914, 248 + i * 52, n, { size: 16 });
  });
  b += lit(40, 452, 'An entry’s author is its signer.', 19);

  // 2 — the shape of the code
  b += k.heading(40, 526, 540, 2, 'Ownership is a tree');
  [['svrngs', 'the command'], ['svrngs-node', 'the I/O: sync, the door'], ['svrngs-runtime', 'one writer per journal'],
    ['svrngs-core', 'pure: no files, clock or network', C.record]].forEach(([n, d, fill = C.paper], i) => {
    const x = 40 + i * 16, yy = 546 + i * 34, w = 540 - i * 32, h = 170 - i * 42;
    b += k.rect(x, yy, w, h, { fill, sw: 3 });
    b += k.text(x + 12, yy + 24, n, { size: 15, mono: true, weight: 700 });
    b += k.text(x + 12 + k.width(n, 15, true) + 12, yy + 24, d, { size: 15 });
  });
  b += k.text(40, 742, 'svrngs-web: the phone’s page, in wasm, on the same core', { size: 12, mono: true });

  // 3 — what we run, and what you can run instead
  b += k.heading(620, 526, 540, 3, 'Ours by default, or yours');
  const rows = [
    ['Phones’ page', 'group.svrnme.sh', 'any static host', '--origin'],
    ['Relay', 'n0’s, for now', 'your own', 'svrngs relay'],
    ['Install', 'our builds, checked', 'build it', 'cargo build'],
  ];
  b += k.text(620, 566, 'piece', { size: 14, weight: 700 });
  b += k.text(764, 566, 'ours', { size: 14, weight: 700 });
  b += k.text(950, 566, 'or yours', { size: 14, weight: 700 });
  b += k.line(620, 578, 1160, 578, { sw: 3 });
  rows.forEach(([piece, ours, yours, flag], i) => {
    const y = 578 + i * 50;
    b += k.text(620, y + 31, piece, { size: 16, weight: 700 });
    b += k.rect(756, y + 9, 178, 32, { fill: C.wire, sw: 2 });
    b += k.text(766, y + 30, ours, { size: 14 });
    b += k.text(950, y + (flag ? 23 : 31), yours, { size: 16 });
    if (flag) b += k.text(950, y + 41, flag, { size: 12, mono: true });
    b += k.line(620, y + 50, 1160, y + 50, { sw: 1 });
  });
  b += k.text(620, 760, 'Nothing we run holds your group’s things.', { size: 17, weight: 700 });

  // 4 — extend it
  b += k.heading(40, 812, 540, 4, 'Extend it with an app');
  [['an app on a port', 'Jellyfin, a model'], ['a shared folder', 'apps/<kind>/'], ['a game', 'any UDP or TCP port']]
    .forEach(([t, m], i) => {
      const x = 40 + i * 185;
      b += k.rect(x, 832, 170, 70, { fill: C.app });
      b += P(x + 12, 856, t, { w: 150, size: 15, weight: 700 });
      b += k.text(x + 12, 890, m, { size: 11.5, mono: true });
    });
  b += lit(620, 866, 'AGPL. Fork it; that’s what it’s for.', 20);

  b += k.foot(W, H);
  write('03-record.svg', k.svg(W, H, b, 'svrngs: same acts, same state',
    'Each device signs its own acts, linked like git. Admission puts them in one order and refuses by name; every copy folds them to the same state. The code nests from a pure core out to the command. We run the phones’ page, a relay and the builds, and each can be your own. You extend it with an app: one on a port, a folder the group keeps, or a game.'));
}

group();
door();
record();
