# Events

The reference app for the primitive event (svrn-docs
`rings/current/EVENTS.md` §1-§5): an event is a post in a group, and people
answer it, as three acts in the group's record. svrngs's code knows none of
it (`nouns.txt`, `scripts/vocab_check.py`); `apps/` is outside the size
budget (`scripts/lean_check.py`).

- **Shape:** carried, and lent too. The page is carried; `server.mjs` is
  the same app as a lent server.
- **Run it:** bring it from your home page (**Add an app**, Events). Its
  server alone: the last paragraph here.
- **How it's made:** the list below. `events.js` is the fold both shapes share.
- **Change it:** in a group of your own, edit `apps/Events/index.js`
  ([open one up](../../docs/guide/examples.md#2-open-one-up)).

- `events.js`: the fold: `event`, `coming` and `came`, a correction counted
  only from its own person, nothing that records "no", and the host's view
  of several groups (`hosted`).
- `events.test.mjs`, `laws.test.mjs`: what the acts come to, and the laws,
  each caught on a planted fold. Run by `scripts/check.sh`.
- `store.mjs`: the group's record through this computer's svrngs, or
  a file of the app's own for running alone, behind one interface.
- `svrngs.mjs`: how it asks svrngs as you, never sending svrngs's token: a
  copy of `apps/client/svrngs.mjs`, byte for byte, as your own app would
  carry it.
- `server.mjs`: its own small server and page. The group is the path's,
  checked against who stands in it; the person is the header its config
  names, `x-svrngs-key` under svrngs, or what the proxy in front sets.
  Invited is whoever is in the group now, decided by events-run
  (`demo/runs/events/2026-10-14/README.md`). To its host, each group's
  section offers "Copy the link to this event", written from what svrngs's
  door tells it (`x-svrngs-label`, `x-svrngs-pages`; DESIGN §3, "What an
  app gets"), or says why there is none. Running alone, nothing tells it a
  page, and it says so.
- `index.html`, `index.js`: its page when a group carries it (svrngs
  DESIGN §3, "Or a folder the group carries"; v1ui-shelf (b)): static
  files beside `events.js`, reading and writing the group at its own
  origin, `svrngs/acts`, where svrngs writes each answer as the person.
  Its folder in a group is these three files.
- `page.test.mjs`: the page in Chromium and in WebKit as an iPhone, run
  alone on its file. Run by `scripts/check.sh`.
- `canary.mjs`: two computers' copies of one group's record fold the same
  (`crates/svrngs-node/tests/canaries.rs`).

Run alone: `node apps/events/server.mjs config.json`, with
`{"file": "events.json", "who": "remote-user"}` and `events.json` holding
`{"groups": {"house": {"name": "The House", "people": ["sam", "bo"]}}}`.
Under svrngs: `{"record": "<svrngs's data dir, ~/.svrngs>", "who":
"x-svrngs-key"}`, lent with `svrngs share`. It reads no token: svrngs.mjs
asks svrngs as you without ever sending it.
