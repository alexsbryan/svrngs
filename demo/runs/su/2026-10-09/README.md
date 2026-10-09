# Scoped use: Ana's taste, offered beside her history — 2026-10-09

The scoped-use spike (DESIGN §8 "Then", `docs/DESIGN.md:1400-1402`; queue
`ralph/next/compose/STATE.md`, row su-0-rule): can an operation its owner
offers beside their data — Ana's taste, ranking films it is given, beside a
watch history it never hands out — be used by one app acting for one person,
and refused to every other, with svrngs carrying what any such service would
otherwise write itself? The survey of record is svrn-docs
`rings/current/ACL_AND_SCOPED_USE.md` (at `fe86f28`; a draft, and says so —
DESIGN stays normative), its smallest slice: "owner-installed operations
served beside their data, using the localhost-service model" (§4) and its
first proof's item 2 (§9). This file is su-0: the slice, what exists at HEAD
for it, the bars and their plants, the rule the lines are counted by, the one
change the spike may make, and a prediction each, committed before any app,
product or data commit of the spike, as mn-0's was
(`demo/runs/mn/2026-10-08/README.md`, `c963f93`). None of it is moved after
its data; what the data gives is written below it, in later commits.

**A spike changes no product** (the queue's fence 1), but for the one header
of §5, in a commit of its own. The taste service lives in `apps/taste/`, its
run and its checks in `apps/taste/check/`; Tonight's Picks in
`apps/movie-night/picks/`. Their nouns go in each app's `nouns.txt`, so
`scripts/vocab_check.py` keeps them out of the infra, and `scripts/check.sh`
runs none of it: the run needs Jellyfin (fence 3). What either would need of
svrngs beyond the header is written down with its evidence, never made.

## 1. The smallest proof

The group as mn-1's run made it (`demo/runs/mn/2026-10-08/run.sh`, through
`apps/jellyfin/check/check.sh up`): Sam's and Dee's Jellyfins, each lent with
`--sign-in` by the mwp middleware, *Blade Runner* in both; Movie Night's
search lent by Sam with `--reads jellyfin`, unchanged. Added:

