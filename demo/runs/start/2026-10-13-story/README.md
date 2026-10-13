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
