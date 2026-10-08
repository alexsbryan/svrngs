# Movie Night, a thin app over the group's Jellyfins — 2026-10-08

The Movie Night spike (DESIGN §8 "Then", :1367-1373; queue
`ralph/next/compose/STATE.md`, first commit `37b4a4c`): can a static page
lent beside the group's Jellyfins search them all as the person using it,
open the chosen copy in its own Jellyfin and come back, writing only what
makes it Movie Night? The survey of record is svrn-docs
`rings/current/THIN_APP_COMPOSITION.md` (at `aec530e`; an exploration, and
says so — DESIGN stays normative). This file is mn-0: the smallest proof,
what exists at HEAD for each operation, the bars, the rule Movie Night's
lines are counted by, and a prediction each, committed before any app or
data commit of the spike, as mw-0's was (`demo/runs/mw/2026-10-08/README.md`,
`4c2b677`). None of it is moved after its data; what the data gives is
written below it, in later commits.

**A spike changes no product** (the queue's fence 1). Movie Night lives in
`apps/movie-night/`, its run and its checks in `apps/movie-night/check/`.
What Movie Night would need of svrngs to work at all is written down with
its evidence, never made: `git diff 37b4a4c..HEAD -- crates/*/src web` is
empty at the spike's end. Its nouns go in `apps/movie-night/nouns.txt`, so
`scripts/vocab_check.py` keeps them out of the infra, and
`scripts/check.sh` runs none of it: it needs Jellyfin (fence 3).

## 1. The smallest proof

Three libraries in one group, on this host, through iroh's public relays as
`apps/jellyfin/check/check.sh up` sets them:

- **Sam's** and **Dee's**: two scratch Jellyfin 12.0.0s, each lent with
  `svrngs share <its port> --sign-in <its middleware's port>` by the mwp
  middleware (`apps/jellyfin/mw/`, as `apps/jellyfin/README.md:28-43` has a
  lender run it), each on its own lender's computer (a data dir of its own).
  `check.sh up` makes one Jellyfin, at 8096 (`check.sh:62`, `:123-124`,
  `:187`); the second is at a port of its own with data dirs of its own,
  as mn-1 extends it.
- ***Blade Runner* (1982) in both**, a short test film each, with a
  `.nfo` beside it naming the film's TMDb and IMDb ids, so Jellyfin has its
  provider ids from the library without the internet. Beside it Sam's
  library has a film Dee's has not, and Dee's one Sam's has not.
- **One film one person may see and another not**: in Sam's Jellyfin a
  second library holding one film, kept from one person by their account's
  policy (`EnableAllFolders` false), set by the check through Jellyfin's
  API once the middleware has made that account. The middleware writes
  only its read-only keys over a user's policy and keeps the rest
  (`apps/jellyfin/mw/src/viewer.rs:296-317`, `:138-146`), so this costs no
  line of the middleware or of Movie Night: it is in. If mn-1 finds it
  costs either, it is out, said.
- **Cy's**, a third lender whose computer is asleep: a lend of kind
  `jellyfin` whose lender's `svrngs` is stopped. Two cases, each recorded:
  asleep *after* the group's page listed it (so its lend is in what
  Movie Night is handed, and Movie Night's request to it fails), and asleep
  *before* (so the page lists Cy among those whose computer did not answer,
  `web/app.js:222-227`, and hands Movie Night nothing of Cy, §2).

**Movie Night** is a static page — HTML, a script, styles; no backend —
in `apps/movie-night/`, served on this computer by a plain file server and
lent by Sam as `svrngs share <its port> --kind movie-night --reads
jellyfin` (`crates/svrngs/src/cli.rs:123-125`). It asks each Jellyfin's
own API from the person's browser. The people are as `check.sh pair`
names them (`:225`): on a phone Ana and Ben in Chromium, Cleo and Dara in
WebKit as an iPhone, each a person new to the group; on a computer Alex,
the bare command's.

## 2. The three operations, and what exists at HEAD

Each as the survey names it (its "Three composition operations"), with
what an app can lean on today. Where nothing exists, Movie Night does what
an app could at HEAD, and every line of it is counted as plumbing (§4).

### Discover — which Jellyfins this person may use

- **On a computer, the app's own computer answers it, but not to a page.**
  `GET /v1/lends?namespace=<id>` answers each lend with its kind, its
  lender's person and key, its `origin`, and the people whose computer did
  not answer (`crates/svrngs-node/src/surface.rs:13-21`, `:138-185`), and
  `GET /v1/groups` the namespace (`:10-12`). It is the record API's, and a
  request from a page in a browser is refused 403 before any route runs
  (`crates/svrngs-node/src/api.rs:100-124`, `not_from_a_page`). A page
  opened at its origin on a computer is handed nothing: no fragment, no
  pass (`crates/svrngs-node/src/origin.rs:57-64`). So on a computer a
  lent page has no way to learn which Jellyfins there are.
- **On a phone, the group's page hands it over, as internal data.** The
  page opening a lend signs a pass for it and every lend of a kind it
  reads (`web/app.js:166-191`; the filter `:172`), and hands the lend's
  origin `{label, lends, pass, holder, relay, page}` in the fragment
  (`:179-186`), each of `lends` only `{label, key, name}` (`:173`): no
  lender's person, no kind, and none of `unanswered`, which the page reads
  (`:207`) and says (`:224`) but does not hand on. The lend's start page
  keeps it in the origin's IndexedDB, `svrngs-lend`, as `pass <label>`
  (`web/open.js:41-44`; `web/store.js:136-152`), strips the fragment
  (`open.js:42`) and opens the lend's root (`:54`). Movie Night's own
  script sees none of it but by reading that store, whose record holds the
  pass and its holder key beside the labels: svrngs's internal handoff,
  which the survey says is "not yet that developer contract".
- **A lend's address is built, not given.** Another lend is at
  `<label>.<this page's host>` (`web/lend.js:78-80`, `baseOf`), the scheme
  the group's page opens it by (`web/app.js:176-178`); nothing tells an app
  it.

### Request — ask a Jellyfin as the person

- **On a phone, a lent page reaching a lend it reads.** A request Movie
  Night's page makes to another lend's address is served by Movie Night's
  own worker, which matches it to a lend in the pass (`web/lend.js:11-13`,
  `:82-91`, `at`), dials that lend's lender with the pass
  (`:259-309`, `x-svrngs-pass` `:277`), and answers as the door gave it; a
  lend not in the pass is refused 403 (`:268`); a lender who does not
  answer is a `502` "The computer lending this did not answer" after a
  second try (`:300-309`). It is held by a browser test: a page whose
  lender declared `--reads` reads a range of another computer's lend at
  that lend's origin, and the other door names the person, not the page's
  lender (`crates/svrngs-node/tests/browser/main.rs:581-584`). The door
  lets the pass's holder in as the person who signed it.
- **The person is signed in by the door, not by Movie Night.** Each
  Jellyfin lent with `--sign-in` has the door ask the lender's program on
  the person's first request and set the answer's `authorization` on each
  of their requests, displacing the client's (DESIGN §3, "The room", call
  6, `docs/DESIGN.md:461-485`), and swap the person's token for the
  stand-in in every textual answer (`:497-524`;
  `crates/svrngs-node/src/door/sign_in.rs:50`, `STAND_IN`). So a request
  from Movie Night carrying no `authorization` at all reaches Jellyfin as
  the person, on their own device — what the survey's "a custom client
  must satisfy Jellyfin's device/API contract" leaves open, and §5
  predicts.
