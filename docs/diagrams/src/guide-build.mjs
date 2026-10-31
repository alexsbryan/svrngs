// guide-build.mjs — the pictures for the guide's people, folders, apps and
// build pages (docs/guide/), as data in the kit's look and its one legend.
// Each has one point and few words; each says what its page says, with the
// page's names and buttons. build.mjs calls guideBuild(k, draw).

export function guideBuild(k, draw) {
  const { C } = k;
  const T = (x, y, str, o) => k.text(x, y, str, o);
  const M = (x, y, str, o = {}) => k.text(x, y, str, { size: 15, mono: true, ...o });
  // The picture's name, in a band of colour shorter than the overview's.
  const top = (W, title) => k.rect(24, 24, W - 48, 64, { fill: C.people }) + T(46, 68, title, { size: 28, weight: 900 });
  // A button, as the pages draw one: yellow, its words bold.
  const button = (x, y, str, size = 16) => {
    const w = Math.round(k.width(str, size) * 1.1) + 24;
    return { s: k.rect(x, y, w, size + 18, { fill: C.lit, sw: 3 }) + T(x + 12, y + size + 5, str, { size, weight: 700 }), w };
  };
  // A computer that may be off: an off one is dashed, its base too.
  const pc = (x, y, w, name, on = true) => {
    const h = Math.round(w * 0.62), dash = on ? null : '7 5';
    return k.rect(x, y, w, h, { fill: C.paper, dash }) +
      k.line(x - w * 0.1, y + h + 6, x + w * 1.1, y + h + 6, { sw: 4, dash }) +
      T(x + w / 2, y + h + 30, name, { size: 16, weight: 700, anchor: 'middle' }) +
      (on ? '' : T(x + w / 2, y + h + 50, 'off', { size: 15, anchor: 'middle' }));
  };

  // ==========================================================================
  // people.md — three links, three ways in
  // ==========================================================================
  {
    const W = 1100, H = 560; let b = '';
    b += top(W, 'Three links, each for someone different.');
    const cols = [
      { btn: 'Invite someone to House', who: 'a new person', sub: 'joins House as themselves',
        can: [[true, 'the group’s record', C.record], [true, 'what everyone lends', C.app]],
        last: 'The link works for a day.' },
      { btn: 'Add your phone or another computer', who: 'you, again', sub: 'joins as you, no name asked',
        can: [[true, 'the group’s record', C.record], [true, 'what everyone lends', C.app]],
        last: 'Send it to nobody else.' },
      { btn: 'Have a guest', who: 'a guest', sub: 'Dee, Cy’s guest',
        can: [[true, 'what Cy lends', C.app], [false, 'the group’s record', C.record]],
        last: 'For 48, 24 or 6 hours.' },
    ];
    cols.forEach((c, i) => {
      const x = 40 + i * 345, w = 330;
      b += button(x, 116, c.btn, 16).s;
      b += k.arrow(x + 40, 154, x + 40, 190, { sw: 3, head: 11 });
      b += T(x + 56, 180, 'lets in', { size: 15 });
      b += k.rect(x, 196, w, 72, { fill: C.people });
      b += T(x + 14, 226, c.who, { size: 22, weight: 900 });
      b += T(x + 14, 254, c.sub, { size: 16 });
      b += T(x, 306, 'can open', { size: 15, weight: 700 });
      c.can.forEach(([ok, str, fill], j) => {
        const y = 318 + j * 46;
        b += ok ? k.tick(x + 12, y + 17, 8) : k.cross(x + 12, y + 17, 7, C.no);
        b += k.rect(x + 32, y, w - 32, 34, { fill: ok ? fill : C.paper, sw: 2 }) + T(x + 44, y + 23, str, { size: 16 });
        if (!ok) b += k.line(x + 40, y + 17, x + 40 + k.width(str, 16) + 10, y + 17, { stroke: C.no, sw: 3 });
      });
      b += k.line(x, 424, x + w, 424, { sw: 1 });
      b += T(x, 452, c.last, { size: 17, weight: 700 });
    });
    draw('guide/people-links.svg', W, H, b, 'Three links, each for someone different',
      'Invite someone to House lets in a new person, who can open the group’s record and what everyone lends; the link works for a day. Add your phone or another computer lets you in again, as you. Have a guest lets in a guest for 48, 24 or 6 hours, who opens only what Cy lends and never the group’s record.');
  }

  // ==========================================================================
  // people.md — removing someone
  // ==========================================================================
  {
    const W = 1100, H = 540; let b = '';
    b += top(W, 'One person decides who is removed.');
    // Bo asks; Sam decides.
    b += k.owned(40, 120, 200, 110, 'Bo');
    b += k.para(56, 182, 'can’t remove Dee, so asks Sam', { w: 170, size: 16 });
    b += k.arrow(242, 175, 314, 175, { sw: 3, head: 12 });
    b += T(252, 165, 'asks', { size: 15 });
    b += k.owned(318, 120, 322, 110, 'Sam');
    b += T(334, 182, 'made House, so decides:', { size: 16 });
    b += M(334, 212, 'svrngs remove Dee --group House', { size: 15 });
    b += k.para(40, 278, 'Everyone sees Bo’s ask. Dee stays in until Sam removes Dee.', { w: 580, size: 16 });
    b += k.lit(40, 340, 'Dee is out on every device.', 19);
    b += k.arrow(640, 175, 700, 175, { sw: 3, head: 12 });

    // Dee, out on every device; what Dee has stays.
    b += k.owned(704, 120, 356, 330, 'Dee');
    b += k.computer(730, 180, 150);
    b += k.rect(744, 194, 122, 62, { fill: C.record, sw: 2 });
    b += k.text(805, 220, 'what Dee', { size: 15, anchor: 'middle' });
    b += k.text(805, 242, 'already has', { size: 15, anchor: 'middle' });
    b += k.phone(960, 172, 52);
    b += k.cross(905, 220, 13, C.no) + k.cross(1036, 220, 13, C.no);
    b += k.para(724, 340, 'Out of House on the computer and the phone. Nothing new reaches Dee.', { w: 320, size: 16 });
    b += k.para(724, 404, 'What Dee already has stays.', { w: 320, size: 16, weight: 700 });
    draw('guide/people-remove.svg', W, H, b, 'One person decides who is removed',
      'Bo can’t remove Dee, so Bo’s ask goes to Sam, who made House and decides. Sam runs svrngs remove Dee --group House. Dee is out of House on every computer and phone; what Dee already has stays, and nothing new reaches Dee.');
  }

  // ==========================================================================
  // folders.md — folders match by their last name
  // ==========================================================================
  {
    const W = 1100, H = 560; let b = '';
    b += top(W, 'Folders are matched by their last name.');
    const at = [[40, 'Ana', '~/Music/', 'rehearsals'], [400, 'Bo', '~/Band/', 'rehearsals'], [760, 'Cy', '~/Band/', 'rehearsals']];
    for (const [x, name] of at) b += k.owned(x, 112, 300, 270, `${name}’s computer`);
    // The wire that makes the three one, drawn under the folders.
    b += k.rect(300, 196, 520, 16, { fill: C.wire, sw: 2 });
    b += k.rect(660, 196, 120, 16, { fill: C.wire, sw: 2 });
    for (const [x, , dir, last] of at) {
      b += k.folder(x + 22, 168, 256, 72, { fill: C.paper });
      const full = dir + last, sx = x + 36;
      b += k.mark(sx + k.width(dir, 16, true), 222, last, 16, { mono: true });
      b += M(sx, 222, full, { size: 16 });
    }
    b += T(570, 260, 'one folder', { size: 16, weight: 700, anchor: 'middle' });
    // Bo's other folder, apart.
    b += k.folder(422, 286, 256, 66, { fill: C.paper, dash: '7 5' });
    b += M(436, 334, '~/Band/recordings', { size: 16 });
    b += T(550, 372, 'a different name: Bo’s own', { size: 15, anchor: 'middle' });
    b += k.lit(40, 428, 'The same last name makes them one folder.', 19);
    b += M(40, 466, 'svrngs share --folder ~/Band/rehearsals --group Band', { size: 15 });
    draw('guide/folders-match.svg', W, H, b, 'Folders are matched by their last name',
      'Ana’s ~/Music/rehearsals and Bo’s and Cy’s ~/Band/rehearsals share the last name rehearsals, so the three are one folder. Bo’s ~/Band/recordings has a different name and stays Bo’s own.');
  }

  // ==========================================================================
  // folders.md — two edits of one file
  // ==========================================================================
  {
    const W = 1100, H = 540; let b = '';
    b += top(W, 'Two edits at once: both are kept.');
    const file = (x, y, w, name, fill = C.paper) => k.rect(x, y, w, 40, { fill, sw: 2 }) + M(x + 12, y + 26, name, { size: 16 });
    for (const [y, name, did] of [[116, 'Bo', 'edits it'], [256, 'Cy', 'edits it too']])
      b += k.owned(40, y, 330, 116, name) + file(58, y + 52, 160, 'setlist.txt') + T(232, y + 78, did, { size: 16 });
    b += T(40, 404, 'before either sees the other’s', { size: 16 });
    b += k.arrow(372, 188, 470, 240, { sw: 3, head: 12 });
    b += k.arrow(372, 330, 470, 278, { sw: 3, head: 12 });

    b += k.owned(476, 116, 584, 288, 'Ana’s, Bo’s and Cy’s computers');
    [528, 638, 748].forEach((x) => { b += k.computer(x, 168, 70); });
    for (const [y, name, whose] of [[248, 'setlist.txt', 'Bo’s edit'], [300, 'setlist (conflict Cy).txt', 'Cy’s, named for Cy']])
      b += file(500, y, 300, name) + T(816, y + 26, whose, { size: 16 });
    b += T(500, 382, 'Every computer ends with both.', { size: 16 });
    b += k.lit(40, 448, 'Neither edit is lost. Open both, merge by hand, delete one.', 19);
    draw('guide/folders-conflict.svg', W, H, b, 'Two edits at once: both are kept',
      'Bo and Cy both edit setlist.txt before either sees the other’s change. Every computer ends with setlist.txt and setlist (conflict Cy).txt, named after whoever wrote it. Open both, merge by hand, and delete the one you don’t need.');
  }

  // ==========================================================================
  // apps.md — a brought app is carried by the group
  // ==========================================================================
  {
    const W = 1100, H = 560; let b = '';
    b += top(W, 'A brought app opens while any computer is on.');
    // 1 — lent
    b += k.heading(40, 150, 470, 1, 'Lent: goes with its lender');
    b += T(40, 184, 'an app Ana lends, on Ana’s computer', { size: 16 });
    b += pc(60, 206, 170, 'Ana', false);
    b += k.rect(82, 222, 126, 72, { fill: C.app, sw: 2 });
    b += T(145, 264, 'Ana’s app', { size: 16, anchor: 'middle' });
    b += k.phone(420, 206, 48);
    b += T(444, 326, 'Kim', { size: 16, weight: 700, anchor: 'middle' });
    b += k.arrow(412, 260, 252, 260, { sw: 3, head: 12, dash: '8 6' });
    b += k.cross(332, 260, 11, C.no);
    b += T(40, 410, 'Opens only while Ana’s computer is on.', { size: 17 });

    // 2 — brought
    b += k.heading(570, 150, 490, 2, 'Brought: carried by the group');
    b += T(570, 184, 'Money, in Elm’s folder on every computer in Elm', { size: 16 });
    [[580, 'Ana', false], [740, 'Bo', true], [900, 'Cy', true]].forEach(([x, name, on]) => {
      b += pc(x, 206, 140, name, on);
      b += k.folder(x + 14, 216, 112, 64, { fill: C.paper, sw: 2 });
      b += k.rect(x + 24, 244, 92, 28, { fill: C.app, sw: 2 });
      b += T(x + 70, 264, 'Money', { size: 15, anchor: 'middle' });
    });
    b += k.phone(1010, 360, 30);
    b += T(1000, 432, 'Kim', { size: 16, weight: 700, anchor: 'end' });
    b += k.arrow(1004, 366, 856, 302, { sw: 3, head: 12 });
    b += k.tick(918, 352, 9);
    b += k.lit(570, 410, 'Opens while any of them is on.', 19);
    draw('guide/apps-carried.svg', W, H, b, 'A brought app opens while any computer is on',
      'An app Ana lends opens only while Ana’s computer is on. Money, brought to Elm, lives in Elm’s folder on every computer in Elm, so with Ana’s computer off Kim still opens it from Bo’s.');
  }

  // ==========================================================================
  // apps.md — Money: nobody owes until they say yes
  // ==========================================================================
  {
    const W = 1100, H = 560; let b = '';
    b += top(W, 'Nobody owes until they say yes.');
    // 1 — Kim adds what Kim paid
    b += k.heading(40, 150, 340, 1, 'Kim adds what Kim paid');
    b += k.owned(40, 172, 340, 250, 'Kim’s Money');
    const field = (y, label, value) => T(58, y, label, { size: 15 }) + k.rect(220, y - 22, 140, 30, { fill: C.paper, sw: 2 }) + M(230, y, value, { size: 16 });
    b += field(236, 'What it was for', 'pizza');
    b += field(276, 'How much', '54');
    b += T(58, 316, 'Who was in on it?', { size: 15 });
    for (const [x, name] of [[220, 'Ana'], [296, 'Bo']]) b += k.rect(x, 300, 18, 18, { sw: 2 }) + k.tick(x + 9, 309, 6) + T(x + 26, 316, name, { size: 15 });
    b += button(58, 356, 'Add it, paid by you', 16).s;
    b += k.arrow(384, 300, 430, 300, { sw: 3, head: 12 });

    // 2 — each says so themselves
    b += k.heading(436, 150, 624, 2, 'Each says so themselves');
    b += k.owned(436, 172, 624, 92, 'Bo');
    b += k.tick(462, 236, 8);
    b += T(480, 242, 'Bo said yes: Bo’s share counts.', { size: 16 });
    b += k.owned(436, 280, 624, 142, 'Ana’s Money');
    b += T(454, 340, 'Kim says you were in on pizza: $18.00.', { size: 17, weight: 700 });
    const yes = button(454, 362, 'Yes, I was in on pizza', 16);
    b += yes.s + button(454 + yes.w + 14, 362, 'Not me', 16).s.replace(C.lit, C.paper);
    b += k.lit(40, 470, 'Until Ana says yes, Ana owes nothing. Kim names people, but never signs for them.', 18);
    draw('guide/apps-money.svg', W, H, b, 'Money: nobody owes until they say yes',
      'Kim adds pizza, $54, and ticks Ana and Bo. Bo says yes, so Bo’s share counts. Ana’s Money asks: Kim says you were in on pizza: $18.00, with Yes, I was in on pizza and Not me. Until Ana says yes, Ana owes nothing: Kim names people but never signs for them.');
  }

  // ==========================================================================
  // build.md — what your app gets: who's asking, and a record
  // ==========================================================================
  {
    const W = 1200, H = 620; let b = '';
    b += top(W, 'What your app gets: who’s asking, verified.');
    // Bo's computer
    b += k.owned(40, 116, 290, 396, 'Bo’s computer');
    b += k.rect(58, 166, 254, 92, { fill: 'whitesmoke', sw: 2 });
    b += T(72, 196, 'Ana’s hello', { size: 16, weight: 700 });
    b += T(72, 236, 'Hello, Bo', { size: 22, weight: 900 });
    b += k.rect(58, 380, 254, 112, { fill: C.record, sw: 2 });
    b += k.para(72, 408, 'signs it as Bo, into the group’s record', { w: 230, size: 16 });
    b += M(72, 474, 'hello/visit', { size: 15 });

    // The wire, the request on it.

    // Ana's computer: the door, then the app.
    b += k.owned(422, 116, 738, 260, 'Ana’s computer');
    b += k.rect(330, 200, 92, 24, { fill: C.wire, sw: 2 });
    b += k.arrow(312, 212, 446, 212, { sw: 3, head: 12 });
    b += k.door(450, 166, 392, 192);
    b += k.cross(470, 194, 7, C.no);
    b += T(486, 200, 'strips any', { size: 16 });
    b += M(486 + k.width('strips any ', 16), 200, 'x-svrngs-*', { size: 16 });
    b += T(486 + k.width('strips any ', 16) + k.width('x-svrngs-* ', 16, true), 200, 'sent', { size: 16 });
    b += k.tick(470, 230, 7);
    b += T(486, 236, 'sets', { size: 16 });
    const hdr = [['x-svrngs-key: 7d2e…', 'key on this'], ['x-svrngs-person: Bo', 'show this'], ['x-svrngs-user: Bo-7d2e…', '']];
    hdr.forEach(([h, note], i) => {
      const y = 272 + i * 30;
      if (i === 0) b += k.mark(468, y, h, 16, { mono: true });
      b += M(468, y, h, { size: 16 });
      if (note) b += T(828, y, note, { size: 15, anchor: 'end', weight: 700 });
    });
    b += k.arrow(844, 262, 900, 262, { sw: 3, head: 12 });
    b += k.rect(904, 196, 236, 132, { fill: C.app });
    b += T(920, 228, 'your app', { size: 18, weight: 700 });
    b += M(920, 262, 'node hello.mjs', { size: 16 });
    b += M(920, 290, '127.0.0.1:4400', { size: 16 });

    // The answer, back, with what it writes.
    b += k.line(1022, 330, 1022, 430, { sw: 3 });
    b += k.arrow(1022, 430, 316, 430, { sw: 3, head: 12 });
    b += T(350, 414, 'Your answer may name entries:', { size: 16 });
    b += M(350, 458, 'x-svrngs-append: [{"group": …, "op": {"kind": "visit"}}]', { size: 15 });
    b += T(350, 488, 'Bo’s computer signs them; the browser never sees the header.', { size: 16 });
    b += k.lit(40, 538, 'Key on x-svrngs-key. Show x-svrngs-person.', 18);
    draw('guide/build-headers.svg', W, H, b, 'What your app gets: who is asking, verified',
      'Bo opens Ana’s hello. The request reaches Ana’s computer, which strips any x-svrngs- header sent and sets x-svrngs-key, x-svrngs-person and x-svrngs-user, then passes it to the app on 127.0.0.1:4400. Entries the answer names in x-svrngs-append are signed by Bo’s own computer into the group’s record, and the browser never sees the header. Key on x-svrngs-key; show x-svrngs-person.');
  }

  // ==========================================================================
  // build.md — the two shapes
  // ==========================================================================
  {
    const W = 1100, H = 600; let b = '';
    b += top(W, 'Two shapes: a lent server, or a carried folder.');
    // 1 — a lent server
    b += k.heading(40, 150, 470, 1, 'Lent: a server on a port');
    b += k.owned(40, 172, 470, 200, 'Ana’s computer');
    b += k.rect(60, 222, 250, 76, { fill: C.app });
    b += M(76, 252, 'node hello.mjs', { size: 16 });
    b += M(76, 280, '127.0.0.1:4400', { size: 16 });
    b += M(60, 338, 'svrngs share 4400 --kind hello', { size: 15 });
    b += k.para(40, 450, 'Goes with Ana: opens while Ana’s computer is on.', { w: 470, size: 17 });

    // 2 — a carried folder
    b += k.heading(570, 150, 490, 2, 'Carried: a folder the group keeps');
    b += k.folder(570, 172, 490, 132, { fill: C.paper });
    b += M(588, 212, '~/svrngs/Elm/apps/guestbook/', { size: 16 });
    b += k.rect(588, 228, 160, 30, { fill: C.app, sw: 2 }) + M(598, 249, 'index.html', { size: 15 });
    b += k.rect(758, 228, 130, 30, { fill: C.app, sw: 2 }) + M(768, 249, 'index.js', { size: 15 });
    b += k.rect(588, 266, 160, 30, { fill: C.record, sw: 2 }) + M(598, 287, 'svrngs/acts', { size: 15 });
    b += T(760, 287, 'reads and writes the record', { size: 15 });
    [[590, 'Ana'], [740, 'Bo'], [890, 'Cy']].forEach(([x, name]) => {
      b += k.line(x + 55, 304, x + 55, 330, { sw: 2 });
      b += k.computer(x + 10, 330, 90);
      b += T(x + 55, 416, name, { size: 16, weight: 700, anchor: 'middle' });
    });
    b += T(570, 450, 'Every computer in Elm serves it.', { size: 17 });
    b += k.lit(570, 488, 'Opens while any of them is on.', 19);
    draw('guide/build-shapes.svg', W, H, b, 'Two shapes: a lent server, or a carried folder',
      'A lent server runs on Ana’s computer at 127.0.0.1:4400, lent with svrngs share 4400 --kind hello; it goes with Ana and opens while Ana’s computer is on. A carried app is a folder apps/guestbook/ with index.html and index.js in Elm’s folder, reading and writing the record at svrngs/acts; every computer in Elm serves it, so it opens while any of them is on.');
  }
}
