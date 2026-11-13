// Events' answers, as its page shows them (events-app (b)), with no browser:
// the server's handler (server.mjs `events`) asked in this process on a file
// of its own, each person named by the header a proxy would set, each form
// sent as a browser sends one (its action, its hidden fields and what was
// typed, urlencoded) and its 303 followed. Sam posts a dinner to The House
// and dWeb; Bo is coming with Maya, came, and says Maya came; Ama says she's
// coming and takes it back; Cy, in dWeb, is coming. Each group's people but
// its host are said as invited, Ama, who took hers back, there and nowhere
// else. Each member's page holds only their own group's answers, the
// host's holds both, and a member of one group opening the other's event
// page is refused. Told its label and its groups' pages as svrngs's door
// tells them (events-links (b)), Sam's page offers a link to send in each
// group's chat, or says why there is none; no one else's does. What a
// browser itself does with the page, a button pressed and the Copy
// button's clipboard, is page.test.mjs's.

import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import { test } from 'node:test';

import { events } from './server.mjs';

const WHO = 'remote-user';
/** Words the page never says (DESIGN §1; the queue's fence 6). */
const NOT_SAID = /\b(key|member|record|act|namespace|origin)\b/i;

const decode = (s) =>
  s
    .replace(/&#(\d+);/g, (_, n) => String.fromCharCode(Number(n)))
    .replace(/&quot;/g, '"')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&');
/** What a reader reads of `html`: its words, a line per block, no markup. */
const said = (html) =>
  decode(
    html
      .replace(/<(style|script)[^>]*>[\s\S]*?<\/\1>/g, '')
      .replace(/<\/?(p|h1|h2|h3|li|ul|section|form|label|br)\b[^>]*>/g, '\n')
      .replace(/<[^>]+>/g, ''),
  )
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean)
    .join('\n');

/** The server on a file of its own, holding `ops` ({<group>: [op]}) already written. */
function serve(ops = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'events-answers-'));
  writeFileSync(
    join(dir, 'store.json'),
    JSON.stringify({
      groups: {
        house: { name: 'The House', people: ['sam', 'bo', 'ama'] },
        dweb: { name: 'dWeb', people: ['sam', 'cy'] },
      },
      ops,
    }),
  );
  const app = events({ file: join(dir, 'store.json'), who: WHO });
  /** One request to the handler; its status, where it sends the browser, and its page. */
  return (path, { headers = {}, form } = {}) => {
    const req = Readable.from(form === undefined ? [] : [Buffer.from(form.toString())]);
    req.method = form === undefined ? 'GET' : 'POST';
    req.url = path;
    req.headers = Object.fromEntries(Object.entries(headers).map(([k, v]) => [k.toLowerCase(), v]));
    return new Promise((done, fail) => {
      const res = {
        writeHead(status, head = {}) {
          this.status = status;
          this.location = head.location ?? null;
          return this;
        },
        end(html = '') {
          done({ status: this.status, location: this.location, html: String(html) });
          return this;
        },
      };
      Promise.resolve(app.handle(req, res)).catch(fail);
    });
  };
}

/** Each `<section data-group>` of `html`: its invited, coming and came, as a reader reads them. */
function lists(html) {
  const items = (s, cls) => {
    const ul = s.match(new RegExp(`<ul class="${cls}">([\\s\\S]*?)</ul>`))?.[1] ?? '';
    return [...ul.matchAll(/<li>([\s\S]*?)<\/li>/g)].map((m) => said(m[1]));
  };
  return [...html.matchAll(/<section data-group="([^"]+)">([\s\S]*?)<\/section>/g)].map(
    ([, group, s]) => ({
      group,
      invited: items(s, 'invited'),
      coming: items(s, 'coming'),
      came: items(s, 'came'),
    }),
  );
}

/** A person at the page, as a browser: what they read, and the forms they press. */
function browser(ask) {
  const read = [];
  let at = { html: '', path: '/' };
  const self = {
    read,
    get html() {
      return at.html;
    },
    get path() {
      return at.path;
    },
    async open(person, path, told = {}) {
      self.headers = { [WHO]: person, ...told };
      const res = await ask(path, { headers: self.headers });
      at = { html: res.html, path };
      read.push(said(res.html));
      return res;
    },
    /** The form whose button says `name`, sent with `typed` ({field: value | [values]}), its 303 followed. */
    async press(name, typed = {}) {
      const forms = at.html.split('<form').slice(1);
      const form = forms.find((f) => f.includes(`<button>${name}</button>`));
      assert.ok(form, `no button says ${name} on ${at.path}`);
      const body = new URLSearchParams();
      for (const [, field, value] of form.matchAll(
        /<input type="hidden" name="([^"]+)" value="([^"]*)">/g,
      ))
        body.append(field, decode(value));
      for (const [field, value] of Object.entries(typed))
        for (const one of [value].flat()) body.append(field, one);
      const action = decode(form.match(/action="([^"]+)"/)[1]);
      const sent = await ask(action, { headers: self.headers, form: body });
      assert.equal(sent.status, 303, `${name}: ${said(sent.html)}`);
      const res = await ask(sent.location, { headers: self.headers });
      at = { html: res.html, path: sent.location };
      read.push(said(res.html));
    },
  };
  return self;
}