- **On a computer, a page reaching another lend.** A request at
  `<label>.localhost:<port>` is carried by `origin.rs` as this computer's
  person to any lend whose lender stands in a group held here
  (`origin.rs:70-111`; the one check `:87-93`): `--reads` is not read on
  this way, so it neither grants nor limits anything. The door sets no
  `access-control-*` header (DESIGN §3, `docs/DESIGN.md:172-175`), so
  whether a page on Movie Night's origin may read Jellyfin's answer rests
  on what Jellyfin's own answer carries.
- **What `reads` grants.** A pass names lends and a lapse, no method or
  path (`crates/svrngs-core/src/pass.rs:136-151`, `:195-210`): a page
  reading `jellyfin` may ask it anything the person may, writes included.
  It is described so, not called read-only.

### Open — the chosen copy in its own Jellyfin, and back

- **On a phone, a lend's origin opens only with its own pass.** A lend's
  start page takes a handoff for its own label alone (`web/open.js:43`):
  "A lent page can use only its own pass" (DESIGN §3, "Lent apps", call 6,
  `docs/DESIGN.md:326-332`). Opened with no fragment it reads the pass it
  keeps, `for` a label, and goes to `back` (`open.js:45-54`), the path the
  lend's worker sends it when it holds no pass (`web/lend.js:262-266`). The
  group's page opens a lend at `#open=<label>` (`web/app.js:255-267`,
  `:220-221`), but at its root: the handoff carries no path
  (`app.js:178`). An origin with no worker is the page's static host, which
  answers a path it does not have 404 (`web/tests/page.mjs:30-42`). So a
  Jellyfin whose origin this phone has opened from the list (its pass kept
  for `PASS_LIFE`, `app.js:63`) can be opened at an item; one it never
  opened cannot, at HEAD, but at its root by way of the group's page.