- **Ana's taste service**, a small program in `apps/taste/` on Ana's computer
  (a computer of the group's with a data dir of its own), on loopback only,
  lent by Ana as `svrngs share <its port> --name taste --kind taste --refuse
  /diary`. It holds a **planted watch history** — rows of films Ana watched,
  each with its time and a marker string of its own, in a file beside it —
  and offers two operations, as the survey's table names them (§2):
  - `rankCandidates`, at `POST /rank`: the films it is given, each `{title,
    year}` and the provider ids where Jellyfin has them, answered in the
    order Ana's history gives them, and nothing else: no score, no row of
    the history. The order the plant gives is written into the plant beside
    the rows, as `expected`, before the run.
  - `viewingDiary`, at `GET /diary`: the rows, kept back. The service answers
    it on its port, which is loopback on Ana's computer; the door refuses the
    path to everyone who reaches the lend (§2, `--refuse`).
  
  Its owner is the keys it is started with — Ana's computer's and each of her
  phones' — since svrngs has no one person across devices
  (`docs/DESIGN.md:1440`, "another device for the same person"); those lines
  are its enforcing, counted (§4).
- **Tonight's Picks**, Movie Night's page that ranks the group's films by
  the person's taste, in `apps/movie-night/picks/`, reusing Movie Night's
  match (`apps/movie-night/match.js`). It is lent by Sam **as a lend of its
  own**, `svrngs share <its port> --name tonights-picks --kind movie-night
  --reads jellyfin --reads taste`: an app is its origin (each lend has its
  own, `web/app.js:176-178`), and the app a pass names first is the lend it
  was opened for (§2), so a feature with a yes of its own is a lend of its
  own, and Movie Night's search, lent `--reads jellyfin` alone, never holds a
  pass that names Ana's taste. Its candidates are the group's films as Movie
  Night lists them. **It asks Ana once**, in words a person reads, the
  survey's (§7):

  > Make these suggestions more your thing? Tonight's Picks can use your
  > movie preferences. Your detailed viewing history stays where it is
  > stored; the app receives ranked film suggestions.
  > [ Use my preferences ] [ Keep browsing ]

  On her yes it sends the yes, as Ana, to her taste service, which keeps it
  for the app the door names (§5); then ranks her candidates by
  `rankCandidates`; and offers "Stop using my taste", sent likewise. Asked
  once means once for this app, whichever of her phones she uses: the yes is
  kept where her taste is, not in a browser.
- **Another app**, a plant: a page in `apps/taste/check/`, lent by Sam as
  `--name other --reads taste`, whose code asks Ana's taste as Tonight's Picks
  does. It is check code, counted nowhere.
- **The people**, on a phone as `check.sh pair` names them: Ana in Chromium
  and in WebKit as an iPhone, two keys, both her taste's owner; Bo in
  Chromium, a person of the group who is not Ana. Each joins and waits for
  every lender's ring round before a bar is judged, as mn-1's `up` waited for
  Dee's (`demo/runs/mn/2026-10-08/README.md` §7, "What no prediction named");
  each one's first ask is recorded, not judged.

## 2. What exists at HEAD, and what does not

### Exists

- **The pass.** A person's key signs that a fresh key, the holder, may act as
  them toward the lends it names until a time (`crates/svrngs-core/src/pass.rs:137-152`):
  `person`, `holder`, `lends`, `until`, `sig`. The door checks the signature,
  the holder, the lapse and that it names this lend (`pass.rs:195-210`), then
  that its signer stands in a group held here, and lets the holder in as that
  person (`crates/svrngs-node/src/door.rs:680-721`, `holder_of`). The pass
  names lends and a lapse, no operation, method or path.
- **The opened app first among its lends.** The group's page signs a pass
  for the lend it opens and every lend of a kind it reads, the opened one
  first: `[l, ...reads]` (`web/app.js:172-173`, in `open`, `:166-191`), for
  `PASS_LIFE`, 12 hours (`:63`), re-signed after "without a word to the
  person" (`:60-62`). The first is a convention of the page's: `pass.rs`
  gives the order no meaning, and the door reads none — `holder_of` keeps
  `pass.person` and drops `pass.lends` (`door.rs:716-720`).
- **`share --refuse`.** A lender keeps a path of a lend from everyone who
  reaches it (`crates/svrngs/src/cli.rs:120`, parsed `:227-233`); the door
  refuses it before the service sees it, compared decoded, case-blind, at a
  segment boundary (`crates/svrngs-node/src/lend.rs:133-147`, `covers`;
  `door.rs:797-804`). `--reads` lets a lent page open what the groups lend
  of a kind, as whoever opens it (`cli.rs:123-124`, parsed `:240-245`).
- **`x-svrngs-person` and `x-svrngs-key`.** The door sets on every request it
  carries the person's name and the key it verified — the pass's signer for
  a holder (`door.rs:112-120`, `:989-992`), every client header under
  `x-svrngs-` stripped whole first (`door.rs:474-500`, `rewrite_headers`;
  held by `crates/svrngs-node/src/door/tests.rs:49-70`).
- **The service believes its port.** A process on Ana's computer reaching
  the taste service's port can name any person; that is Ana's own computer,
  and §9's per-process tie keeps its trigger (`docs/DESIGN.md:1446`,
  `mint_tie`).

### Does not exist

- **The door telling a service which app is asking.** Nothing a service
  receives names the page that asked; a request from Tonight's Picks and one
  from the other app reach Ana's taste identical but for their bodies (§5
  adds one header).
- **A lend only its owner and the apps they allow reach.** The door lets in
  anyone of a group held here whose pass names the lend (`door.rs:708-712`),
  and a pass names every lend of a kind its page reads (`web/app.js:172`), so
  Bo's pass names Ana's taste the moment Tonight's Picks is opened.
- **A person's yes before a lend enters a pass.** The page puts every lend of
  a read kind in the pass with no word to the person (`web/app.js:172-175`);
  a yes is the app's to ask and the service's to keep.
- **A stop sooner than the pass's lapse.** A pass opens its lends until
  `until` (`pass.rs:203-205`); nothing withdraws one, and the page re-signs
  another unasked (`web/app.js:60-63`). Only the service can stop sooner.
- **On a computer**, a page reaches every lend of a group held here as the
  computer's person, with no pass (`crates/svrngs-node/src/origin.rs:84-93`;
  `docs/DESIGN.md:1449`), so no app can be named there at all. The spike is
  judged on a phone; the computer is said, not judged.
