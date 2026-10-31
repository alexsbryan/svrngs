// guide-use.mjs — the pictures for the guide's using pages (docs/guide/
// start, lend, models, games, tv), as data, in build.mjs's idioms. Each
// picture makes the one point of the section it sits on, with the page's
// own names and button labels. build.mjs calls guideUse(k, draw); they
// land in docs/diagrams/guide/.

export function guideUse(k, draw) {
  const { C } = k;
  // The kit's width is a regular face's; a bold line runs a little wider.
  const lit = (x, y, str, size) =>
    k.rect(x - 4, y - size * 0.86, k.width(str, size) * 1.06 + 8, size * 1.18, { fill: C.lit, stroke: 'none', sw: 0 }) +
    k.text(x, y, str, { size, weight: 700 });
  const owned = (x, y, w, h, name) => k.owned(x, y, w, h, name, Math.round(k.width(name, 17) * 1.08));

  // A button as the pages draw one, yellow when it is the press to keep.
  const btn = (x, y, label, { size = 16, fill = C.lit } = {}) => {
    const w = Math.round(k.width(label, size) * 1.12 + 24);
    return k.rect(x, y, w, 34, { fill, sw: 3 }) +
      k.text(x + w / 2, y + 23, label, { size, weight: 700, anchor: 'middle' });
  };

  // An app on a port, its name in bold; svrngs at work, in the door's heavy rule.
  const app = (x, y, w, h, name, size = 18) => k.rect(x, y, w, h, { fill: C.app }) + k.text(x + 14, y + 30, name, { size, weight: 700 });
  const svrngs = (x, y, w, h, dy = 32) => k.door(x, y, w, h) + k.text(x + 16, y + dy, 'svrngs', { size: 20, weight: 900 });
  const term = (x, y, w, str) => k.rect(x, y, w, 34, { fill: 'whitesmoke', sw: 2 }) +
    k.text(x + 12, y + 23, str, { size: 15, mono: true });

  // The guide's band: smaller than the overview's, signed with its page.
  const band = (W, title, sub, page) =>
    k.rect(24, 24, W - 48, 96, { fill: C.people, sw: 3 }) +
    k.text(48, 72, title, { size: 32, weight: 900 }) +
    k.text(50, 104, sub, { size: 19 }) +
    k.text(W - 48, 52, page, { size: 15, mono: true, anchor: 'end' });


  // ==========================================================================
  // start — one line, one group, one link
  // ==========================================================================
  {
    const W = 1200, H = 640; let b = '';
    // A computer that keeps Lab, and where.
    const keeps = (x, y, w, h) => k.rect(x, y, w, h, { fill: C.record, sw: 2 }) +
      k.text(x + 14, y + h / 2 + 6, 'keeps Lab', { size: 16, weight: 700 }) + k.text(x + 110, y + h / 2 + 6, '~/svrngs/Lab', { size: 15, mono: true });
    b += band(W, 'One line, one group, one link.', 'Sam makes Lab; Bo joins on a computer, Ana on a phone.', 'guide/start');

    b += owned(40, 150, 390, 355, 'Sam’s computer');
    b += k.text(60, 210, 'Paste one line in Terminal.', { size: 16 });
    b += term(60, 222, 350, '$ curl …/install.sh | sh');
    b += k.text(60, 294, 'The home page opens. Make Lab:', { size: 16 });
    b += k.rect(60, 306, 350, 116, { fill: C.paper, sw: 2 });
    b += k.rect(76, 320, 70, 32, { fill: C.record, sw: 2 }) + k.text(111, 342, 'Lab', { size: 16, weight: 700, anchor: 'middle' });
    b += k.rect(156, 320, 70, 32, { fill: C.people, sw: 2 }) + k.text(191, 342, 'Sam', { size: 16, weight: 700, anchor: 'middle' });
    b += btn(76, 372, 'Invite someone to Lab');
    b += keeps(60, 446, 350, 40);

    b += k.text(470, 262, 'The link goes in the chat.', { size: 16, weight: 700 });
    b += k.rect(470, 276, 250, 84, { fill: 'whitesmoke', sw: 2 });
    b += k.text(486, 306, 'sam: come join Lab', { size: 16 });
    b += `<text x="486" y="338" font-family="${k.MONO}" font-size="15" fill="blue" text-decoration="underline">group.svrnme.sh/#…</text>`;
    b += k.arrow(300, 389, 466, 330, { sw: 2.5, head: 11 });
    for (const [y1, y2] of [[300, 246], [340, 404]]) b += k.arrow(720, y1, 776, y2, { sw: 2.5, head: 11 });

    b += owned(780, 150, 380, 160, 'Bo’s computer');
    b += k.text(800, 208, 'One line in Terminal.', { size: 16 });
    b += term(800, 218, 340, '$ curl … \'<the link>\'');
    b += keeps(800, 262, 340, 36);

    b += owned(780, 330, 380, 175, 'Ana’s phone');
    b += k.phone(802, 376, 40);
    b += k.rect(864, 380, 120, 34, { fill: C.paper, sw: 2 }) + k.text(876, 403, 'Ana', { size: 16 });
    b += btn(996, 380, 'Join');
    b += k.text(864, 448, 'Opens Lab. Nothing installed.', { size: 16 });
    b += k.text(864, 476, 'while a computer in Lab is on', { size: 15 });

    b += lit(40, 548, 'Sam’s and Bo’s computers keep Lab. Ana’s phone opens it.', 20);
    draw('guide/start.svg', W, H, b, 'Start: one line, one group, one link',
      'Sam pastes one line in Terminal; the home page opens, Sam makes Lab and presses Invite someone to Lab, and the link goes in the chat. Bo opens it on a computer with one line in Terminal; Ana opens it on a phone, types Ana and presses Join. Sam’s and Bo’s computers keep Lab, each with its folder; Ana’s phone opens it while a computer in Lab is on.');
  }

  // ==========================================================================
  // lend — lend it once; Sam's computer answers only the group
  // ==========================================================================
  {
    const W = 1200, H = 640; let b = '';
    b += band(W, 'Lend it once. Your group opens it.', 'Sam’s computer checks each ask before Jellyfin sees it.', 'guide/lend');

    b += owned(40, 150, 600, 355, 'Sam’s computer');
    b += app(60, 204, 190, 110, 'Jellyfin');
    b += k.text(74, 262, 'localhost:8096', { size: 15, mono: true });
    b += k.door(320, 196, 300, 200);
    b += k.text(338, 226, 'Before it answers', { size: 18, weight: 700 });
    for (const [y, str, ok] of [[262, 'checks who’s asking', true], [306, 'adds the app’s password; it stays here', true], [366, 'keeps refused paths closed']])
      b += (ok ? k.tick(344, y - 5, 7) : k.cross(344, y - 5, 6, C.no)) + k.para(362, y, str, { w: 240, size: 16 });
    b += k.arrow(320, 260, 254, 260, { sw: 3, head: 11 });
    b += btn(60, 420, 'Lend jellyfin to everyone in your groups');
    b += k.text(60, 482, 'or', { size: 16 }) + k.text(84, 482, 'svrngs share 8096', { size: 16, mono: true });

    b += k.rect(622, 296, 88, 28, { fill: C.wire });
    b += k.arrow(708, 310, 624, 310, { sw: 3, head: 11 });
    for (const y of [218, 362, 476]) b += k.line(710, 310, 780, y, { sw: 3 });
    b += k.cross(745, 393, 8, C.no);

    b += owned(780, 150, 380, 120, 'Bo’s computer');
    b += k.rect(800, 200, 180, 36, { fill: C.app, sw: 2 }) + k.text(812, 224, 'Sam’s jellyfin', { size: 16, weight: 700 });
    b += k.text(994, 224, 'on the home page', { size: 15 });
    b += owned(780, 290, 380, 140, 'Ana’s phone');
    b += k.phone(802, 334, 34);
    b += k.rect(856, 346, 180, 36, { fill: C.app, sw: 2 }) + k.text(868, 370, 'Sam’s jellyfin', { size: 16, weight: 700 });
    b += owned(780, 446, 380, 60, 'Not in the group');
    b += k.rect(964, 459, 176, 34, { fill: C.no, sw: 2 });
    b += k.cross(984, 476, 6) + k.text(1000, 482, 'gets nothing', { size: 16, weight: 700 });

    b += lit(40, 548, 'Bo and Ana open it as themselves. No port opened on Sam’s router.', 20);
    draw('guide/lend.svg', W, H, b, 'Lend: lend it once, and your group opens it',
      'Sam lends Jellyfin, on localhost:8096, with Lend jellyfin to everyone in your groups or svrngs share 8096. Bo opens Sam’s jellyfin from the home page and Ana from a phone. Before Jellyfin answers, Sam’s computer checks who is asking, adds the app’s password, which stays on Sam’s computer, and keeps refused paths closed. Someone not in the group gets nothing. No port is opened on Sam’s router.');
  }

  // ==========================================================================
  // models — a program on Bo's computer calls Sam's model
  // ==========================================================================
  {
    const W = 1200, H = 580; let b = '';
    b += band(W, 'Your program calls a friend’s model.', 'Through your svrngs, then Sam’s computer, to llama-server.', 'guide/models');

    b += owned(40, 150, 560, 290, 'Bo’s computer');
    b += k.rect(60, 200, 260, 220, { fill: C.app });
    b += k.text(74, 228, 'a program', { size: 18, weight: 700 });
    b += k.text(74, 252, 'chat app, editor, script', { size: 15 });
    b += k.text(74, 292, 'base URL', { size: 15, weight: 700 });
    b += k.text(74, 314, 'http://….localhost:…/v1', { size: 15, mono: true });
    b += k.text(74, 352, 'API key', { size: 15, weight: 700 });
    b += k.rect(70, 362, 180, 32, { fill: 'whitesmoke', sw: 2 }) + k.text(80, 384, '~/.svrngs/token', { size: 15, mono: true });
    b += k.arrow(320, 300, 378, 300, { sw: 3, head: 11 });
    b += svrngs(380, 200, 200, 220);
    b += k.tick(402, 271, 7) + k.text(420, 276, 'token checked', { size: 16 });
    b += lit(396, 330, 'token stops here', 17);
    b += k.text(396, 392, 'asks as Bo', { size: 16 });

    b += owned(680, 150, 480, 290, 'Sam’s computer');
    b += k.rect(600, 296, 80, 28, { fill: C.wire });
    b += k.arrow(582, 310, 718, 310, { sw: 3, head: 11 });
    b += svrngs(720, 200, 200, 220);
    b += k.tick(742, 271, 7) + k.para(760, 276, 'checks it’s Bo', { w: 150, size: 16 });
    b += k.tick(742, 315, 7) + k.para(760, 320, 'adds Sam’s own key, if any', { w: 150, size: 16 });
    b += k.arrow(920, 270, 958, 270, { sw: 3, head: 11 });
    b += app(960, 200, 180, 140, 'llama-server');
    b += k.text(974, 256, 'Sam’s coder', { size: 15 });
    b += k.text(974, 282, ':8101', { size: 15, mono: true });

    b += lit(40, 488, 'Your token stops at your computer. Sam never receives it.', 20);
    draw('guide/models.svg', W, H, b, 'A program on your computer calls a friend’s model',
      'A program on Bo’s computer calls the address Bo copied, a name ending in .localhost, with the token from ~/.svrngs/token as its API key. Bo’s svrngs checks the token and asks Sam’s computer as Bo; the token goes no further. Sam’s computer checks it is Bo, adds Sam’s own key if llama-server has one, and passes the request to llama-server.');
  }

  // ==========================================================================
  // models — the pantry: every friend's model behind one address
  // ==========================================================================
  {
    const W = 1200, H = 580; let b = '';
    b += band(W, 'Every friend’s model, one address.', 'The pantry sends each chat to the friend whose model it names.', 'guide/models');

    b += owned(40, 150, 520, 290, 'Your computer');
    b += app(60, 200, 210, 110, 'chat app');
    b += k.text(74, 258, 'model:', { size: 15 });
    b += k.text(74, 282, 'Ana/coder', { size: 15, mono: true });
    b += k.arrow(270, 255, 328, 255, { sw: 3, head: 11 });
    b += k.door(330, 200, 210, 110);
    b += k.text(346, 228, 'the pantry', { size: 18, weight: 700 });
    b += k.text(346, 256, '127.0.0.1:8787/v1', { size: 15, mono: true });
    b += k.text(346, 290, 'adds your token', { size: 15 });
    b += k.text(60, 380, 'Tools: each at its own address,', { size: 15 });
    b += k.text(60, 402, 'from', { size: 15 }) + k.text(98, 402, 'pantry.mjs list', { size: 15, mono: true });
    b += k.text(240, 402, '+', { size: 15 }) + k.text(256, 402, '/mcp', { size: 15, mono: true });

    for (const y of [190, 290]) b += k.line(540, 255, 760, y, { sw: 3 });
    b += k.line(320, 390, 760, 390, { sw: 3, dash: '8 6' });

    const rows = [[150, 'Ana’s computer', 'Ana/coder', 'model'], [250, 'Sam’s computer', 'Sam/voice', 'model'], [350, 'Bo’s computer', 'tools', 'MCP']];
    for (const [y, name, id, what] of rows) {
      b += owned(760, y, 400, 80, name);
      b += k.rect(960, y + 22, 180, 38, { fill: C.app, sw: 2 });
      b += k.text(974, y + 47, id, { size: 16, mono: true, weight: 700 });
      b += k.text(1128, y + 47, what, { size: 15, anchor: 'end' });
    }

    b += lit(40, 488, 'Pick Ana/coder, and your chat goes to Ana’s computer.', 20);
    draw('guide/pantry.svg', W, H, b, 'The pantry: every friend’s model behind one address',
      'On your computer, a chat app points at the pantry, at 127.0.0.1:8787/v1, which adds your svrngs token. Its model list shows Ana/coder and Sam/voice; picking Ana/coder sends the chat to Ana’s computer. A friend’s tools, such as Bo’s, are reached at their own address from pantry.mjs list with /mcp on the end.');
  }

  // ==========================================================================
  // games — Play: the game connects to your own computer
  // ==========================================================================
  {
    const W = 1200, H = 580; let b = '';
    b += band(W, 'Play: your game connects to your computer.', 'svrngs carries it to Sam’s server.', 'guide/games');

    b += owned(40, 150, 600, 290, 'Ana’s computer');
    b += k.rect(60, 200, 560, 80, { fill: C.paper, sw: 2 });
    b += k.text(76, 228, 'Sam’s minecraft', { size: 17, weight: 700 });
    b += k.text(76, 262, 'Connect to 127.0.0.1:41821. Copied.', { size: 15 });
    b += btn(536, 210, 'Play');
    b += app(60, 310, 220, 110, 'Minecraft');
    b += k.text(74, 366, 'Multiplayer,', { size: 15 });
    b += k.text(74, 388, 'Add Server', { size: 15 });
    b += k.arrow(280, 365, 358, 365, { sw: 3, head: 11 });
    b += svrngs(360, 310, 260, 110, 30);
    b += k.lit(376, 368, '127.0.0.1:41821', 15, true);
    b += k.text(376, 396, 'on Ana’s own computer', { size: 15 });

    b += owned(700, 150, 460, 290, 'Sam’s computer');
    b += k.rect(640, 351, 60, 28, { fill: C.wire });
    b += k.arrow(620, 365, 738, 365, { sw: 3, head: 11 });
    b += k.text(720, 216, 'svrngs share 25565 --tcp', { size: 15, mono: true });
    b += k.text(720, 242, 'lends the port as it is', { size: 15 });
    b += svrngs(740, 310, 170, 110, 30);
    b += k.text(756, 368, 'checks it’s Ana', { size: 15 });
    b += k.arrow(910, 365, 938, 365, { sw: 3, head: 11 });
    b += app(940, 310, 200, 110, 'Minecraft server', 17);
    b += k.text(954, 368, ':25565', { size: 15, mono: true });

    b += lit(40, 488, 'The game sees only an address on Ana’s own computer.', 20);
    draw('guide/play.svg', W, H, b, 'Play: your game connects to your own computer',
      'Ana presses Play on Sam’s minecraft, and the row says Connect to 127.0.0.1:41821. Copied. Ana’s Minecraft connects to that address on Ana’s own computer; svrngs carries it to Sam’s computer, which lent the port with svrngs share 25565 --tcp, and on to Sam’s Minecraft server on 25565.');
  }

  // ==========================================================================
  // games — Give over time: the game outlives any one computer
  // ==========================================================================
  {
    const W = 1200, H = 620; let b = '';
    b += band(W, 'Give: the group keeps the game.', 'One computer runs it; every computer holds its copies.', 'guide/games');

    const PW = 210, at = (i) => 40 + i * 226, top = 150, bot = 478;
    // A computer with its name under it.
    const scr = (x, y, w, name, o) => k.computer(x, y, w, o) + k.text(x + w / 2, y + Math.round(w * 0.6) + 28, name, { size: 15, anchor: 'middle' });
    const keep = (x) => btn(x + 8, 330, 'Agree and keep it here', { size: 15 });
    // Each panel: its title, its caption, and what it shows.
    const panels = [
      ['Kept by Cy’s computer', 'A copy every 5 minutes, into the group’s folder on every computer.', (x) => {
        let s = scr(x + 16, 240, 80, 'Cy', { fill: C.app });
        for (const [yy, name] of [[218, 'Bo'], [290, 'Dee']]) {
          s += k.arrow(x + 100, 262, x + 140, yy + 16, { sw: 2, head: 9, dash: '5 4' });
          s += k.rect(x + 144, yy, 50, 30, { fill: C.paper, sw: 2 });
          s += k.text(x + 169, yy + 21, name, { size: 15, anchor: 'middle' });
        }
        return s + k.text(x + 144, 348, 'copies', { size: 15 });
      }],
      ['Move it to… Bo', 'Nothing moves until Bo says yes.', (x) =>
        scr(x + 16, 236, 64, 'Cy') + k.arrow(x + 88, 255, x + 124, 255, { sw: 2.5, head: 10 }) +
        scr(x + 130, 236, 64, 'Bo', { fill: C.app }) + keep(x)],
      ['Bo’s computer goes quiet', '15 minutes with no copy. Every page says when it last answered.', (x) =>
        scr(x + 65, 236, 80, 'Bo', { dash: '7 5' }) +
        k.text(x + 105, 272, '15 min', { size: 15, weight: 700, anchor: 'middle' }) +
        k.text(x + 105, 352, 'no copy', { size: 15, anchor: 'middle' })],
      ['Keep it here?', 'Dee’s computer starts the newest copy. At most about 5 minutes lost.', (x) =>
        k.rect(x + 12, 240, 74, 30, { fill: C.paper, sw: 2 }) + k.text(x + 49, 261, '14:05', { size: 15, anchor: 'middle' }) +
        k.arrow(x + 90, 255, x + 124, 255, { sw: 2.5, head: 10 }) +
        scr(x + 130, 236, 64, 'Dee', { fill: C.app }) + keep(x)],
      ['Bo’s computer comes back', 'Its copy is listed. Nothing merged, nothing thrown away.', (x) =>
        k.rect(x + 12, 226, PW - 24, 76, { fill: 'whitesmoke', sw: 2 }) +
        k.para(x + 22, 250, 'Minecraft as Bo’s computer had it at 14:10', { w: PW - 44, size: 15 }) +
        btn(x + 12, 330, 'Start instead', { size: 15, fill: C.paper })],
    ];
    panels.forEach(([title, cap], i) => {
      const x = at(i);
      b += k.rect(x, top, PW, bot - top, { fill: C.paper, sw: 2 });
      b += k.text(x + 12, top + 30, `${i + 1}`, { size: 18, weight: 900 });
      b += k.para(x + 32, top + 30, title, { w: PW - 70, size: 17, weight: 700 });
      b += k.para(x + 12, 410, cap, { w: PW - 24, size: 15 });
    });
    for (const [i, [, , show]] of panels.entries()) b += show(at(i));
    b += k.arrow(40, 500, 1154, 500, { sw: 2, head: 11 });
    b += k.text(1140, 488, 'time', { size: 15, anchor: 'end' });

    b += lit(40, 540, 'The game doesn’t depend on one computer being on.', 20);
    draw('guide/give.svg', W, H, b, 'Give: the group keeps the game',
      'Five panels in time. Cy’s computer keeps Minecraft and copies it every 5 minutes into the group’s folder on every computer. Cy moves it to Bo, and nothing moves until Bo presses Agree and keep it here. Bo’s computer goes 15 minutes with no copy. Dee is offered the newest copy, from 14:05, and Dee’s computer starts it. Bo’s computer comes back and its copy is listed, as Bo’s computer had it at 14:10, with Start instead.');
  }

  // ==========================================================================
  // tv — the room: anything on this WiFi opens it as Cy
  // ==========================================================================
  {
    const W = 1200, H = 600; let b = '';
    b += band(W, 'The room: your WiFi opens it as you.', 'A TV, a phone, a laptop: through Cy’s computer, to Sam’s Jellyfin.', 'guide/tv');

    b += k.rect(40, 150, 740, 320, { fill: C.paper, dash: '10 6' });
    b += k.rect(40, 150, 140, 34, { fill: C.wire }) + k.text(53, 173, 'home WiFi', { size: 17, weight: 700 });

    // A TV: a wide screen on a short stand.
    b += k.rect(70, 206, 110, 62, { fill: C.paper }) + k.line(125, 268, 125, 280, { sw: 4 }) + k.line(100, 280, 150, 280, { sw: 4 });
    b += k.text(196, 244, 'TV', { size: 15 });
    b += k.computer(84, 312, 80);
    b += k.text(196, 342, 'laptop', { size: 15 });
    b += k.phone(110, 384, 28);
    b += k.text(196, 422, 'phone', { size: 15 });
    for (const [y1, y2] of [[240, 300], [338, 330], [418, 360]]) b += k.arrow(250, y1, 346, y2, { sw: 2.5, head: 11 });

    b += owned(350, 200, 410, 250, 'Cy’s computer');
    b += btn(370, 250, 'Open it to this WiFi');
    b += k.text(370, 320, 'Type into the TV’s app:', { size: 16 });
    b += k.lit(370, 350, 'http://192.168.1.4:48719/', 17, true);
    b += k.text(370, 400, 'Sam’s jellyfin, opened as Cy', { size: 16 });

    b += owned(840, 150, 320, 180, 'Sam’s computer');
    b += k.rect(760, 276, 80, 28, { fill: C.wire });
    b += k.arrow(762, 290, 858, 290, { sw: 3, head: 11 });
    b += app(860, 210, 200, 100, 'Jellyfin');
    b += k.text(874, 268, ':8096', { size: 15, mono: true });

    b += k.rect(840, 360, 320, 110, { fill: C.paper, dash: '10 6' });
    b += k.rect(840, 360, 270, 34, { fill: C.wire }) + k.text(853, 383, 'Cy’s computer, another WiFi', { size: 17, weight: 700 });
    b += k.computer(864, 410, 56);
    b += k.rect(944, 410, 196, 36, { fill: C.no, sw: 2 });
    b += k.cross(962, 428, 6) + k.text(978, 434, 'closed for now', { size: 16, weight: 700 });

    b += lit(40, 520, 'Anything on this WiFi opens it as Cy.', 20);
    draw('guide/tv.svg', W, H, b, 'The room: your WiFi opens a lend as you',
      'Cy presses Open it to this WiFi. A TV, a laptop and a phone on the home WiFi type http://192.168.1.4:48719/ and reach Cy’s computer, which opens Sam’s Jellyfin for them as Cy. When Cy’s computer is on another WiFi, it is closed for now.');
  }
}