test("each group's people see only their own group's answers, the host every group's", async () => {
  const page = browser(serve());

  await page.open('sam', '/');
  await page.press('Post', {
    what: 'dinner',
    when: 'Friday 7',
    where: 'the house',
    group: ['house', 'dweb'],
  });
  const house = page.path;
  assert.match(house, /^\/g\/house\/e\/[0-9a-f]{16}$/);
  const dweb = house.replace('/g/house/', '/g/dweb/');

  await page.open('bo', house);
  await page.press("I'm coming", { guests: 'Maya' });
  await page.press('I came');
  await page.press('Maya came');
  await page.open('ama', house);
  await page.press("I'm coming", { guests: '' });
  await page.press('Take my name off');
  await page.open('cy', dweb);
  await page.press("I'm coming", { guests: '' });

  assert.deepEqual(
    lists(page.html),
    [{ group: 'dweb', invited: ['cy'], coming: ['cy'], came: [] }],
    "Cy's page: dWeb's answers alone",
  );
  await page.open('bo', house);
  assert.deepEqual(
    lists(page.html),
    [
      {
        group: 'house',
        invited: ['ama', 'bo'],
        coming: ['bo + Maya'],
        came: ['bo', "Maya, bo's guest"],
      },
    ],
    "Bo's page: The House's answers alone",
  );
  const refused = await page.open('cy', house);
  assert.equal(refused.status, 403);
  assert.match(said(page.html), /You're not in The House/);
  // What svrngs's door tells the app (events-links (a)): its label, and
  // the page of each of Sam's groups, dWeb keeping none.
  await page.open('sam', house, {
    'x-svrngs-label': 'lbl',
    'x-svrngs-pages': 'house=https://group.example/ dweb=none',
  });
  assert.deepEqual(
    lists(page.html),
    [
      {
        group: 'house',
        invited: ['ama', 'bo'],
        coming: ['bo + Maya'],
        came: ['bo', "Maya, bo's guest"],
      },
      { group: 'dweb', invited: ['cy'], coming: ['cy'], came: [] },
    ],
    "Sam's page: both groups' answers",
  );
  const links = [...page.html.matchAll(/<section data-group="([^"]+)">([\s\S]*?)<\/section>/g)].map(
    ([, group, s]) => ({
      group,
      link: s.match(/<input type="text" class="link" readonly value="([^"]*)">/)?.[1] ?? null,
      said: s.match(/<p class="nolink[^"]*">([^<]*)<\/p>/)?.[1] ?? null,
    }),
  );
  assert.deepEqual(
    links.map((l) => ({
      ...l,
      link: l.link && decode(l.link),
      said: l.said && decode(l.said),
    })),
    [
      { group: 'house', link: `https://group.example/#open=lbl&at=${house}`, said: null },
      {
        group: 'dweb',
        link: null,
        said: "dWeb keeps no page for links, so there's no link to send.",
      },
    ],
    "Sam's page: a link to send in each group's chat, or why not",
  );
  assert.match(
    page.html,
    /<input type="text" class="link" readonly value="[^"]*"> <button type="button" onclick="[^"]*">Copy the link to this event<\/button>/,
    'the Copy button beside the link',
  );
  for (const [person, path] of [
    ['bo', house],
    ['cy', dweb],
  ]) {
    await page.open(person, path, {
      'x-svrngs-label': 'lbl',
      'x-svrngs-pages': 'house=https://group.example/',
    });
    assert.doesNotMatch(
      page.html,
      /class="link"|class="nolink/,
      `${person} is not offered the host's link`,
    );
  }
  for (const text of page.read) assert.doesNotMatch(text, NOT_SAID);
});

test("an answer another person's computer wrote says whose", async () => {
  // ralph/FINDING_ON_BEHALF_OF.md's scene: Sam's computer writes Bo's
  // "coming" in Bo's name; Bo's "came" is his own computer's.
  const event = {
    kind: 'event',
    event: '0123456789abcdef',
    what: 'dinner',
    when: 'Friday 7',
    where: 'the house',
  };
  const page = browser(
    serve({
      house: [
        { id: 'a', ts_unix: 1000, person: 'sam', payload: event },
        {
          id: 'b',
          ts_unix: 1100,
          person: 'sam',
          on_behalf_of: 'bo',
          payload: { kind: 'coming', event: event.event, guests: ['Maya'] },
        },
        {
          id: 'c',
          ts_unix: 1200,
          person: 'bo',
          on_behalf_of: 'bo',
          payload: { kind: 'came', event: event.event },
        },
      ],
    }),
  );
  await page.open('bo', `/g/house/e/${event.event}`);
  assert.deepEqual(
    lists(page.html).map(({ coming, came }) => ({ coming, came })),
    [{ coming: ["bo + Maya, through sam's computer"], came: ['bo'] }],
  );
  assert.match(said(page.html), /^sam is hosting\.$/m, "Sam's own post says nothing more");
  assert.doesNotMatch(said(page.html), NOT_SAID);
});
