<!-- section: relay-commons after=intro -->
Read this queue's CHARTER.md and STATE.md, and svrn-docs
`rings/current/RELAY_COMMONS.md`, before any work. The operator decided the
design and asked for this queue. Keep marching through the assigned unit;
another step boundary is not a new approval gate. A cap decision is the
operator's under CHARTER, not a reason for a workaround.

**No check.sh before the very end** (operator, 2026-11-18: "I don't want us
to run interminable check.sh runs -- let's just save that for the very end").
- Here, AGENTS.md's "Done" is this queue's final audit.
- Before REVIEW-audit-relay-commons, run no `scripts/check.sh` in any form:
  not the full gate, not `--touched`, and not its tiers by hand.
- Close each unit on its row's own named focused checks.
- The commit hook's `check.sh --fast` is the only check.sh a unit meets.

**Decisions** (operator, 2026-11-18: "Continue", after these were put to
them as recommendations):
- **A spent share stays spent** for the life of that relay connection.
  iroh 1.4.2 cannot remake a relay connection (1b08385c's probe), so a
  timed clear would let a second throttling stall with no words.
  share-refusal removes the minute's timer and `Net::remade`. The refusal
  holds until the connection to the build's relay ends of itself.
- **A row's allowance is its estimate; the caps decide** (operator,
  2026-11-19: "It's unclear what decision is needed from me that can't be
  answered from principles"). Past the estimate, make the economizing pass
  (AGENTS.md "Done"). If `lean_check` holds, land the row, and its commit
  gives a `Size:` paragraph: the counts, what the pass cut, and why the
  rest stays. Stop with `needs-human` only when a cap would not hold.
- **An advisory that blocks commits is judged by reachability** (675b0cc1).
  If none of its affected functions is called (advisory-db
  `[affected.functions]`, then grep the crate that pulls it in and its
  callers), add a `deny.toml` exception whose reason gives that evidence,
  with a review date two weeks out. If one is called, or this cannot be
  shown, stop.
- **Tests are not budgeted** (operator, 2026-11-19: "on test lines of
  code, I don't really care, write as many tests as you want that's not
  part of budgeting"). Write the tests a change needs, on the existing
  harnesses; a row's test-line figure is no limit and no reason to stop.
- **self-host's part (a) is set aside** in
  `ralph/next/relay-commons/ctl/self-host-a.patch` (four files, green, made
  before webrtc-page shared the worktree). `git apply` it first, then
  continue at NEEDS_HUMAN-self-host.md §4.
- **webrtc-page resumes from its patch** (operator, 2026-11-19: "Raise the
  JS cap to 8,668", then "Just raise the cap": e46447fe and
  80d270f2, JavaScript most 8,674, its +56 after self-host's +7). Its page side is set
  aside in `ralph/next/relay-commons/ctl/webrtc-page.patch`, out of the
  worktree while self-host ran: `git apply` it first, then continue at
  NEEDS_HUMAN-webrtc-page.md §3. Fix one bug first, planted: in its
  `web/shim.js`, `setTimeout(done, 3000)` never ends the wait, since `done`
  resolves only once `iceGatheringState` is `complete`. Resolve on the
  timeout unconditionally, so a gathering that never completes still sends
  its offer after 3 s.
- **DESIGN's 60 lines are net growth**, added minus removed.
- **JavaScript gets no raise in advance.** At its cap, stop with the
  measured counts.
- **webrtc-node** (operator, 2026-11-18: "Approved both"): its product
  allowance is +400, and the trust floor's line for the WebRTC peer's
  socket (95f58944) is approved. It needs no further ask; a
  `NEEDS_HUMAN-webrtc-floor.md` is not needed.

**Words a person reads never say "relay"** (DESIGN §1, rule 1:
docs/DESIGN.md:96-100). RELAY_COMMONS's design text said "shared relay";
that was the design note's slip, not a wording to keep. Use the home page's
own words (home-page.js, group.js): our relay is "the public way", a group's
own is "where your group usually meets", and n0's is "its backup".
share-refusal's two strings, before it closes (or the audit's, if it has
closed):
- the door's words (door.rs `spent_words`): "{by} has used its share of the
  public way for now. Play it at full speed on the same Wi-Fi {wifi}, or in
  the svrngs app, soon."
- the lender's line (home-page.js): "This computer's share of the public
  way was used up on {day}; large things you lend wait until it connects
  afresh."

The app builders' guide (docs/guide/build.md) is a developer's, and may say
"relay".

**The UDP decision** (operator, 2026-11-19, on the trust floor's WebRTC
socket; it replaces the unconditional approval of 7b0b36cd): "On a network
the lender has enabled for direct connections, a currently authorized phone
may request one temporary UDP socket bound to the lender's selected private
IPv4 address. Use host-only WebRTC, keep iroh authentication and relay
fallback, and cap outstanding attempts. Close the socket on setup timeout,
session end or idle expiry, or network change. If the network or address is
ineligible, stay on the relay without binding."
- **What is reused.** Room's precedent: network.rs `here` (the network, by
  its router's hardware address) and `pick` (a private IPv4, never a public,
  tunnel or link-local one), and Room's remembered network. Room's app
  access is not turned on.
- **What the guard is.** Network recognition guards against opening by
  accident on a different network. It is not a firewall guarantee, so
  nothing may say it keeps the port "on this Wi-Fi".
- **Where it lands.** webrtc-node closes as built. The gate is the new row
  webrtc-network, before webrtc-page.

**A `git commit … -- ralph/next/relay-commons/STATE.md` you did not make is
the operator's watcher** landing the loop's done marks. The loop's own
commit fails on this host's Node 20, so the watcher commits it from the
toolbox. Leave it alone, and do not inspect processes or `/proc`. Reads
there are refused and end your session (webrtc-network 1).

This is a dedicated worktree off main. Do not import another worktree's
dirty files.
- **Your scope:** this worktree and synthetic identities.
- **Hands off:** the operator's running svrngs and its data,
  `cw-jellyfin`, and the live llama-server on 9799.
- **Never:** deploy, change DNS, buy, push, merge to main, release, or run
  deploy-relay.sh against a real host.
- **Only CHARTER's new dependencies:** iroh 1.4.x and its matching set, and
  str0m 0.25. CSP and the trust floor stay unchanged.

**Do not read `~/.cargo` itself.** That means `~/.cargo/config.toml` and its
like. Every refusal so far was the same command, `ls ~/.cargo/config.toml;
cat ~/.cargo/config.toml`, retried until the session ended (relay-survey 1
and 2: `external_directory (/home/alexbryan/.cargo/*); auto-rejecting`).
- The build uses this repo's own config alone, so there is nothing there to
  find.
- Crate sources are readable at `~/.cargo/registry/src/**`, by this
  worktree's `.opencode/opencode.json`.
- If such a read is refused, vendor the crate into this worktree and read it
  there: `cargo vendor --versioned-dirs target/vendor`.
- `cargo tree`, `cargo metadata` and `cargo info` need no path.
- `~/dev/*`, the svrn-docs spikes among them, is readable.

The spikes in svrn-docs `rings/current/webrtc-spike/` and
`webrtc-transport-spike/` are ours. Port them, citing them in the commit
body. Their findings bind (CHARTER, "Reuse"):
- a fresh data endpoint dials WebRTC alone, then with the relay;
- paths are given up after 3 s;
- host ICE candidates only;
- datagrams go in through a free function, never a wasm-bindgen method;
- iroh never lives in a lent page.

Do not send iroh's patch upstream or file anything with n0.

This host is Fedora:
- **Builds and focused checks:** in the toolbox (`toolbox run -c
  sovereign-vulkan …`), with `CARGO_INCREMENTAL=0` and `TOKIO_WORKER_THREADS=4`
  for thread-heavy suites. The toolbox caps 2048 tasks; read
  `podman stats --no-stream sovereign-vulkan` before reading a thread panic as
  a failure.
- **The browser tiers and the final gate:** in the `svrngs-gate` container
  exactly as `scripts/gate/README.md` gives it.
- **Serialization:** browser tests and the gate run under
  `flock /tmp/svrngs-gate.lock`. This host has `flock`, not BSD `lockf`.
- **Commits:** made inside the toolbox (`toolbox run -c sovereign-vulkan git
  commit -F <file>`), since the host's Node fails the hook. Never
  `--no-verify`.
- **The container and the toolbox share `target/`, and
  `crates/svrngs-node/build.rs` bakes its manifest path into the compiled
  build script** (build.rs:13 reads `src/shelf.txt`): whichever side built
  it last, the other panics `shelf.txt is read: Os { code: 2 }`.
  `touch crates/svrngs-node/build.rs` before switching sides (iroh-1-4,
  793e4827; the final audit needs it after any toolbox build).

<!-- section: done -->
For each intermediate row, done means its assigned focused checks, red and
green evidence, counts and commits, within CHARTER. Do not claim the full
gate passed at an intermediate close. REVIEW-audit-relay-commons must run
the unchanged full scripts/check.sh in the svrngs-gate container, exit 0.
Then run ralph-result done as its own shell command. Long checks use
`ralph-result await <budget> -- <command>`; the loop owns their lifetime. An
operator-only blocker gets a committed package and `ralph-result needs-human`.

**The audit also checks** (director, 2026-11-19):
- **Each trust floor `listen` line's description against the code it pins.**
  `scripts/trust_check.py` matches the bind line alone, so a change in whom
  a listener admits leaves the description stale with the check green
  (13ffd41e found the relay's). Read each line this queue's code reaches.
- **A group whose own relay is down along with ours.** n0's relays are held
  only while none of ours answers (`tiers.rs`); test that a group with its
  own relay, both down, still meets at n0's, and says so on the home page.

**webrtc-page's stall: the director's read of the second session's runs**
(Update: webrtc-page closed it, 36a118cf and 47462fb6. The stall was the
data endpoint garbage-collected mid-fetch, now held in `directs`
(NEEDS_HUMAN-webrtc-page.md section 6), not 1 or 3 below; the mid-film close
falls back over the relay in 3.4-3.6 s. **2 stands, for the audit:** the
wire's per-worker globals.)
(2026-11-19, from target/webrtc-page-browser-all-5.log and -6.log):
1. **The data connection has no relay path, so nothing can fall back.**
   Each failing run's own trace: `page goes direct: [{"kind":"webrtc",
   "rtt_ms":1,"selected":true}]`, one path, no relay. The fetch then sits
   60 s, where with the relay beside it iroh would drop the silent path
   after `PATH_IDLE` (3 s) and carry on. `Direct::dial`'s second dial was
   to add the relay, but iroh 1.4 hands back the first connection and the
   relay never joins it. RELAY_COMMONS design 4 keeps the relay as backup,
   and HUMAN-phone-home's check is a fall-back mid-stream, so fix this
   first. Prove it with the plant that exists for it (`__svrngsCloseWebrtc`,
   the channel closed mid-film): the film must finish over the relay, and
   `paths()` must list a relay path beside the WebRTC one.
2. **Latent: the wire's state is one per worker, not one per attempt.**
   `OUT`, `ARRIVED` and `PEER` in web/src/lib.rs are `thread_local!`s that
   every `direct_start` overwrites, so two attempts in one worker (two lends
   at once, or a re-announce after `letGo` or a new pass clears `directs`)
   send the first one's datagrams to the second one's window. Own them per
   `Direct`: an id the `send` closure captures (a u32 is Send) keying
   thread-local maps, and `go`'s push closure capturing its own sender
   rather than reading `ARRIVED`. The traces show one attempt per run, so
   this is not the stall; it is the next one.
3. **The stall's own cause is open.** The node read fewer than 20 datagrams
   in about 60 s, where a 1 s keep-alive alone gives about 60, so the page's
   datagrams, keep-alives included, stop reaching it. Lead (a) in
   NEEDS_HUMAN-webrtc-page.md, counting at each hop, localizes it. With 1
   fixed, it degrades to a 3 s fall-back rather than a hang, but the direct
   path is the point of the row.

**The audit's gate runs from a plain clone.** In the `svrngs-gate`
container, this worktree's `.git` file points at the main checkout's
`.git/worktrees/`, which `/work` does not mount, so `scripts/trust_check.py`'s
git steps fail and every listener reads as gone (NEEDS_HUMAN-webrtc-page.md
section 6). Run the full `scripts/check.sh` from `git clone` of this branch's head
(with `npm ci --prefix web`), mounted as `/work` as scripts/gate/README.md
gives it, and record that clone's commit.
