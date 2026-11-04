# Events

The reference app for the primitive event (svrn-docs
`rings/current/EVENTS.md` §1-§5): an event is a post in a group, and people
answer it, as three acts in the group's record. svrngs's code knows none of
it (`nouns.txt`, `scripts/vocab_check.py`); `apps/` is outside the size
budget (`scripts/lean_check.py`).

- `events.js`: the fold: `event`, `coming` and `came`, a correction counted
  only from its own person, nothing that records "no", and the host's view
  of several groups (`hosted`).
- `events.test.mjs`, `laws.test.mjs`: what the acts come to, and the laws,
  each caught on a planted fold. Run by `scripts/check.sh`.
- `store.mjs`: the group's record through this computer's loopback API, or
  a file of the app's own for running alone, behind one interface.
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
Under svrngs: `{"record": "<the address the data dir's api file names>",
"token": "<the data dir's token file>", "who": "x-svrngs-key"}`,
lent with `svrngs share`.
