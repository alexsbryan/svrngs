# The N+1 bar's steps on one host — setup and predictions, 2026-10-15

Row v1ui-run (`ralph/next/v1ui/STATE.md`): `ONE_PAGER.md`'s N+1 bar,
"Four of five strangers settle an expense and open a title in the library
within three minutes of joining the chat, with zero accounts and zero key
prompts" (V1_UI_BUILD §7), its steps run on this host with every person a
program. Written before the driver exists and before any data; `README.md`
here will say what the run gave, against each line below. Nothing in the
product changes in this row. A program is not a stranger: **the people
bar never-ran**, the operator's to run with real people (`demo/README.md`).
One host, one run of each: not a rate (AGENTS.md 7).

## 1. Setup

`W=<scratch> python3 demo/runs/v1ui/2026-10-15/run.py`, data in `data/`;
with `PLANT=accounts`, the same story with the one change below, data in
`data/plants/accounts/`. `judge.py` reads a take's `summary.json` and
writes its `verdicts.json`, one verdict a prediction, AGENTS.md 6's four.
scripts/check.sh runs none of this.

- **svrngs as a newcomer gets it.** `cargo build --release -p svrngs` at
  the driver's commit, tarred as `svrngs-darwin-aarch64.tar.gz` beside its
  `SHA256SUMS` under the site's `get/`, as `scripts/build-release.sh` names
  them; this host's platform only (the other three are not fetched here).
  The page built by `scripts/build-web.sh`, served on this host by
  `web/tests/serve.mjs`; nothing published. n0's public relays, the
  default. Every computer a `HOME` and data dir of its own under `W`,
  `USER` its person's name.
