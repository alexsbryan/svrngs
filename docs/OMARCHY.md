# svrngs and Omarchy: proposed changes, for review

Status: proposal, 2026-11-06. Nothing here is approved and nothing here has
landed. All of it is outside the order of work in `docs/PLAN.md` §1, which has
no Omarchy row, so under `AGENTS.md` it waits unless the operator adds it.

The operator asked for this review because the work had grown into changes to
the whole system on the strength of one use case. That concern is correct. The
code on the branches was written before any design was agreed, which is the
wrong order (`AGENTS.md`, "Work is the next unit"). The operator's earlier
answers on the Wi-Fi dialog, `new`/`link` and the budget were given before this
document existed, so they are treated as withdrawn until this is reviewed.

## 1. The short version

Testing svrngs on Omarchy found four kinds of thing. They should not be judged
together.

| # | Change | Where the need comes from | Recommendation |
|---|---|---|---|
| A | Rails forms fail through any lend | General bug: every Rails app, on any OS | Land |
| B | Live updates never start on a computer | General bug: every Action Cable app, on any OS | Land |
| C | Install line and "Open this folder" hang | General bug: Linux desktops where `xdg-open` stays in the foreground | Land if the budget allows; see the cost |
| D | Port 80 mishandled | Narrow: only an app lent on port 80, such as ONCE's proxy | Hold |
| E | Wi-Fi lend blocked by the firewall | Omarchy's firewall default; a change to what svrngs does to a computer | Smallest option only; hold the rest |
| F | `svrngs new` and `link` while running | A plan for an agent skill that does not exist yet | Hold |
| G | Bar plugin, install entry, agent skill | Omarchy only | Outside core; no decision needed now |

If only one line is read: **A and B are the case for changing core. Nothing
else here needs to change core now.** An earlier recommendation to have svrngs
run `pkexec ufw` is withdrawn as a recommendation; section 4 gives both sides.

## 2. What was tested

Omarchy 4.5.0 (`omarchy 4.5.0-3`, kernel 7.2.15) from the published ISO,
checksum verified, unattended install, in QEMU on the Fedora box. Three VMs:
two computers and a router, on a virtual LAN. Scripts and images are in
`~/omarchy-lab` there, not in this repository. If any of this is relied on, the
runs belong in `demo/runs/`.

| Measured | Result |
|---|---|
| Install line, browser already open | 4 s to join |
| Install line, browser closed | Hangs until the browser closes |
| svrngs back after a reboot | 14 s |
| A person's name reaches a lent app | Yes, on a computer and in a phone-sized browser through the public page |
| Stock Rails 8.1 scaffold form, v1.6 | `POST 422` |
| The same form, A applied | `POST 200`; `POST 302` in the phone-sized browser |
| Django admin login, the control | `POST 200` before and after |
| Campfire live message, v1.6 | Never delivered, 8 sockets, no subscribe |
| Campfire live message, A, B and D applied, CSP disabled | 224, 76, 81 ms, 1 socket on the sender |
| A TV-like device reaching a Wi-Fi lend, stock firewall | Blocked, `[UFW BLOCK]` in the log |
| The same, one rule for this network and port | `200` in 2 ms, survives a reboot |
| 300 MB between two computers on the LAN, stock firewall | 124-138 MB/s, 0 MB through the router |
| The same, firewall off, one run | 156 MB/s |
| Two computers, LAN up, internet down, 40 s | Never connected, with the firewall on or off |

Branch `omarchy-lend-fixes`, in the gate image on the Fedora box: `cargo fmt`
and clippy pass, door tests 53 of 53, door unit tests 20 of 20, each fix's
plant watched red. 17 page laws and one `home` test fail there, the same 17 by
name and the same test on unpatched `cf38fa6f`; why they fail there was not
looked into.

**Limits of this evidence.**
- One run of each measurement, in VMs. One run is not a rate (`AGENTS.md` 7).
- No real Wi-Fi, no real TV, no real lid close.
- WebKit was not run, and the full gate has not run on any of this.
- A, B and D were exercised end to end on v1.6, then ported. They have not been
  run end to end at the current commit.
