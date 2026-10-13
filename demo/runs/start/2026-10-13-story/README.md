# A friend's model, lent, from start to answer — 2026-10-13

Row start-run (`ralph/next/start/STATE.md`): the getting-started story run
on this host, on what start-address, start-llama, start-home and
start-group landed. Sam, with no link, gets svrngs by the founder's one
line, makes the group on the home page and lends llama-server; Bo joins by
the link with the one line, opens the friend's model from the home page
and chats with it in Chromium and WebKit, then gives a program the address
the home page shows, before and after Bo's svrngs starts again; Cy, on a
phone, opens the chat through the group's page; three people at once
against the lend's limit and llama-server's `--parallel`; what the lend
keeps back asked by Bo. Every step timed. start-doc writes the newcomer's
document from this run.

This file's first commit (4caed70) held a setup and predictions written
before start-group, when someone with no link had no line and Sam was set
up by hand. start-group gave the founder's line and "Make a group" on the
home page (63dcaee, 0f468a3, 2b2a751), so §1-2 are written again here for
the story as it now runs, in a commit before any data. Every person is a
program: no people bar is judged (AGENTS.md 7). One host, one run: not a
rate.

## 1. Setup

`W=<scratch> python3 demo/runs/start/2026-10-13-story/run.py build`, then
`PY=<venv with openai>/bin/python W=<scratch> python3
demo/runs/start/2026-10-13-story/run.py`, data in `data/`. The build is its
own step so neither outlasts one sitting of the driver's shell; the story
refuses a site built from another commit than its own.

- **The page and the tarball**, as `demo/runs/c/dry-2026-10-12-rerun/`
  served them: `scripts/build-web.sh` and `scripts/build-release.sh --out
  <W>/site`, built from this row's commit, served on this host by
  `web/tests/serve.mjs` at `http://localhost:<port>/`, nothing published.
  The darwin-aarch64 tarball checked against the `SHA256SUMS` beside it.
- **llama-server** as start-measure started it: Homebrew's, version 10180
  (11b068d06), `-hf Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M --host
  127.0.0.1 --port 18089 -c 4096 --parallel 2`, on Sam's computer. Its
  answers are not judged.
- **Each person** a `HOME` of their own, `USER` their name, a program
  named `open` first on their `PATH` that writes what it was asked to open
  and opens nothing (the rerun's stub).
- **One LaunchAgent label on one host.** install.sh keeps svrngs at login
  as the LaunchAgent `sh.svrnme.svrngs`, one per login; Sam's line and
  Bo's both make it here. The run refuses to start if one of that label is
  loaded already. Once Sam's line has run and Sam's home page answered,
  Sam's svrngs is booted out of launchd and run by the driver as the
  LaunchAgent runs it — the bare command, the same data dir, so the same
  address — which frees the label for Bo's line (the rerun's `take_over`).
  It is taken over before the group is made, not after Sam lends (as this
  file first had it): launchd starts its programs with the login's own
  `HOME`, not the run's, so the first attempt's launchd-run svrngs put
  Lab's folder in this host's own home (§3). Bo's stays in launchd; it
  makes no folder of its own, and is booted out at the end.
- **Sam, with no link, by the founder's line.** The group's page opened
  with no link, in Chromium as a computer; the line it offers run in `sh`
  with Sam's `HOME`, as pasted. Then Sam's home page, at the address the
  line opened, in Chromium: "Make a group" with "Lab" and "Sam"; then "Make
  a link to invite someone to Lab", and the link it shows, which is Bo's.
- **Sam lends llama-server**, in a terminal, as the home page's "Done in a
  terminal" says: `svrngs share 18089` first, as typed from the page's
  words, then what its words say to type next. svrngs typed by the path
  install.sh put it at, `~/.local/bin/svrngs`, as install.sh says to when
  that is not on the `PATH`.
- **Bo, by the one line.** Sam's link opened in Chromium as a computer;
  the line the page shows run in `sh` with Bo's `HOME`, as pasted. It
  installs, joins and keeps svrngs running at login.
