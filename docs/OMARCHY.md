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
| E | Wi-Fi lend blocked by the firewall | Omarchy's firewall default | Omarchy layer only; at most one general sentence in core |
| F | `svrngs new` and `link` while running | A plan for an agent skill that does not exist yet | Hold |
| G | Bar plugin, install entry, agent skill | Omarchy only | Omarchy layer; no decision needed now |

If only one line is read: **A and B are the case for changing core. Anything
for Omarchy lives in a thin layer of its own and touches nothing near core**
(section 2). An earlier recommendation to have svrngs run `pkexec ufw` is
withdrawn: that behavior belongs to that layer or to nothing.

## 2. Boundaries

The operator's rule, 2026-11-06: if svrngs builds something for Omarchy, there
can be something Omarchy in a super thin layer, and it touches nothing close to
core.

```
  Omarchy layer         its own repository, its own release, Omarchy's words
  (bar plugin, status script, install entry, firewall prompt, agent skill)
        |
        |  uses only what any program on this computer may use
        v
  The public surface    `svrngs` commands, install.sh, the token file,
                        GET /v1/groups, GET /v1/lends, /v1/room, POST /v1/open
  ----------------------------------------------------------------- the line
  svrngs                svrngs-core, svrngs-node, the CLI, the pages
                        knows no distribution, desktop or firewall
```

**What each side may do.**

| | Omarchy layer | svrngs |
|---|---|---|
| Names Omarchy, Hyprland, Quickshell, `ufw`, `pkexec` | Yes | Never, in code or comments |
| Holds privileged steps, such as a password prompt | Yes, with the person's yes | No |
| Reads another tool's files | Yes | No |
| Adds a route, flag or setting | No | Only for a reason that holds with Omarchy gone |
| Ships with svrngs's release | No | Yes |
| Counts against the size budget | No, as `apps/` does not | Yes |

**The test for any change below the line: would it be justified if Omarchy did
not exist?** Testing on Omarchy can reveal a bug. It cannot be the reason for a
feature. If the layer needs something svrngs lacks, the layer does without it,
or the need is argued as a general one in its own right.

**How the line stays held.** `AGENTS.md` 10 asks for rules that are structural,
not remembered, and the tool exists. `scripts/vocab_check.py`, in
`scripts/check.sh`, refuses any noun listed in an `apps/**/nouns.txt` from infra
code. A list of this layer's nouns (`omarchy`, `hyprland`, `quickshell`, `ufw`,
`pkexec`) makes crossing the line fail the gate. It reads `.rs` files and the
pages but skips comments and does not read `web/install.sh`, so those gaps would
need closing or stating. Today no file in `crates/`, `web/`, `scripts/` or
`apps/` names any of these.

**Where the layer lives.** Its own repository, as `AGENTS.md` 12 says of a chat
adapter. The plugin marketplace requires `manifest.json` at a repository's
root, so it cannot sit in `apps/` here. Open question 7 asks whether a noun list
belongs in this repository without the code.

**Each change against the line.**

| Change | Side | Holds with Omarchy gone? |
|---|---|---|
| A, B, C | svrngs | Yes: general bugs in lends and in opening a browser |
| D | svrngs | Yes, but waits for a need |
| E, a general sentence | svrngs | Yes: true on any computer, and reads nothing |
| E, reading firewall rules | Layer | No, so it may not be in svrngs |
| E, the password prompt | Layer | No, so it may not be in svrngs |
| F | svrngs | Unproven; held |
| G | Layer | No, so it stays in the layer |

## 3. What was tested

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
looked into. Since that run the branch has changed only in comments.

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

## 4. The three general bugs

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
- About 11 net lines, which goes over the budget (section 7).
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

## 5. The changes that come from Omarchy

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

**Under section 2, the Omarchy answer is entirely in the layer.** It needs no
core change, because `/v1/room` already gives any local program each lend's
address and port.

| Part | Lives in | What it does |
|---|---|---|
| E1. Docs | svrngs docs | A line in `docs/guide/tv.md` and troubleshooting, for any computer with a firewall |
| E2. One sentence | svrngs, one wording | The room's words add that this computer's firewall may stop a device. It reads nothing, and is true anywhere |
| E3. Check | Omarchy layer | Reads `/v1/room`, then `/etc/ufw/user.rules`, and shows in the panel if the port is blocked |
| E4. Ask | Omarchy layer | On the person's press, runs `pkexec ufw allow` for this network and port |

**What was tested for E3 and E4.** The rule works from a real LAN device and
survives a reboot. `/etc/ufw/user.rules` is world-readable on Omarchy. `pkexec`
brought up Omarchy's themed password dialog from a background service. Cancel
leaves no rule. A prompt nobody answers waits forever, so the layer must not
block on it. None of this has been run from inside the plugin.

