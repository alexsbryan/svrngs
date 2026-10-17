# House rules

Your house writes down what it agrees, and how it agrees it. Someone
proposes a rule, the rest of you agree or object with a reason, and the page
says when it has become your house's rule. It runs on every computer and
phone in your group, so nobody keeps a server on and nobody keeps the book.

Underneath it is [canon](https://github.com/alexsbryan/canon), a library for
a group's rules, at its release 0.5.0. canon decides every answer the page
gives: what is a rule, what is still waiting, who has a say. The page only
says those answers in plain words. svrngs carries the entries in your
group's record and vouches for who wrote each one; it never reads a rule or
enforces one (svrn-docs `rings/current/CANON_IN_SVRNGS.md`).

## How it works

The first person to open it answers one question: how does your house
decide its rules? Anyone proposes and it holds after two days unless someone
objects; most of you agree; or everyone agrees. That answer is your house's
first rule, and your house changes it the way it decides anything else.

Everyone in your group has a say, from the day they joined until the day they
left. svrngs says who that is and when (`spans`, DESIGN §3), and every
computer turns it into canon's own grants of a say, made by the house and
kept in memory, never written down. So a rule agreed last month stays agreed
when someone joins today, and the house itself never has a say.

Each entry is the person svrngs says wrote it. An entry written through
someone else's computer says so: "bo, through dee's computer".

## What waits

canon can also draw people by lot, rotate a shared thing, and let a smaller
group decide its own corner. The page offers none of these yet. In canon
0.5.0 the house, which gives everyone their say, would be counted among the
people in those draws and rotations (canon `draw.rs:179-183`,
`allot.rs:200-205`, `policy.rs:537-552`). They wait for a canon that counts
only people there.

## The files

- `index.html`, `index.js`: the page, brought to a group from the shelf as
  `House-rules` (`crates/svrngs-node/src/shelf.txt`), reading and writing at
  its own origin, `svrngs/acts`.
- `house.js`: what the record holds, handed to canon as canon's own entries,
  and canon's answers handed back. It decides nothing.
- `vendor/`: canon-core built to plain JavaScript, pinned by sha256
  (`vendor/README.md`), with canon's licence, the GNU AGPL 3.0.
- `build.sh`, `build/`: how `vendor/` is built and proven against canon's
  own command line.
- `laws.test.mjs`, `page.test.mjs`, `canon.test.mjs`: the laws house.js
  keeps, the page in Chromium and in WebKit as an iPhone, and the vendored
  build checked byte for byte. `scripts/check.sh` runs them.
- `nouns.txt`: canon's words, which svrngs's own code never uses
  (`scripts/vocab_check.py`).
