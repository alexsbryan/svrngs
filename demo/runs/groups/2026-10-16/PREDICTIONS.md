# Leaving a group on one host — setup and predictions, 2026-10-16

Row groups-run (`ralph/next/groups/STATE.md`): Ana leaves Trip from her
home page; Sam, who keeps the order of removals, tries to leave before
handing it on; Ana is sent a new link. What groups-leave (1b19d80f,
1676df59, e7bd7ad3), groups-keys (345db9dd: "final from this computer,
through V1", so no key per group was built) and groups-leave-others
(3d3ad2d6, f53e06f6, 0c3b464d) built, run as a person meets it, with
every computer and phone on this host. Written before the driver and any
data; `README.md` here says what the run gave, against each line below.
Every person is a program: no people bar is judged (AGENTS.md 7). One
host, one run: not a rate.

## 1. Setup

`W=<scratch> python3 demo/runs/groups/2026-10-16/run.py`, data in `data/`;
with `PLANT=<name>`, the same story against a build of svrngs with one
planted change, data in `data/plants/<name>/`. `judge.py` reads a take's
`summary.json` and writes its `verdicts.json`, one verdict a bar
(AGENTS.md 6: passed, failed, could-not-judge, never-ran). scripts/check.sh
runs none of this.

- **svrngs** as events-run built it (`demo/runs/events/2026-10-14/`):
  `cargo build -p svrngs` at this commit, each computer a `HOME` and data
  dir of its own under `W`, each svrngs run by the driver with
  `RUST_LOG=…,removal=debug,door=debug,origin=debug,join=debug`. The page
  built by `scripts/build-web.sh` and served on this host by
  `web/tests/serve.mjs`. n0's public relays, the default.
- **The cast.** Trip: Sam's computer, Ana's computer, Bo's computer, Cy's
  phone. Sam makes Trip (`svrngs new Trip --as Sam --origin <page>`); Ana
  and Bo join by its link (`svrngs join <link> --as …`); Cy's phone, WebKit
  as an iPhone (`web/tests/page.mjs`), opens its page address and types
  her name.
- **What they lend.** Sam lends `notes`, Bo `photos`, Ana `list`: each a
  `python3 -m http.server` on this host serving one page with its lender's
  words, lent by `svrngs share <port> --kind <name> --name <name>`.
- **Folders.** Each computer's Trip folder is the one `new` and `join` put
  at `~/svrngs/Trip`. Sam puts `before.txt` in his before Ana leaves and
  `after.txt` after.
- **Pages.** Each computer's home page is read in Chromium by
  `web/tests/home.mjs`, as `crates/svrngs/tests/leave_page.rs` reads it.
  What a computer's lend answers is asked at the address its own
  `/v1/lends` lists, on loopback with that address's `Host`, as a program
  asks (`origin.rs` `app_of`).
- **The story, in order.**
  1. Before: Ana's computer opens Sam's `notes` and Bo's `photos`; Sam's
     opens Ana's `list`; `before.txt` reaches Ana's and Bo's folders.
  2. Ana's computer goes to sleep (its svrngs stopped), its data dir is
     copied whole to `ana-copy`, as a backup of it would be, and it wakes.
  3. Ana's home page: she reads Trip's section and presses "Leave Trip".
  4. Sam's and Bo's computers are watched until each holds her out.
  5. Ana's computer opens `notes` and `photos` again; Sam's opens `list`.
  6. Ana's copy of Trip read; `after.txt` put in Sam's folder, watched on
     Bo's and Ana's.
  7. Sam's and Bo's home pages, and Cy's phone's page, read.
  8. Ana's computer asleep; `ana-copy`, her computer as it was before the
     leave, started in its place and asked for `notes` and `photos`; its
     `/v1/groups` watched. It stops; Ana's computer wakes.
  9. Sam sends a new link (`svrngs link Trip`, which runs on). Ana pastes
     it on her home page's join box. Her computer asleep, she runs `svrngs
     join <link> --as Ana`; it wakes.
  10. Ana's phone, WebKit as an iPhone, opens the same link's page
      address and types "Ana".
  11. Sam's home page: he presses "Leave Trip"; runs `svrngs leave Trip`;
      presses "Hand it on" with Bo chosen; presses "Leave Trip". Bo's home
      page read.
- **Plants**, each the whole story against a build with one change, the
  change checked to apply once, the tree put back after the build:
  - `untold`: the leave told to no computer (removal.rs `cut`, the
    computers it tells filtered to none).
  - `deleted`: the leave deleting the group's folders on the leaver's
    computer (removal.rs `leave`, `dirs_of`'s each removed first), as
    groups-leave (b) planted it.
  - `removed`: a leave said in a removal's words (question.js `removal`,
    "Ana removed Ana"), as groups-leave (c) planted it.
  - `keeper`: a keeper's own leave counted with others standing
    (membership.rs, the keeper's clause of `leaves` made true).
  - `rejoin`: a computer that left let to redeem again (join.rs `asked`
    without its `Standing::Left` arm), as groups-leave (c) planted it.

## 2. Predictions

1. **Ana leaves from her home page.** Before, Trip's section says
   "Leaving Trip stops it reaching you, and stops what you lend reaching
   its people. What you have of it stays on this computer, its folder too.
   Rejoining needs a new invitation, and this computer can’t rejoin yet."
   beside "Leave Trip", and no hand-on (she does not keep the order of
   removals). Pressed, within 5 s her page says "You left Trip. Rejoining
   needs a new invitation, and this computer can’t rejoin yet.", lists
   "Groups you left" with "Trip (Sam’s, since 16 Oct): what you have of it
   stays on this computer, its folder at …/ana/svrngs/Trip, and nothing new
   of it reaches you.", and offers "Leave Trip" no more.
2. **Every lender holds her out within 30 s**, with no round asked by the
   driver: Sam's and Bo's `/v1/groups` each list Trip's removal of Ana as
   `said: "left"`, and their `/v1/rail/people` no longer list Ana. Plant
   `untold`: red, neither within 30 s.
3. **Nothing lent reaches her, nor hers them.** Before, step 1 answers
   200 with each lend's page. After, Ana's computer answers each of
   `notes` and `photos` itself, 404, `x-svrngs-refused: unlent`, "nothing
   is lent here" (origin.rs `dialled`), never the lend's page; Sam's
   `/v1/lends` lists Ana's `list` no more, and its old address is refused
   the same way. These words do not say she left: if so, a finding, not
   this bar's failure (fence 2).
4. **Asked by her computer as it was before the leave, each lender
   refuses her.** `ana-copy` holds Trip as though she stands. Each of
   `notes` and `photos`, asked at the address it lists, is never the
   lend's page: 403, `x-svrngs-refused: removed`, "You are no longer in
   this group, so nothing of it is open to you." from the lender's door
   (`door.rs` `gone`), or `ana-copy`'s own 404 if its first sync was told
   the leave first (ring_routes.rs `told_cut`); which, recorded with the
   lenders' logs. Within 60 s of its start, `ana-copy`'s `/v1/groups` says
   Trip `left: true` (groups-leave-others (b)). Plant `untold`: red, the
   lend's page served to her copy.
5. **Her copy is kept, and nothing new reaches it.** After the leave her
   computer's `/v1/rail/log?namespace=<Trip>` still reads, holding every
   act it held before the press and one more, her leave; `before.txt` is
   in her folder, the same bytes. `after.txt` reaches Bo's folder within
   60 s, and is not in Ana's 20 s after it is in Bo's. Plant `deleted`:
   red, her folder gone.
6. **"Ana left" on the others' pages.** Sam's and Bo's home pages list
   "Ana left on <date, time>" under Trip, and Cy's phone's page the same
   in its list, within 30 s of the press on Bo's and Cy's; no page says
   Ana was removed. Plant `removed`: red, "Ana removed Ana".
7. **A keeper's leave refused until they hand on.** Sam's page shows "You
   keep the order of removals in Trip. Choose who keeps it after you, then
   leave:" with Bo to choose, beside "Leave Trip". Pressed, "Leave Trip"
   adds "You keep the order of removals in Trip: choose who keeps it after
   you, then leave." and Sam stays in Trip (his `/v1/groups`, `left:
   false`). `svrngs leave Trip` says "You decide who is removed from Trip,
   so you cannot leave it yet: hand that to someone else first, with
   `svrngs handover <name> --group Trip`, then leave." "Hand it on" says
   "Bo keeps the order of removals in Trip now, so you can leave it."; then
   "Leave Trip" leaves, "You left Trip.". Within 30 s Bo's page lists "Sam
   left on …" and Bo's `/v1/groups` says he keeps it. Plant `keeper`: red,
   Sam's first press leaves.
8. **Ana told she cannot rejoin from this computer yet.** groups-keys
   built no key per group (345db9dd), so: her home page, the new link
   pasted, says "You’re no longer in Trip on this computer, and you can’t
   join it again from here yet. Join from your phone or another computer
   instead." and offers no "Join Trip"; `svrngs join <link> --as Ana`
   stops saying "you're no longer in Trip on this computer, and you can't
   join it again from here yet: join from your phone or another computer
   instead", exit 1; Sam's register lets no key of hers in again. Plant
   `rejoin`: red, "Join Trip" offered or her page let in.
9. **Ana rejoins from her phone**, as the page tells her to: her phone
   says "You’re in Trip, as Ana" within 30 s; Sam's `/v1/rail/people`
   lists an Ana standing whose key is not her computer's, and Sam's page
   still says "Ana left on …".
10. **Words.** No page or command read says key, member, keeper, record,
    act or relay, and none says a leave as a removal.
11. **A second host, a phone on cellular, real people**: could-not-judge
    here; every people bar: never-ran.