- **An app SDK and a sandbox** wait in §9 (`docs/DESIGN.md:1426`, "an app
  SDK, a sandboxed fold engine"): the taste service is a program Ana runs,
  not code the group ships to her.

## 3. The bars, in their spirit, and their plants

Judged on a phone in Chromium and in WebKit as an iPhone (the plants in
Chromium), from the group's list. Each bar names what it guards and the
boundary that enforces it; where letter and spirit part, both are said and
the spirit judges (fence 4). One host, one run of each: not a rate
(AGENTS.md 7). A run that could not finish is could-not-judge, never a pass
(AGENTS.md 6). Every refusal is watched as its status, its body and the log
line of the boundary that made it.

1. **Ana's picks, ranked by her taste, asked once.** Ana opens Tonight's
   Picks, reads the question, says yes, and sees the group's films in the
   plant's `expected` order; opened again, on either phone, she is not asked
   again. Guards: a person adds a benefit plainly, once, and gets it.
   Recorded: the time from her yes to the ranked list, and from opening to it
   after.
2. **Bo's Tonight's Picks asking Ana's taste is refused** — at the taste
   service, by its owner's keys: the door lets Bo in (his pass names the
   lend, §2). Bo sees the films unranked, said so. Guards: one person's
   taste is never another's. Plant `bo`: the owner check taken out, Bo's
   ask answered.