- C has a test that has never been run. Only the bug was reproduced.
- No real person has used any of it.
- One unexplained reading: Cy showed as away while running. Sixteen later
  readings were clean. It is neither a bug nor ruled out.

## 3. The three general bugs

### A. Rails forms fail through any lend

**Problem.** The door tells a service two things that disagree. `tell_origin`
sends `X-Forwarded-Host` as the browser's lend address, added for Jellyfin's
sign-in in `1efa9a2d` (2026-10-21). `own_origin` rewrites `Origin` to
`http://localhost:<port>`, added for Paperless in `20868eeb` (2026-10-05). Rails
builds its base URL from the first and compares it with the second, so every
form post is refused. Each change was right for its own app; nothing tested
them together.

**Who is affected.** Every Rails app lent on any OS. North star 2 says anything
on localhost can be shared. A stock `rails new` app cannot take a form post.

**Change.** Send `X-Forwarded-*` only to lends with `sign_in_with_svrngs`, the
case it was added for. Client-typed copies are still removed for every lend.
About 5 lines fewer than today, because a branch that could never run goes too.

**Risk.** This changes a request header for every lend that does not sign
people in.
- Evidence it is safe: the catalogue run (`demo/runs/catalogue/2026-10-05/`)
  predates `X-Forwarded-Host`, and Home Assistant, Immich, Nextcloud, Actual and
  Paperless all passed without it.
- That run has not been repeated. Those five are the ones to rerun.
- An app that builds absolute URLs from a trusted proxy header, in a plain lend,
  now sees `localhost`. Campfire's join link does.
- A Rails app that also signs people in with svrngs keeps the bug. This does
  not fix it, and nothing here does.

**Alternatives.**
- Leave `Origin` alone when forwarding. Rejected: it breaks Paperless, which is
  why the rewrite exists.
- Per-app handling by `--kind rails`. Rejected: per-app glue.
- Do nothing. Every Rails developer must turn off a security check by hand.

**Recommendation.** Land, after rerunning the catalogue's five apps. Open
question 1 asks whether that rerun is required first.

### B. Live updates never start on a computer

**Problem.** `session-client.js` replaces `WebSocket` with a wrapper whose state
constants are not enumerable. Action Cable finds a socket's state with
`for (state in adapters.WebSocket)`, so it never sees the socket as open. It
opens socket after socket and never subscribes. Pages load normally and live
updates silently never arrive.

**Who is affected.** Every Action Cable app opened on a computer, including
anything using Hotwire's Turbo Streams over it, on any OS. `web/shim.js`, used
on phones, sets the same constants with `Object.assign`, so phones should be
unaffected. That was read, not tested.

**Change.** `enumerable: true` on two lines. Zero net lines, plus a law in
`web/tests/`.

**Risk.** Very low. It makes the wrapper match a browser's own `WebSocket`.

**Recommendation.** Land. It is the clearest change here.

### C. The install line and "Open this folder" hang

**Problem.** `svrngs open` waits on `xdg-open`, which stays in the foreground
while a browser it started runs. The install line appears stuck until the
browser closes, and its last lines never print. "Open this folder" does the
same inside a home page request, for as long as the file manager stays open.
The folder hang was reproduced with `xdg-open` by hand, not through the button.

**Who is affected.** Any Linux desktop where `xdg-open` behaves this way. This
was seen on Omarchy only. Other desktops were not tried.

**Change.** A shared `opened()` in `home.rs` starts the opener, gives it one
second to refuse, and does not wait further. Its output goes nowhere, so a
browser cannot hold a pipe open.

**Cost, not small.**
- About 11 net lines, which goes over the budget (section 6).
- Every `svrngs open` and folder press takes a fixed extra second, on Macs too,
  which do not have this bug.
- That second blocks a request's thread, as the current code does for longer.