- **launchd is played by the driver.** Four computers on one host would
  share install.sh's one LaunchAgent label (`sh.svrnme.svrngs`;
  `demo/runs/start/2026-10-13-story/README.md` §1 took it over by hand
  for two). So a `launchctl` first on each computer's `PATH` writes what
  it was asked and exits 0, and the driver starts what the plist it was
  given names, its `ProgramArguments` with its `SVRNGS_DIR`, the moment
  it is asked: the program launchd would start, not launchd. An `open`
  stub writes the address it was asked to open (start-run's). launchd
  itself is start-run's, measured there.
- **The library.** A scratch Jellyfin, the macOS app's server with its
  own data dirs on 127.0.0.1:8096, as `apps/jellyfin/check/check.sh up`
  makes one: three films, each H.264 in MP4, 120 s, in a library
  "Movies"; its wizard done by the lender's own account. Jellyfin's
  middleware (`apps/jellyfin/mw`, debug) on 8097, given a key from
  Jellyfin's API Keys, as `apps/jellyfin/README.md` says.
- **Before anyone new.** Elm Street is made and furnished; none of this
  is a newcomer's, and none of it is judged:
  - **Sam**, the link's computer: `svrngs new "Elm Street" --as Sam
    --origin <page>`, left running. Its page link, `<page>#<token>`, is
    the chat's link: the one every newcomer opens.
  - **Priya**, a member already: `svrngs join <link> --as Priya`, left
    running. Priya lends the library from a terminal as
    `apps/jellyfin/README.md` says, `svrngs share 8096 --sign-in 8097`.
    On her home page in Chromium she brings what the shelf ticks, Money
    and Events ("Bring the ticked ones to Elm Street"), opens Money from
    the index, and adds "Gas bill", 60, paid by her.
- **Five newcomers, one at a time**, each a program pressing what the
  page shows, in this order: Ann, Bo and Cy on computers; then Di on a
  computer and Eve on a phone, each only joining; then **Sam's computer
  stopped** (its svrngs sent Ctrl-C); then Di's steps, then Eve's. The
  issuer is stopped after the last join because a link is redeemed at
  its issuer (`web/app.js` `join`, `connect_to_join`); after it, Di's
  computer and Eve's phone do everything with it off.
- **A newcomer on a computer** (Chromium as a desktop, a browser each):
  1. *the link*: opens the chat's link; the page's line read from it;
  2. *the line*: that line run in `sh` with their `HOME`, as pasted, its
     stdin closed, so anything it asks is refused, not answered;
  3. *the home page*: the address `open` was asked for, opened;
  4. *Money*: the index's Money row's link pressed; in its tab, "Say you
     were in on it" beside Gas bill, then each "Say you paid …" until
     the page says "You’re square with everyone.";
  5. *the library*: the index's row for Priya's jellyfin pressed; in its
     tab, a film's card pressed, then its Play; done when the film plays
     (its `<video>` past 0.5 s and not paused).
- **Eve, on a phone** (WebKit as an iPhone, `web/tests/page.mjs`): opens
  the chat's link, types "Eve" where the page asks her name, presses
  Join. After Sam's computer is stopped she opens the page again with no
  link, as from her home screen; presses Money, the same presses as a
  computer; then the page again, presses Priya's jellyfin, a film, Play.
- **Each newcomer's clock** starts when they open the link: the chat's
  moment. Each step is timed from its first act, in ms.
- **What each newcomer is shown** is kept at each step: the page's words,
  and whether it holds a password box or Jellyfin's sign-in
  (`#txtManualName`, `.manualLoginForm`, `.btnManual`, as
  `apps/jellyfin/check/person.mjs` reads one). The line's every word is
  kept.
- **After everyone**, each computer's home page is read again (the door
  with Sam's computer stopped), each computer's Money page, and each
  computer's copy of Elm Street's record (`/v1/rail/log`), the acts
  matched to the presses.
- **The plant, `accounts`**: Priya lends Jellyfin from her home page
  instead, as the page offers it today — "Look for programs running on
  this computer", then "Lend jellyfin to everyone in your groups" — with
  no middleware. Everything else the same.

## 2. Predictions

1. **The link, on a computer.** The page shows the line, `curl -fsSL
   <page>install.sh | sh -s -- '<page>#<token>'`, with "On this computer,
   paste this line in a terminal: …", within 5 s of the link opened.
2. **The line.** Exits 0 within 30 s, asking nothing; says "You're in
   Elm Street, as <name>." (the name from `USER`, nothing typed) and last
   the home page's address; `open` asked for it.
3. **The home page, the door.** Within 30 s of the line's end the door
   lists Sam, Priya, and each newcomer joined before, each with a
   computer lit "on now" while it runs, the newcomer included; once Eve
   has joined, Eve "in from a phone", never lit. The index lists a Money
   row and "Open Priya’s jellyfin".
4. **Money, on a computer.** Opened in its tab with Gas bill listed
   within 10 s of the press; "You’re square with everyone." within 30 s
   of the first press after. Each press writes one act, by its presser,
   in Elm Street's record.
5. **The library, on a computer.** Jellyfin's home in its tab, asking no
   sign-in, within 15 s of the press; the film playing within 30 s of
   its card pressed.
6. **Eve joins.** The page asks her name within 10 s of the link opened;
   "You’re in Elm Street" within 30 s of Join.
7. **Eve with the issuer stopped.** The page opened again says "You’re in
   Elm Street (…), as Eve, through <someone else>’s computer" within 15 s
   (V1_UI_BUILD §7, 3.3's bar), never Sam's; its door draws Sam's
   computer off, "did not answer", never lit and never left out.
8. **Eve's Money and library.** As 4 and 5, through another person's
   computer: Money square within 60 s of its press, the film playing
   within 60 s of jellyfin's press.
9. **Di with the issuer stopped.** As 3–5; Di's door draws Sam off.
10. **Within three minutes.** Each of the five has Money square and a
    film playing within 180 s of opening the link; the slowest within
    120 s.
11. **Zero accounts and zero key prompts.** No page any newcomer is shown
    — the link's, the home page, the group's page, Money, Jellyfin's —
    holds a password box or Jellyfin's sign-in; the line asks nothing.
    Jellyfin then holds one account for each of the five, each named by
    the middleware (`<name>-<16 hex>`), none made by a person. Eve types
    one thing, her name; no computer newcomer types anything.
12. **The door after everyone.** On each of the five computers still on,
    Sam drawn off ("did not answer"), Priya, Ann, Bo, Cy and Di lit "on
    now", Eve "in from a phone".
13. **Money agrees everywhere.** Each newcomer settles before the next
    says they were in, so each later share is owed only by its newcomer:
    the last Gas bill split six ways, $10 each. After everyone, Money on
    each of the five computers still on says "You’re square with everyone." and
    "Nobody owes anybody."; each computer's copy of the record holds the
    same Money acts, each matched to its person's press, none unpressed.
14. **Words.** No svrngs page any newcomer reads (the link's, the home
    page, the group's page, Money) says key, member, record, act, hash,
    origin, port or relay (fence 3); Jellyfin's own pages not judged.
15. **The plant `accounts`.** Each newcomer's library step stops at
    Jellyfin's sign-in, with nothing of theirs to type: 5, 8 and 11 fail,
    10 with them; Money's predictions hold.
16. **Not run here.** Real people, a phone on cellular, a second host:
    could-not-judge; **the people bar: never-ran**, the operator's.