3. **No app reaches the history** — at the door, by `--refuse /diary`:
   Tonight's Picks asking `/diary` as Ana is refused 403 and the service
   logs no request for it; and no planted row's marker is in any byte any
   browser received (a scan, as `apps/jellyfin/check/scan.py`'s). Guards:
   what the owner keeps back stays where it is stored. Plant `history`: the
   lend shared without `--refuse`, the history answered.
4. **Another app holding a pass that names Ana's taste is refused** — at
   the taste service, by the header of §5: Ana opens the other app, its pass
   names her taste, and its ask is refused, since her yes names Tonight's
   Picks. Guards: a yes to one app is not a yes to all. Plant `other-app`:
   the app check taken out, the other app answered.
5. **Any ask after Ana stops it is refused, its delay said honestly** — at
   the taste service. After "Stop using my taste", the next ask from
   Tonight's Picks, on either phone, is refused, and what Ana reads at the
   stop says what is true: that the service refuses from now on, and that
   the ranked list already on her screen is not unread. The pass still opens
   the lend at the door until its `until` (§2), so the door carries asks the
   service refuses; that is said, not hidden. Recorded: the time from the
   stop to the first refused ask; an ask in flight when the stop lands is
   recorded as answered or refused, not engineered. Guards: changing your
   mind takes effect where the data is, and nobody is told it took effect
   sooner than it did. Plant `after-stop`: the stop check taken out, the ask
   answered.
6. **A refusal is never shown as an empty answer** — in Tonight's Picks.
   Each refusal of bars 2, 4 and 5, and Ana's computer asleep (its svrngs
   stopped after the list, as mn-1's Cy), is said as what it is — not hers,
   not allowed, stopped, did not answer — and the films are shown unranked,
   never as no picks or a ranking of nothing. Guards: a person never reads a
   refusal as "you have no taste". Plant `empty`: a refusal shown as an empty
   ranking.

**Measured, not a bar: a yes the app sends unasked.** The other app sends
the yes itself, as Ana, with no word to her. The taste service cannot tell
it from the yes Ana pressed in Tonight's Picks: both arrive as Ana, from an
app the door names. Recorded as answered or refused; it is the gap of "a
person's yes before a lend enters a pass" (§2), the verdict's.

**Each plant watched failing first** (su-1), behind a switch its check sets,
each plant's lines marked `plant` so §4's count keeps them apart; then green.

## 4. The rule the lines are counted by

The spike's output is what the taste service had to enforce itself, counted,
against what the door could: a service beside its data writes its operation,
and every line it writes to decide *who may* is a line svrngs might carry.

**Code lines.** Every line of every `*.js`, `*.mjs`, `*.html` and `*.css`
under `apps/taste/`, outside `apps/taste/check/`, and under
`apps/movie-night/picks/`, that is not blank and does not begin with `//`,
`/*`, `*` or `<!--`. A file shared by a link is counted where it lives, not
again through the link.

**Marks.** A comment line `// su: <side> <piece>` (`/* su: … */` in CSS,
`<!-- su: … -->` in HTML), optionally followed by why, sets the piece of
every code line after it in its file, until the next mark. The sides and
pieces are a closed set:

| Side | Piece | What it holds |
|---|---|---|
| own | `rank` | the taste service's operation: scoring candidates beside the history, answering the order |
| own | `diary` | holding the history: reading the planted rows, answering `/diary` on its port |
| own | `serve` | the taste service's server: its port, routes, reading a body, writing an answer |
| own | `picks` | Tonight's Picks' page: its candidates from Movie Night's match, the ranked and unranked lists, its start |
| own | `consent` | the words Ana reads — the question, the stop, each refusal said — and her presses |
| enforce | `owner` | the taste service deciding the asker is Ana: her keys, compared with `x-svrngs-key` |
| enforce | `app` | the taste service deciding the app is one Ana said yes to: the header of §5, compared with the yeses kept |
| enforce | `yes` | taking Ana's yes for an app and keeping it |
| enforce | `stop` | taking Ana's stop, and refusing after it |
| enforce | `diary` | the taste service keeping the history from an ask through the door |
| enforce | `refuse` | the taste service's refusals as refusals: status, kind, words; an absent header refused, never defaulted |
| plumbing | `find` | Tonight's Picks finding Ana's taste and the Jellyfins among what svrngs handed it, and building their addresses |
| plumbing | `refusal` | Tonight's Picks telling a refusal — svrngs's or the service's — from an answer |
| plant | `<its name>` | a line that exists only for one of §3's plants: counted apart, in no side |

A line before any mark in its file is `unmarked`; the count is
could-not-judge until there are none. A region doing two pieces' work is
split until each does one. Tonight's Picks' files also open with `// mn: own
picks`, so mn-0's count (`demo/runs/mn/2026-10-08/README.md` §4), if rerun,
reads them as Movie Night's own rather than `unmarked`; su's count skips
that line as a comment. The count is this, run from the repository's root —
watched on a planted sample before this commit, `/tmp/su-plant`: a service
file with a block comment and a plant, an HTML file with a line before its
mark, a file under `check/`, and a Picks file with a line between its `mn:`
mark and its first `su:` mark, which gave `1 enforce owner`, `3 own serve`,
`1 plant bo`, `1 plumbing find`, `2 unmarked`, each as the rule says:

    { find apps/taste -path apps/taste/check -prune -o -type f \
        \( -name '*.js' -o -name '*.mjs' -o -name '*.html' -o -name '*.css' \) -print
      find apps/movie-night/picks -type f \
        \( -name '*.js' -o -name '*.mjs' -o -name '*.html' -o -name '*.css' \) -print
    } | sort | xargs awk '
      FNR == 1 { piece = "unmarked" }
      /^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*su: (own|enforce|plumbing|plant) [a-z-]+/ {
        sub(/^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*/, ""); piece = $2 " " $3; next }
      /^[[:space:]]*($|\/\/|\/\*|\*|<!--)/ { next }
      { n[piece]++ }
      END { for (p in n) print n[p], p }' | sort -k2

**Against what the door could.** For each `enforce` piece su-1 writes its
lines (file:line) and what the door could decide in its place from what it
already verifies — the key it checked, the pass it checked, the lend's
declaration — or why it could not; each `plumbing` piece likewise, as mn's
were. Written here before the data, as the question each answers:

- `owner`: the door verifies the key; it could keep a lend to its lender's
  person if svrngs knew a person across devices (§2; `docs/DESIGN.md:1440`).
- `app`: the door verifies the pass; with §5's header the service decides,
  and the door could, if it held which apps the lender allows.
- `yes`: the group's page could ask before a lend enters a pass, and the
  door read the answer from the pass; that would also close §3's unasked
  yes.
- `stop`: the door could refuse a pass's lend sooner than its lapse if it
  held the stop, and the page stop re-signing it.
- `diary`: the door does it already, by `--refuse` (§2): predicted 0 lines.
- `refuse`: a service's own words; the door's part is that its refusals and
  the service's are told apart (mn's candidate 2,
  `demo/runs/mn/2026-10-08/README.md` §9).