**Alternatives.**
- Fix only the install line in `install.sh`. Smaller, but it leaves the folder
  button hanging.
- Do the wait on Linux only. Removes the Mac cost, adds a line or two.
- Do nothing, and have the install line print its last lines before opening the
  browser. Worth pricing; it may be two lines and leaves the folder bug.

**Recommendation.** Land some version, but open question 3 should settle which.
The version on the branch is not obviously the smallest good one.

## 4. The changes that come from Omarchy

### D. Port 80

**Problem.** `Origin` goes out as `http://localhost:80`, and `LendPath::parse`
only recognizes `http://host:port`. An app on port 80 fails posts and redirects
the visitor to their own computer.

**Who is affected.** Only apps lent on port 80, in practice ONCE's proxy.

**Why hold.** Its only caller today is a Campfire demo. Campfire also needs
`DISABLE_CONTENT_SECURITY_POLICY=true`, shows a `localhost` join link and is
one product. Its line cost is about 4 and it is a separate commit, so it can
wait for a real need. If ONCE matters to the Omarchy pitch, the operator can
overrule this.

### E. The Wi-Fi lend

**Problem.** On default Omarchy, incoming connections are denied. "Open it to
this WiFi" shows an address, and the TV can never reach it. The page says it is
open when it is not, which goes against `AGENTS.md` 6.

**Who is affected.** This is the honest weakness of the proposal. Omarchy turns
this firewall on by default. How other distributions behave was not tested, and
the belief that most do not block this is unverified. Group members' computers
are not affected: they connect directly under the stock firewall. Only a device
like a TV is.

There are three sizes of answer.

| Size | What it does | Core change | Decides for the person |
|---|---|---|---|
| E1. Docs | A line in `docs/guide/tv.md` and troubleshooting: the rule to add | None | Nothing |
| E2. Say so | The page and `svrngs room` say the firewall may stop this, and give the line to paste | Reads firewall state, adds words | Nothing |
| E3. Ask | svrngs runs `pkexec ufw allow` so the system's password dialog appears | E2 plus a new capability | Asks to change the firewall |

**The case for E3.** It was tested and works: Omarchy's own themed dialog
appears from a plain background service, naming the rule. A person a bit afraid
of terminals gets one press. The rule is as narrow as it can be, one network
and one port. Cancel leaves nothing behind.

**The case against E3, which is why it is no longer recommended.**
- svrngs would become a program that changes a computer's firewall. It has never
  done anything like that. `docs/guide/start.md` tells people the line "changes
  none of your computer's settings"; this would not break that sentence, but it
  moves away from what it promises.
- It depends on `/etc/ufw/user.rules`, an internal file of another program. It
  is world-readable on Omarchy. Whether that holds elsewhere was not checked.
- It would be one firewall's code in core. The operator has ruled out per-app
  glue, and this is the same kind of narrowing, applied to a system tool.
- A prompt nobody answers waits forever, so it needs careful handling.
- It changes what can reach a person, which is the trust floor's subject
  (`scripts/trust_floor.txt`) and the operator's alone.

**A middle path.** E3's behavior could live only in the Omarchy plugin, which is
already allowed to be Omarchy-specific and is outside core. Core would stay at
E1 or E2. The plugin can read `/v1/room`, and runs in the session.

**Recommendation.** E1 now. E2 only if the operator agrees the page must not
overclaim, knowing it means reading firewall state in core. E3 in core: no. E3
in the plugin: decide with G.

### F. `svrngs new` and `link` while svrngs runs

**Problem.** Both refuse with "stop it, then run this again", and the installer
always leaves svrngs running. The home page's forms work.

**Why it is not a bug.** `docs/DESIGN.md:686` says this is the design: they are
"the page's alone while it runs". A test pins the refusal
(`crates/svrngs/tests/address.rs`). The confusing part is the message, which
tells people to stop a service they never started.

**Why hold.** The driver is an agent skill that does not exist yet. Reversing a
written design decision for a hypothetical caller is exactly what this review
is meant to catch. DESIGN §2, rule 3: "Build what the next demo needs; everything
else waits for its trigger."