**What is given up.** A press on the home page no longer leads to a password
dialog. Someone without the layer gets the sentence and the docs, and pastes a
line. Their page stops overclaiming, but it cannot know for certain.

**Rejected: firewall code in svrngs.** Reading `ufw` state or running `pkexec`
from svrngs fails section 2 outright. It would also make svrngs a program that
changes a computer's firewall, which it has never done, and `docs/guide/start.md`
tells people the line "changes none of your computer's settings".

**Rejected: an install-time port range rule.** It opens 9,000 ports on every
network and needs `sudo` in the line.

**Recommendation.** E1 now. E2 if the operator wants the page to stop
overclaiming: it is a general sentence, so it passes section 2. E3 and E4 only
in the layer, and only if G goes ahead.

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

This is the Omarchy layer of section 2. The plugin runs today against
`/v1/groups` and `/v1/lends`, with no Rust changes. It needs a confirm step
before lending: in testing, one Enter on the default row lent a browser's debug
port.

Two facts for whoever builds it. `omarchy-webapp-install` overwrites the
`svrngs.desktop` that `install.sh` writes, so the layer must not use that name.
A launcher with a bare URL dead-ends without a session, so it should mint an
address through `POST /v1/open`, which is untested from the layer.

No decision is needed now. The question for later is whether exposure on
Omarchy is worth a separate repository to maintain. The marketplace holds about
6,300 entries, and plugins in the same niche showed 0 or 1 stars.

## 6. Against the principles

| Change | North star test | DESIGN rule at stake | Verdict |
|---|---|---|---|
| A, B | 2: makes "if you can run it on localhost" true for more apps | None; they fix bugs | Passes |
| C | 1: a person's first ten seconds | None | Passes, at a cost |
| D | 2, for one class of app | §2 rule 3 | Waits for a trigger |
| E1 | Neutral | None | Passes |
| E2, a general sentence | 2: the page stops overclaiming | None | Passes |
| E3, E4, in svrngs | Helps 1 for one distribution | §1 "the system learns by adding apps, never by adding platform" | Fails, and section 2 rules it out |
| E3, E4, in the layer | Helps 1 for those who install it | None: outside core | No core question |
| F | Helps 2 for a caller that does not exist | `DESIGN.md:686`; §2 rule 3 | Waits for a trigger |
| G | Outside core | None | No core question |

## 7. Budget

`scripts/lean_check.py` fails when a count reaches its cap, so `.` must stay
below 20,455, which leaves room for 1 more line on main today.

| State | `.` lines | Under the cap |
|---|---|---|
| Main, `7232aadd` | 20,453 | Yes |
| A, B, D, branch `omarchy-lend-fixes` | 20,452 | Yes, 1 fewer than main |
| A, B, D, C, branch `omarchy-rest` | 20,463 | No, over by 9 |
| E1, E2 | About one line of wording | Depends on C |
| E3, E4, G | 0: they live in the layer | Not counted |
| F | Unwritten and held | Not estimated |

A and B alone need no raise. C needs a cut or a raise, and `AGENTS.md` requires
an economizing pass before a raise is asked. Nothing here asks for one.

## 8. Open questions for the operator

1. Must the catalogue's five apps be rerun before A lands? This document says
   yes. It costs a run but A changes a header for every plain lend.
2. Is A's known cost acceptable: apps that print their own address show
   `localhost` in a plain lend?
3. For C: the shared `opened()`, an `install.sh`-only fix, or Linux-only waiting?
4. Should the room's words say a firewall may stop a device (E2)? It reads
   nothing and is true anywhere, but it adds a caveat every person sees.
5. Is Omarchy work wanted at all before the next unit in PLAN §1? If yes, it
   needs a row. If no, A and B can still land as bug fixes.
6. Should the Omarchy runs be recorded in `demo/runs/`? `AGENTS.md` 4 and 7 ask
   for claims to rest on runs a reader can find.
7. Should this repository hold a noun list for a layer whose code lives
   elsewhere, so `scripts/vocab_check.py` can guard the line? If so, where,
   since the check reads `apps/**/nouns.txt`?

## 9. What exists now

A, B, C and D are on `main` and deployed (`90629602`, `334f30ef`, `4eca3fef`,
`f867f78a`). E1 is in `docs/guide/tv.md` and troubleshooting. E2 waits on question 4.
F is held with no code written. E3, E4 and G belong to the layer, outside this
repository; the plugin prototype is `~/omarchy-lab/plugin` on the Fedora box,
unpublished and with no confirm step. The earlier branches and worktrees are gone.