- **On a computer** a lend's origin answers any path with no pass
  (`origin.rs:66-111`), so Jellyfin opens at the item's address directly.
- **Back** is the browser's: nothing in svrngs carries a way back between
  two lends.
- **Signed in on arrival.** The door puts the sign-in's script first in
  each of the person's Jellyfin pages (`docs/DESIGN.md:525-548`), which
  writes jellyfin-web's stored sign-in with the stand-in, so jellyfin-web
  opens as the person with nothing typed (`demo/runs/mwp/2026-10-08/README.md`).

## 3. The bars, in their spirit

Judged on a phone in Chromium and in WebKit as an iPhone, from the group's
list; and on a computer as far as `reads` goes there, the gap said where it
stops. Each bar names what it guards; where the letter and the spirit part,
both are said and the spirit judges (the queue's fence 4). One host, one
run of each: not a rate (AGENTS.md 7). A run that could not finish is
could-not-judge, never a pass (AGENTS.md 6).

1. **A typo search gives one card per film, its copies under it.**
   `bladruner` gives one *Blade Runner* card with two copies, Sam's and
   Dee's; the person who may not see Sam's kept film has no card or copy of
   it, and the person who may does. Guards: the film is what a person
   browses, a library's item only a source, and each person's results are
   what each library lets *them* see. Matched by the films' provider ids
   where both copies have them, else by title and year; a title alone never
   merges two (§4). Recorded: the time from the query typed to the first
   card, and to the last library's answer — north star 1's "ten seconds"
   the guide, not a cutoff.
2. **Choosing a copy opens it playing in its own Jellyfin.** The chosen
   copy's Jellyfin opens at that item, signed in as the person with nothing
   typed, and one press of jellyfin-web's own Play gives a `<video>` that
   moves within 30 s (u5-3 (a)'s). Guards: the person reaches the film they
   chose without hunting for it or signing in; Jellyfin owns playback. The
   letter says "opens playing"; jellyfin-web's item page has no address
   that plays (the survey: "choose Play, and arrive"), so one press of its
   Play is the spirit's. Judged twice on a phone: by a person whose phone
   never opened that Jellyfin from the list, and after they have.
3. **Coming back keeps the query and the results.** From Jellyfin, the
   browser's Back, as many presses as Jellyfin's own history takes
   (counted), returns to Movie Night with the query typed and the same
   cards and copies on screen, nothing retyped. Guards: choosing is cheap
   to undo.
4. **The asleep library is said not to have answered, never an empty
   one.** While Cy's library is being asked, Movie Night says it is still
   asking; when it fails, it says Cy's library did not answer; at no time
   does it say or show that Cy's has nothing. The others' results appear
   without waiting for it. Guards: a person never concludes a film is not
   in the group because a computer was asleep. Judged in both of §1's
   cases; the time until it is said recorded.
5. **Movie Night holds no credential, no proxy and no copy of svrngs's
   addresses.** Its code sets no `authorization`, token, cookie or pass of
   its own and keeps none; it has no server but the file server that
   serves it; and it neither spells a lend's address nor reads svrngs's
   own store. Searched: every byte its page received (`apps/jellyfin/check/scan.py`)
   holds no lender's key, password or token, nor the person's own Jellyfin
   token; and Movie Night's source names none of `x-svrngs-`, `svrngs-lend`,
   `.localhost` or a lend's label. Guards: an app that composes others
   cannot leak what it never holds, and does not break when svrngs's
   addresses change.

**Each bar watched failing on its plant** (mn-1), in Movie Night's own
code, behind a switch its check sets, each plant's lines marked so §4's
count keeps them apart: the asleep library shown as an empty one (bar 4);
one card per copy rather than per film (bar 1); a lender's token written
into Movie Night (bar 5).

## 4. The rule Movie Night's lines are counted by

The spike's output is this split, measured; what the middleware taught
(DESIGN §3, "The room", call 6, "Made production", `docs/DESIGN.md:427-436`)
is that a spike can pass its bars with the library carrying none of the
weight.

**Code lines.** Every line of every `*.js`, `*.mjs`, `*.html` and `*.css`
under `apps/movie-night/`, outside `apps/movie-night/check/`, that is not
blank and does not begin with `//`, `/*`, `*` or `<!--`.

**Marks.** A comment line `// mn: <side> <piece>` (`/* mn: … */` in CSS,
`<!-- mn: … -->` in HTML), optionally followed by why, sets the piece of
every code line after it in its file, until the next mark. The sides and
pieces are a closed set:

| Side | Piece | What it holds |
|---|---|---|
| own | `match` | understanding Jellyfin's catalogue — which endpoint, which fields, reading the answer; normalising titles, the fuzzy match, grouping copies into films by provider ids, else title and year |
| own | `cards` | the page itself: its HTML, styles and start; cards, copies under them, the query box; what each library said, in the person's words; keeping its own query and results across a reload |
| own | `choose` | which copy is offered first, the others shown; the item's address within its Jellyfin (jellyfin-web's own route to an item) |
| plumbing | `find` | which lends there are and of what kind, whose each is, which did not answer; reading anything svrngs keeps; building a lend's address |
| plumbing | `ask` | anything about who is asking: headers naming a client, device or token; telling svrngs's refusals (a lender away, a pass refused) from the library's own answer |
| plumbing | `open` | getting the chosen lend's origin opened — a pass there, svrngs's start page, the group's page — and a way back from it |
| plumbing | `signin` | signing the person in to a library, in any form |
| plant | `<its name>` | a line that exists only for one of §3's plants: counted apart, in neither side |

A line before any mark in its file is `unmarked`; the count is
could-not-judge until there are none. A region doing two pieces' work is
split until each does one. The count is this, run from the repository's
root (watched on a planted sample before this commit: three files, one
under `check/`, gave `3 own cards`, `3 own match`, `1 plant asleep-empty`,
`1 plumbing find`, `2 unmarked`, each as the rule says):

    find apps/movie-night -path apps/movie-night/check -prune -o -type f \
      \( -name '*.js' -o -name '*.mjs' -o -name '*.html' -o -name '*.css' \) -print \
      | sort | xargs awk '
      FNR == 1 { piece = "unmarked" }
      /^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*mn: (own|plumbing|plant) [a-z-]+/ {
        sub(/^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*/, ""); piece = $2 " " $3; next }
      /^[[:space:]]*($|\/\/|\/\*|\*|<!--)/ { next }
      { n[piece]++ }
      END { for (p in n) print n[p], p }' | sort -k2

**Each plumbing piece is a candidate for svrngs**, and mn-1 writes each
with its lines (file:line), what svrngs would carry in its place, and the
evidence; the operator queues what is made (the queue's preamble).

**Measured, not fixed, beside the count:**

- **What the door's limit counts while people browse.** A person is
  counted on a lend for every request in flight and for `WINDOW`, 30 s,
  after their last ends (`crates/svrngs-node/src/door/limit.rs:17-19`,
  `:35-38`, `:92-96`), unless their path ends at a private address and they
  are guessed in the lender's home, uncounted (`:21-26`, `:49-65`;
  `crates/svrngs-node/src/door.rs:806-810`). From each lender's door log,
  `door: let in, and told their share` with its `people`
  (`door.rs:837-840`): how many each Jellyfin counted while people only
  searched, how long each searcher stayed counted after Movie Night's
  last request, and whether each was guessed home.
- **`reads` on a computer**: what a page at Movie Night's origin on a
  computer can discover, ask and open (§2), each step's answer recorded,
  the first that stops it named.

## 5. Predictions, 2026-10-08 21:22 UTC, before any app or run

- **Bar 1: passes on a phone in both engines.** Jellyfin's search does not
  forgive `bladruner` (its `searchTerm` matches by substring), so Movie
  Night asks each library for its whole list of films once and matches in
  the page. Jellyfin reads the provider ids from the `.nfo`s, so the two
  copies are one card. Sent no `authorization`, Jellyfin answers each
  request as the person the door signed in, so the kept film is in Ana's
  and Cleo's results and not in Ben's or Dara's. First card within 10 s of
  the query on a phone, cold — the dial and the sign-in's first ask making
  each person a Jellyfin account — and under 1 s for a second query.
- **Bar 2: fails for a phone that never opened that Jellyfin from the
  list; passes after it has.** Never opened, the item's address is the
  static host's 404, and the best an app can do at HEAD — svrngs's start
  page's `for`/`back`, or the group's page's `#open=` — lands on "Open this
  from your group's page" or at Jellyfin's home, not the item (§2). Opened
  once from the list, the item opens signed in, and Play plays within
  30 s. On a computer it opens directly, signed in, and plays.
- **Bar 3: passes, with one or two presses of Back** (jellyfin-web's
  playing page is a history entry after its item page); Movie Night keeps
  its query and results in its own history entry, so its lines are own,
  `cards`.
- **Bar 4: passes in the first case, fails in the second.** Asleep after
  the list, Movie Night's request to Cy's lend ends `502` from its worker
  after iroh gives up its dial, twice (`web/lend.js:300-309`), and Movie
  Night says so; the time is unknown, predicted 10 to 60 s. Asleep before
  the list, Movie Night cannot know Cy exists — the handoff carries no
  `unanswered` (§2) — so it says nothing of Cy: not an empty library, but
  not "did not answer" either. The spirit holds (nothing is shown empty);
  the letter fails, and the gap is `find`'s.
- **Bar 5: fails in its letter on a phone, by discovery alone.** To find
  the Jellyfins Movie Night reads `pass <its label>` from `svrngs-lend`,
  holding the pass and its holder key in its memory though it sends them
  nowhere, and builds each lend's address from its label as `baseOf` does:
  both are §4's `find`, both are what bar 5 forbids. No byte its page
  received holds a lender's key or token, nor the person's own token, and
  its source sets no credential.
- **The count: own about 200 lines (120 to 300), plumbing about 45 (20 to
  80)** — `find` about 25 (the store read, the labels, the addresses,
  whose each is), `ask` about 10 (svrngs's `502` and `403` told from
  Jellyfin's own), `open` about 10 (svrngs's start page and `back`),
  `signin` none, the door doing it. Over 80, svrngs carries less of the
  three operations than §2 reads.
- **The limit while browsing.** On a phone, through the relay, each person
  who searches is counted on every library searched, for 30 s after Movie
  Night's last request — the same as one watching — so two people
  searching fill two places on each Jellyfin; nobody is refused, since
  each lend's limit is above two. A computer whose dial goes direct to a
  private address is guessed home and not counted.
- **`reads` on a computer: stops at discovery.** No lend is handed to a
  page there and the record API refuses it (§2). Handed the labels by the
  run as a stand-in for discovery, said and counted `find`, Movie Night's
  requests reach each Jellyfin as Alex, `--reads` or not; whether the
  browser lets it read them rests on Jellyfin answering a cross-origin
  read, which Jellyfin's `CorsHosts` setting governs and which this host's
  scratch Jellyfin is predicted to allow. Opening the item there passes.
- **The product: untouched.** `git diff 37b4a4c..HEAD -- crates/*/src web`
  is empty at the spike's end, and `scripts/check.sh` names no Jellyfin.

## 6. Out

Each waits for the proof above, as the survey orders it ("the smallest
proof first"):

- five libraries, and different cuts of one film kept apart;
- a lend at its limit, and what Movie Night says of it;
- a source gone between search and open, and another copy offered;
- recommendations, shelves and "what should we watch tonight?";
- a shared index, a background index or a backend of any kind; a
  per-person index beyond what one page holds in memory;
- the survey's cross-check, the same three operations serving a second,
  non-media composition (Mealie and Grocy);
- a person on a computer and a phone being one person (the middleware
  spike's follow-up 3, `demo/runs/mw/2026-10-08/README.md` §7).