**Smallest useful change, if any.** Change the refusal's words to point at the
home page. A one-line change that reverses nothing.

**Recommendation.** Hold the behavior. Consider the words.

### G. The plugin, install entry and agent skill

These need no change to core. The plugin runs today against `/v1/groups` and
`/v1/lends`, with no Rust changes. It must live in its own repository, because
the marketplace requires `manifest.json` at the repository root. It needs a
confirm step before lending: in testing, one Enter on the default row lent a
browser's debug port.

No decision is needed now. The question for later is whether exposure on
Omarchy is worth a separate repository to maintain. The marketplace holds about
6,300 entries, and plugins in the same niche showed 0 or 1 stars.

## 5. Against the principles

| Change | North star test | DESIGN rule at stake | Verdict |
|---|---|---|---|
| A, B | 2: makes "if you can run it on localhost" true for more apps | None; they fix bugs | Passes |
| C | 1: a person's first ten seconds | None | Passes, at a cost |
| D | 2, for one class of app | §2 rule 3 | Waits for a trigger |
| E1 | Neutral | None | Passes |
| E2 | 2: the page stops overclaiming | §1 "the most general surface with the fewest features" | Operator's call |
| E3 | Helps 1 for one distribution | §1 "the system learns by adding apps, never by adding platform" | Fails in core |
| F | Helps 2 for a caller that does not exist | `DESIGN.md:686`; §2 rule 3 | Waits for a trigger |
| G | Outside core | None | No core question |

## 6. Budget

`scripts/lean_check.py` fails when a count reaches its cap, so `.` must stay
below 20,455, which leaves room for 1 more line on main today.

| State | `.` lines | Under the cap |
|---|---|---|
| Main, `7232aadd` | 20,453 | Yes |
| A, B, D, branch `omarchy-lend-fixes` | 20,452 | Yes, 1 fewer than main |
| A, B, D, C, branch `omarchy-rest` | 20,463 | No, over by 9 |
| E2, E3, F | Unwritten; a guess of 40-60 more | No |

A and B alone need no raise. C needs a cut or a raise, and `AGENTS.md` requires
an economizing pass before a raise is asked. Nothing here asks for one.

## 7. Open questions for the operator

1. Must the catalogue's five apps be rerun before A lands? This document says
   yes. It costs a run but A changes a header for every plain lend.
2. Is A's known cost acceptable: apps that print their own address show
   `localhost` in a plain lend?
3. For C: the shared `opened()`, an `install.sh`-only fix, or Linux-only waiting?
4. Should the page say the firewall may block a Wi-Fi lend (E2), knowing that
   means core reads firewall state?
5. Is Omarchy work wanted at all before the next unit in PLAN §1? If yes, it
   needs a row. If no, A and B can still land as bug fixes.
6. Should the Omarchy runs be recorded in `demo/runs/`? `AGENTS.md` 4 and 7 ask
   for claims to rest on runs a reader can find.

## 8. What exists now

Nothing below is on `main`, and nothing is pushed.

| Item | Where | State |
|---|---|---|
| A, D, B | Branch `omarchy-lend-fixes` at `325c73b9`, worktree `~/dev/svrngs-lanes/omarchy-lend-fixes` | Three commits, tested as in section 2, not rebased on `7232aadd`, commit bodies lack their evidence |
| C | Branch `omarchy-rest` at `64036dc4`, worktree `~/dev/svrngs-lanes/omarchy-rest` | One commit, on top of the above, test never run |
| E, F | Nowhere | No code written |
| Plugin | `~/omarchy-lab/plugin` on the Fedora box | Prototype, no confirm step, unpublished |
| Test VMs | `~/omarchy-lab` on the Fedora box | Disposable |

If A and B are approved, B and A can be taken from the branch without D. If this
review rejects the work, the branches can simply be deleted. No shipped behavior
depends on any of it.