**Each piece svrngs could carry is a candidate**, with its lines and
evidence; the operator queues what is made (the queue's preamble).

## 5. The one change the spike may make

At most one header, set by the door from what it already verifies, in a
commit of its own; su-0 chooses it, since bar 4 cannot be judged without
it — at HEAD the service cannot tell Tonight's Picks from the other app (§2).

**`x-svrngs-app`: the label of the pass's first lend**, set by the door on
every request it carries for a pass's holder, beside `x-svrngs-person` and
`x-svrngs-key`. It is what the page opened (`web/app.js:172-173`), under the
person's own signature (`pass.rs:182-193`, the message covers each lend in
order); the holder, which an app's origin keeps, cannot re-sign it. Absent on
a request that came with no pass — a member's own key, a computer's origin
(`origin.rs:84-85`, `Who::Own`) — and a service reads its absence as no app
named, never as an app allowed. A client's own is stripped, as every header
under `x-svrngs-` is already (`door.rs:474-500`). The pass's type does not
change: the first lend's meaning becomes the door's to read, and whether a
pass should name its app apart from its lends is the verdict's.

The commit is held by a door test that the header is the pass's first lend
and that a client's own is stripped, watched failing on its plant, every
door test unchanged, and DESIGN §3's account of the door's headers changed in
the same commit (AGENTS.md 3). Size: the repository is at 14,466 of 14,750
(`scripts/lean_check.py` at `766be45`), and su-1's row holds the header
inside 14,500. Anything more — the door keeping a lend to its owner, a yes,
a stop — is the verdict's.

## 6. Predictions, 2026-10-09 05:17 UTC, before any app or run

- **Bar 1: passes in both engines.** One question, one yes; the ranked list
  in `expected` order within 2 s of the yes through the relay, the taste
  service a few milliseconds of it; opened again, on the other phone too, no
  question, the list within 2 s of opening. Each Ana's first ask after
  joining is refused by Ana's computer's door until its ring round, as Dee's
  was in mn-1 (`demo/runs/mn/2026-10-08/README.md` §8, bar 1) — recorded,
  not judged, and said.
- **Bar 2: passes**, the service refusing Bo 403 by his key while the door
  logs "a pass opens this lend" (`door.rs:714-715`) for the same request:
  the door carries what the service refuses.
- **Bar 3: passes**, the door refusing `/diary` 403 "taste is lent without
  /diary" (`door.rs:802-804`), no line in the service's log, no marker in
  any byte.
- **Bar 4: passes with the header.** Without it — the plant's guard out, or
  the header absent — the other app is answered, since nothing else tells it
  from Tonight's Picks.
- **The unasked yes: answered.** The other app's own yes is kept, and its
  next ask ranked: the gap §2 names.
- **Bar 5: passes.** The first ask after the stop refused at once, within one
  round trip (under 2 s), on both phones; the door still letting her pass in,
  until a lapse up to 12 h away (`web/app.js:63`). An ask in flight at the
  stop: answered, the service taking one request at a time; not judged.
- **Bar 6: passes.** Each refusal said as its own, the films unranked.
  Asleep: "did not answer" after 60 s, as Cy's was (60,291 and 60,440 ms,
  `demo/runs/mn/2026-10-08/README.md` §7, bar 4), the unranked films shown
  at once.
- **The count:** the taste service own about 90 (50 to 160), enforce about
  50 (25 to 90) — `owner` 6, `app` 6, `yes` 12, `stop` 8, `diary` 0,
  `refuse` 15; Tonight's Picks own about 80 (40 to 150), plumbing about 30
  (15 to 60) — `find` 20, `refusal` 10. Of the enforce lines, all but
  `refuse`'s are the door's to carry if svrngs held a person across devices,
  the apps a lender allows, a yes and a stop: about three in four.
- **The product: one commit.** `git diff <this commit>..HEAD -- crates/*/src
  web` is the header's commit alone, under 40 lines of product code;
  `scripts/check.sh` names neither Jellyfin nor taste.

## 7. Out