- **Bo's chat**: Bo's home page in Chromium, then in WebKit, each on a
  computer; "Open Sam’s llama-server" clicked; in the tab it opens,
  llama-server's own chat page, "Name three rivers in Spain." typed and
  sent; its answer read until it stops growing.
- **Bo's program**: Python's `openai` 3.29.0, in a venv of its own, given
  as its `base_url` the address the home page's "Copy its address for a
  program" put on Chromium's clipboard, nothing added to it. First as a
  newcomer would write it, with no `api_key`; then with any `api_key`
  ("none"); a chat streamed. Then **Bo's svrngs restarted** as launchd
  restarts it, `launchctl kickstart -k`, and the same program at the same
  address again, and the home page at its address.
- **What the lend keeps back**, asked from Bo's computer at Bo's address
  for the lend: each route the well-known table keeps back of
  llama-server (`crates/svrngs/src/well_known.txt`, its `llama-server`
  row), and `/health`, `/props` and `/v1/models`, which it does not.
- **Cy's phone**: WebKit as an iPhone (`web/tests/page.mjs`), Sam's link
  opened, the name asked typed as "Cy", Sam's llama-server pressed on the
  group's page, and in llama-server's chat page the same question.
- **Three people at once**: Cy's phone and two more, Dee's and Eve's,
  joined the same way, each in llama-server's chat page; each sends a
  longer question at the same moment ("Tell a long story about a river
  and a mill."). Phones, since on one host the door counts no computer
  (`demo/runs/c/dry-2026-10-12-rerun/` row 9): a computer whose path ends
  at a private address is guessed in the lender's home
  (`crates/svrngs-node/src/door/limit.rs:55`, `guess`). **First with no
  limit**, as `share` leaves it. Then, after the door's 30 s window
  (`limit.rs:39`, `WINDOW`) with Eve's chat page closed, **Sam sets the
  limit to 2 on Sam's home page** (the box and "Let in this many"); Cy and
  Dee send again, then Eve opens Sam's llama-server from the group's page
  and sends. Bo's program asks once beside them.
- **Each step timed** from its first act, in ms; screenshots in
  `data/shots/`.

## 2. Predictions

1. **Sam, with no link**: the page opened with no link offers, within
   10 s, the founder's line, `curl -fsSL <page>install.sh | sh -s --
   '<page>'`, with the words "To make your group, paste this line in a
   terminal on your computer: …". The line exits 0 within 60 s, joining
   nothing; it says "svrngs runs now, and whenever you log in", and last
   the home page's address, "the same address each time svrngs starts";
   `open` asked for that address.
2. **Sam's home page**: with no group it says "This computer is in no
   group yet. Make one below, …". "Make a group", Lab and Sam: within 5 s
   it says "You made Lab. Make a link below to invite someone to it." and
   "Lab’s folder is on this computer at <HOME>/svrngs/Lab", which exists.
   The link it then makes is on the page's address,
   `http://localhost:<port>/#<token>`, said with "It opens in a browser,
   which says how to join from there."
3. **Sam lends**: `svrngs share 18089` exits 1 saying what it needs, a
   kind, and naming `llama-server` among the kinds it knows (18089 is no
   well-known port); `svrngs share 18089 --kind llama-server` exits 0
   within 2 s: "Lent llama-server (port 18089) to everyone in your groups,
   as many at once as want it", with how to let fewer in, and naming what
   it keeps back. Its words carry "port", a word fence 3 keeps off the
   home page: the terminal's words, said as found, not judged.
4. **Bo, by the one line**: the line shown within 10 s of the link
   opening; the line exits 0 within 30 s, its last words the home page's
   address, "the same address each time svrngs starts"; `open` asked for
   that address; the home page lists "Open Sam’s llama-server" within 5 s
   of the line's exit.
5. **Bo's chat, Chromium and WebKit**: llama-server's chat page in a tab
   of its own within 5 s of the click; the answer's first words within
   10 s of sending, the answer whole within 30 s; no request of the chat
   page's own refused by svrngs.
6. **Bo's program**: the address copied is the lend's,
   `http://<label>.localhost:<port>/`. With no `api_key`, `openai` raises
   before it sends anything, naming `api_key` (could-not-reach: no word of
   the home page's says a program may need one). With any `api_key`, the
   address as copied answers, streamed (llama-server answers
   `/chat/completions` beside `/v1/chat/completions`).
7. **After the restart**: Bo's home page and the lend's address are the
   same strings; the home page answers at its address, and the program,
   unchanged, is answered streamed at the address it was given, within
   30 s of the restart.
8. **Kept back**: each route the table keeps back answers 403 from
   svrngs, `x-svrngs-refused: path`, "llama-server is lent without
   <path>"; `/health`, `/props` and `/v1/models` answer 200.
9. **Cy's phone**: the name asked within 10 s of the link opening; "You’re
   in Lab, as Cy" within 15 s; Sam's llama-server listed; its chat page
   through the group's page within 30 s of the press; the answer's first
   words within 15 s of sending, streamed through the phone's page.
10. **Three at once, no limit**: all three let in and answered. With
    `--parallel 2`, two answers start within 2 s of sending and the
    third's first words wait for a slot, seconds behind; the third is told
    nothing of why it waits.
11. **The limit set on Sam's home page**: it then says "llama-server: up
    to 2 people at once"; Cy and Dee answered; Eve refused, 503, by the
    door's words, "llama-server is lent to 2 people at once, and 2 are
    using it now: try again when one stops. That number is the choice of
    the person who lends it, and they can change it." (`door.rs:1118`) —
    at Eve's first request, the chat page itself, so Eve never reaches
    the box to type in. Eve's phone shows the door's answer as it comes,
    its JSON: the words reach Eve inside braces, not as a page. Bo's
    program, beside them, is answered: guessed home, not counted.
12. **Two homes, and a phone on cellular**: could-not-judge here, the
    operator's (CHARTER.md); the commands written below, after the run.
13. **Every people bar**: never-ran.

## 3. What the run gave

`W=/tmp/start-story python3 run.py build` then `PY=/tmp/sm-py/bin/python
W=/tmp/start-story python3 run.py`, at 2eb5d36, exit 0; the site built
there (`events.jsonl`, `start`: `built.head` 2eb5d36, `dirty` only this
`data/`), served at `http://localhost:63714/`, the darwin-aarch64 tarball
13,160,864 bytes matching its sum (`served`). The host's 1-minute load
was 17.0 at the start and 9.0 at the end, on 12 cores (`start`, `end`).
Every time below is ms from the step's first act, in `summary.json`;
screenshots in `data/shots/`.

**The first attempt** (at 0a6365b, `data/first-attempt.out`, its token
redacted) stopped on three faults of the driver's own, fixed in 2eb5d36
before this run: launchd started Sam's svrngs with this host's own `HOME`,
so "Make a group" put Lab's folder at `/Users/alexsbryan/svrngs/Lab`
(removed); the driver read only a page's first 600 characters, so "You
made Lab." was never seen; and it asked the home page as a bare program,
which the page refuses, `403 {"error":"this page answers only itself"}`
(home.rs `only_itself`), then raised on that answer. Its other steps,
from the founder's line to the program before the restart, gave the same
verdicts as the run below.

Each prediction, by AGENTS.md 6's verdicts:

1. **Sam, with no link: passed, with a finding.** The page offered the
   founder's line at 771 ms, `curl -fsSL http://localhost:63714/install.sh
   | sh -s -- 'http://localhost:63714/'`, with "To make your group, paste
   this line in a terminal on your computer: …" (`sam.page`). The line
   exited 0 in 1,366 ms, joined nothing, said "svrngs runs now, and
   whenever you log in" and last "Your home page, the same address each
   time svrngs starts: http://svrngs.localhost:63742/"; `open` was asked
   for that address at 1,359 ms (`sam.line`), and it answered 200 under
   launchd (`sam_launchd_home_status`). **The finding**: above the line
   the page says, in red, "Open this page from your group’s link.", and
   below it shows a name box with "What’s your name? Everyone in sees
   it.", "Join", "Yes" and "Not now" (`shots/sam-page.png`). The same
   boxes show under Bo's line (`bo.shown.said`, `shots/bo-line.png`).
   Both forms are `hidden` in the page, and its style `form { display:
   grid; }` (`web/index.html:53`) shows them anyway.
2. **Sam's home page: passed.** With no group it said "This computer is
   in no group yet. Make one below, …" (`shots/sam-home-none.png`). "Make
   a group", Lab and Sam: "You made Lab. Make a link below to invite
   someone to it." 320 ms after the press, and "Lab’s folder is on this
   computer at /tmp/start-story/sam/svrngs/Lab", which exists
   (`sam.make`, `sam.folder_is_dir`). The link came 274 ms after its
   button, `http://localhost:63714/#<the token>`, said with "It opens in
   a browser, which says how to join from there." (`sam.make.link_said`);
   the page kept is the one svrngs came from (`sam.origin_kept`).
3. **Sam lends: passed.** `~/.local/bin/svrngs share 18089`: exit 1, "say
   what is on port 18089: `svrngs share 18089 --kind <kind>`, its own name
   if it is one svrngs knows: jellyfin, navidrome, calibre-web,
   llama-server". With `--kind llama-server`: exit 0 in 8 ms, "Lent
   llama-server (port 18089) to everyone in your groups, as many at once
   as want it. To let fewer in at once, change it on your home page, …",
   naming the nine routes it keeps back (`sam.share`). Its last line asks
   for a password, "If llama-server takes a password on each request,
   paste what it takes in its Authorization header, then press Enter
   (Enter alone for none):" — for a lender of a llama-server started with
   none, one more question to answer; said, not judged.
4. **Bo, by the one line: passed.** The line shown at 301 ms; exit 0 in
   2,406 ms, "You're in Lab, as Bo.", the folder, and last the home
   page's address with "the same address each time svrngs starts"; `open`
   asked for it at 2,398 ms (`bo.line`, `data/bo-line.out`). Bo's home
   page listed "Open Sam’s llama-server" 292 ms after Chromium started,
   which was when the line ended (`bo.chat_chromium.listed`).
5. **Bo's chat: passed in what Bo sees; failed in its letter.**
   Chromium: the tab at 98 ms after the click, the box at 881, the
   answer's first words 27 ms after sending and whole 922 ms after it.
   WebKit: the tab 332 ms after the click, first words 105 ms after
   sending, whole 1,006 ms after (`bo.chat_<engine>.ask`;
   `shots/bo-chat-*.png`). But the prediction said no request of the chat
   page's own refused by svrngs, and in each engine three were: `GET
   /tools` twice and `POST /v1/streams/lookup`, 403 `x-svrngs-refused:
   path` (`tab_seen`; `data/sam.log`, "door: refused — the lend refuses
   this path", `refused="/tools"` and `refused="/v1/streams"`). Both are
   in the table's llama-server row; the chat page showed nothing of them
   and answered.
6. **Bo's program: passed.** The address copied was the lend's,
   `http://<label>.localhost:63811/`, the same as Bo's `/v1/lends` gives
   (`program.copy.clipboard`, `program.lends`). With no `api_key`,
   `openai` 3.29.0 raised before sending, "Missing credentials. Please
   pass an `api_key`, …" — could-not-reach, as predicted: no word of the
   home page's says a program may need one. With `api_key="none"`: 22
   chunks streamed, the first at 138.1 ms, all in 250.8 ms
   (`program.first`).
7. **After the restart: passed.** `launchctl kickstart -k` exit 0; Bo's
   svrngs started twice in its log (`grep -c "Leave this running"` 2); the
   home page answered 200, "Your groups", 445 ms after, at the same
   address, `http://svrngs.localhost:63811/`; the lend's address the same
   string; the program, unchanged, streamed 22 chunks, the first at 64.1
   ms, answered 1,433 ms after the restart began (`restart`).
8. **Kept back: passed.** All nine routes of the table's row answered 403,
   `x-svrngs-refused: path`, "llama-server is lent without <path>";
   `/health`, `/props` and `/v1/models` answered 200 (`kept_back`).
9. **Cy's phone: failed.** The name asked 1,003 ms after the link opened,
   "You’re in Lab, as Cy" at 2,896 ms, Sam's llama-server listed, and its
   address reached 1,004 ms after the press — and there the page said only
   "Error: gzip is not supported by this browser" (`phones.cy`,
   `shots/cy-chat.png`). llama-server answered `GET /` 415 (`phones.cy.seen`).
   Straight at a llama-server of the same build, `GET /` with no
   `Accept-Encoding` answers `415 Unsupported Media Type`, and with
   `Accept-Encoding: gzip` 200 `text/html` (`data/gzip-probe.out`). The
   phone's page carries a lend's request with the headers its service
   worker sees (`web/lend.js:306-307`), and a browser never shows a page
   `Accept-Encoding`, so no phone gets llama-server's chat page. The door
   let Cy in, counted, through the relay ("door: let in", `person="Cy"
   people=1 relayed=Some(true)`, `data/sam.log`). Dee and Eve met the same
   (`phones.dee`, `phones.eve`).
10. **Three at once, no limit: could-not-judge.** No phone had a box to
    type in (`phones.none.each`: "no box to type in: Error: gzip is not
    supported by this browser").
11. **The limit set on Sam's home page: passed in part, the rest
    could-not-judge.** After "Let in this many" with 2, Sam's page said
    "llama-server: up to 2 people at once, 0 using it now." 548 ms after
    the press (`phones.limit`, `shots/sam-limit.png`). Bo's program,
    beside, was answered, 22 chunks (`phones.two.bo`), and the door logged
    no refusal for being full (`door.full` 0). Whether Eve is refused at
    the limit could not be judged: Cy and Dee were using nothing, so Eve
    was let in as one of 1 (`person="Eve" people=1`) and met the same
    415.
12. **Two homes, and a phone on cellular: could-not-judge**, the
    operator's (§5).
13. **Every people bar: never-ran.**

## 4. What it found, for rows after this one

- **A phone cannot open llama-server's chat page** (prediction 9). The
  computer's half of the story ran whole, from no link to an answer, by
  the page and two lines typed; the phone's half stops at the chat page.
  What would fix it is the phone page's transport asking for gzip and
  unpacking it, in `web/lend.js` — product, so not this row's.
- **The group's page shows its hidden forms** on a computer (prediction
  1): a name box, "Join", "Yes" and "Not now", under a red "Open this
  page from your group’s link." beside the founder's line, which tells
  someone with no link to do what they cannot. `web/index.html:53`.
- **The chat page asks for two routes svrngs keeps back**, `/tools` and
  `/v1/streams/lookup`, on every load (prediction 5). It showed nothing
  for it here; whether a newer llama-server's chat page does is unread.
- **`share` asks the lender a question** about a password (prediction
  3), after saying it is lent.
- **On one host, launchd gives a LaunchAgent the login's `HOME`**: a run
  that keeps several people's svrngs under launchd must take each over
  before it places a folder (§3, the first attempt).

## 5. The operator's: two homes, and a phone on cellular

Not run here (CHARTER.md: a second computer or network, a real phone on
cellular, and publishing are the operator's). With the page published by
`scripts/deploy-page.sh` (the operator's; `<page>` its address, as
`group.svrnme.sh` is named in `crates/svrngs/src/cli.rs:129`):

```
# Sam's computer, at one home: llama-server, then the founder's line
llama-server -hf Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M --host 127.0.0.1 --port 18089 -c 4096 --parallel 2
curl -fsSL <page>install.sh | sh -s -- '<page>'
#   on the home page it opens: Make a group, then Make a link to invite someone
~/.local/bin/svrngs share 18089 --kind llama-server

# Bo's computer, at another home: the link Sam sent, opened, and its line pasted
curl -fsSL <page>install.sh | sh -s -- '<the link>'
#   on the home page it opens: Open Sam’s llama-server; Copy its address for a program

# Cy's phone, on cellular with Wi-Fi off: the link Sam sent, opened in Safari
```

The phone's step waits on the first finding of §4: it meets the same
415 until the phone's page asks for gzip.