Each waits for this proof, as the survey orders it (§4, §9: "not an
invitation to build a general framework before the experience works"):

- the runner and sandbox, and code that travels to the data (§4's isolated
  runner; DESIGN §9, `docs/DESIGN.md:1426`);
- peer inference and the journal, a source's excerpts on another computer
  (§5);
- fair-use budgets, and repeated asks that reconstruct a history (§4, §6);
- code-version approval: a yes is to an app's lend, whatever code it serves;
- grants kept and synced across computers, recovered when a host returns
  (§6): the yes lives in the taste service's own file on Ana's computer;
- the computer's origin (§2), a second non-media composition, and the
  access review page (§7) beyond Tonight's Picks' own stop.

## 8. What the run gave (su-1)

The run of 2026-10-09, 06:06–06:21 UTC, on this host through iroh's public
relays: `run.sh` here, through `apps/taste/check/check.sh`, the plants
first (06:09–06:15), then the runs as the code is kept (06:15–06:21), then
the judge (`apps/taste/check/judge.py`, its verdicts in `verdicts.json`,
each line in `steps.log`). The code run is `305efde`'s; the header is
`55dca40`'s. A first whole run at 05:46–06:06 gave the same verdicts; what
it showed was fixed in `305efde` before this one, and its data is not kept.
One host, one run of each: not a rate (AGENTS.md 7).

Each person joined from the group's link, Ana twice, a phone an engine,
each phone's key read from Sam's computer's log as it admitted it and added
to her taste's owners (`check.sh`, `OWN`; nothing in svrngs names a person
across devices, §2). Every refusal below is the taste's 403 with its kind,
the taste's log line (`<run>-taste.log`) and Ana's door's
(`<run>-ana-door.log`).

### Each bar against its prediction

1. **Ana's picks, asked once: passed in both engines**, as predicted. One
   question, the survey's words (`verdicts.json` "bar 1" `question`); the
   list in the plant's `expected` order, Arrival, Brazil, Blade Runner,
   Alien (`diary-planted.json`), 243 ms after her yes in Chromium and 229
   ms in WebKit (`yes_to_list_ms`); opened again on that phone, no
   question, the list in that order in 2,572 and 2,114 ms; on her other
   phone, no question, in 3,623 and 3,102 ms (`again_ms`,
   `other_phone_ms`). The prediction said within 2 s of opening: the
   reopenings took 2.1–3.6 s, because Tonight's Picks asks the taste
   only after both Jellyfins have listed their films; the taste logged
   each answer 0 to 48 ms after the ask, most within 5
   (`night-*-taste.log`). Predicted, each Ana's
   first ask refused by Ana's computer's door until its ring round: so in
   all six first opens of the plants and `asleep`, each a phone that had
   just joined alone (`refused-svrngs` in their `tries`; Ana's door "its
   signer stands in no group held here", 8 times in its log, twice in
   `history` and `other-app`), and in neither night, where three joins
   before the first open took about 45 s (`join_ms`, about 15 s each).
   Said, not judged.
2. **Bo refused: passed in both**, as predicted: the taste refused him
   `not-yours` by his key while Ana's door logged "a pass opens this lend
   ... person="Bo"" for the same request (`verdicts.json` "bar 2"); he read
   "The taste at 93868136 is someone else's. These films are not ranked.",
   the group's four films beside it, unranked.
3. **No app reaches the history: passed in both**, as predicted: `/diary`
   asked from Tonight's Picks' page as Ana answered 403 `{"error":"taste
   is lent without /diary"}`, Ana's door logging "the lend refuses this
   path", the taste no line for it. The scan (`scan.json`, by
   `apps/jellyfin/check/scan.py` unchanged, the five rows' markers among
   the secrets it searches): no marker in any byte any of the seven
   browsers of the kept runs received, 62 to 174 responses each; the
   five in `plant-history`'s, its plant.
4. **Another app refused: passed in both**, as predicted with the header:
   the other app's ask, from a pass naming Ana's taste, refused
   `not-asked` — the door named it `x-svrngs-app` by its own label, which
   Ana's yes does not name (`night-*-taste.log`).
5. **Stopped: passed in both**, as predicted: after "Stop using my
   taste" on her second phone, the next ask, from that phone reopening,
   refused `stopped`, 397 ms after the press in Chromium-first and 422 ms
   in WebKit-first (`stop_to_refused_ms`), and her first phone's next ask
   too. What she read at the stop: "Stopped. From now on the taste at
   93868136 refuses Tonight's Picks. The list ranked before you stopped
   stays on this screen until you leave it." The door carried each refused
   ask, "a pass opens this lend" then `status=403` (`night-chromium-ana-
   door.log`, 06:17:22.918): it will until the pass's `until`, 12 hours
   on. No ask was in flight at a stop.
6. **A refusal never shown as empty: passed**, as predicted: Bo's and the
   stop's refusals each said as themselves beside the four films unranked,
   in both engines; and Ana's computer put to sleep after her list, "The
   taste at 93868136 did not answer: its computer may be asleep. These
   films are not ranked." after 32,384 ms, not the 60 s predicted from
   Cy's (`verdicts.json` "bar 6" `asleep`), the films shown unranked.

**The unasked yes: answered**, as predicted, in both runs: the other app
sent a yes as Ana with no word to her (200), and its next ask was ranked
(`the unasked yes`). The taste cannot tell it from the yes she pressed:
both come as Ana from an app the door names.

**Each plant watched failing first**, then the code as kept green in the
runs after (`verdicts.json` "plants", each `failed` as it should be):
`bo` — Bo ranked by Ana's taste; `history` — `/diary` answered 200 with
the rows, all five markers in what the browser received; `other-app` —
the other app ranked; `after-stop` — her next ask after the stop ranked;
`empty` — Bo's refusal shown as "No picks for you tonight." and no films.

**The product: one commit**, as predicted: `git diff cf56039..HEAD --
crates web` is `55dca40` alone, `crates/svrngs-node/src/door.rs` +29 −4
and its test; `scripts/lean_check.py` 14,485 of 14,750, 19 lines over
su-0's 14,466; `scripts/check.sh` names neither Jellyfin nor taste.

### The count, and what the door could carry

By §4's awk at `305efde`, none unmarked (`verdicts.json` "the count"):

| Side | Lines | Pieces (predicted) |
|---|---|---|
| own, the taste service | 52 (about 90) | serve 28, rank 22, diary 2 |
| own, Tonight's Picks | 134 (about 80) | picks 86, consent 48 |
| enforce, the taste service | 46 (about 50) | yes 16 (12), app 9 (6), stop 9 (8), refuse 9 (15), owner 3 (6), diary 0 (0) |
| plumbing, Tonight's Picks | 39 (about 30) | find 26 (20), refusal 13 (10) |
| plant, apart | 5 | bo, other-app, after-stop, empty, the switch |

Every figure inside its predicted range, the taste's own near its
floor (52 against about 90, 50–160). The enforce lines, at `apps/taste/taste.mjs` as
`305efde` has it, and what the door could decide in their place:

- `owner`, 3 (:76-77, :130): Ana's keys in a file, compared with
  `x-svrngs-key`. The door holds the lend's lender and verifies the key;
  it could keep a lend to its lender's person if svrngs knew a person
  across devices. The run had to read each phone's key from Sam's log.
- `app`, 9 (:94-96, :100, :107-108, :132, :150-151): the yes looked up by
  `x-svrngs-app`, an absent header refused (:132, here and not under
  `refuse`). With 55dca40 the service decides; the door could, if it held
  which apps the lender allows.
- `yes`, 16 (:82-90, :134-140): a file of yeses by app, written on
  `/yes`. The page could ask before a lend enters a pass, and the door read
  the answer from it; that also closes the unasked yes, which here is
  answered.
- `stop`, 9 (:105, :142-148, :153): a stop kept beside the yes and every
  ask after refused. The door could refuse a pass's lend sooner than
  `until` if it held the stop, and the page stop re-signing it.
- `diary`, 0: the door did it, by `--refuse` (bar 3).
- `refuse`, 9 (:68-71, :115-119): who asks, read from the door's headers,
  and a refusal as one. A service's own words; svrngs's part is that its
  refusals are told from the service's, which Tonight's Picks did itself
  (`plumbing refusal`, 13, `apps/movie-night/picks/picks.js:58-70`).

Of the 46, `owner`, `app`, `yes` and `stop` — 37, about four in five — are
the door's to carry if svrngs held a person across devices, the apps a
lender allows, a yes and a stop (predicted about three in four).
`plumbing find`, 26 (`picks.js:24-51`), is mn's candidate again: the lends
read from svrngs's own store whole, and a taste told from a library by its
lend's name, since no kind comes with a lend.

### What no prediction named

- **The `Origin` a service sees.** The door left the worker's
  `origin`, Tonight's Picks' own origin, on each ask to the taste (Ana's
  door, "the request's origin is not the lend's own; left as it is"): set
  by svrngs's worker for a post (`web/lend.js:278-289`), not signed, so a
  holder outside a browser can name any. `x-svrngs-app` is the pass's
  first lend, under the person's signature.
- **Ana's computer refused her new phones, as predicted, but not in the
  nights:** the ring round had come while the others joined (bar 1).
  `past` in `su.mjs` reopened until it had.
- **One `--reads` page holds the taste for anyone who opens it.** Bo's
  pass named Ana's taste the moment he opened Tonight's Picks (bar 2's
  door line): the refusal is wholly the taste's.
