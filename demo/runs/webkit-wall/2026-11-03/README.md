# The shared WebKit stops loading after 64 pages, 2026-11-03

Why the gate's `browser` step went red on this Mac from 2026-11-02 21:27
on (js-webkit (a), `ralph/next/js/STATE.md`). Measured with Playwright
1.59.1 (WebKit 26.4, `webkit-2272`) on node 20.20.2, Darwin 27.0.0,
12 cores, the page code at 5a088a49.

**The cause.** Every page Playwright's WebKit opens on macOS starts AppKit's
window order-in animation, an `NSAnimation` that runs on a thread of
libdispatch's user-interactive pool (`-[NSAnimation _runBlocking]`). That
thread does not come back while the display is off: none of 11 came back
in 15 s, closed contexts and all (`leak.mjs`). At 64 pages
the pool, 64 threads, is full. From then on WebKit's UI process can't run
the reply to its AppSSO check, which it makes before every navigation
(`SOAuthorizationCoordinator::tryAuthorize`, which asks AuthKit's `akd`).
So no navigation in that browser is ever decided again. `goto` waits out
its 30 s on `load`, and so does `about:blank`. Nothing is busy: the
network process, the page servers and the proxy all sit idle.

`harness.rs` `browsers` shares one WebKit across the whole `browser` step
(c4a294fc, 2026-10-25), and the step opens about 70 pages in it. The tests
whose pages open after the 64th time out. Which tests those are follows
from the step's order, so the same four failed in every full run:
`the_page_offers_a_computer_one_line_and_sends_the_token_nowhere`,
`tabs::the_group_s_page_in_two_tabs_reaches_the_group_from_one`,
`the_phone_goes_through_the_link_s_computer_while_it_answers`, and
`the_phone_reaches_whoever_s_awake`, each `::webkit`. Load doesn't move
it: the four failed at load 80-90 (run 1) and at 15-40 (run 2). The step's
extra 70-100 s since js-format-3 is these failures' own waits: the reach
tests wait 120 s for a `ready` file that a stalled page never writes
(`main.rs` `with_stopped`).

**Why it began on 2026-11-02.** The display was last turned off at
21:27:13 that evening (`display.txt`, from `pmset -g log`). js-format-3's
green run, at 161 s, came before that, and every red run since came after
it. An `NSAnimation` is paced by the display, so with the display off it
never finishes. This explanation is could-not-judge: the display was not
turned on to watch the threads come back. The cause above does not depend
on it, and neither does the fix, which removes the animation.

## The runs

- `browser-step.txt`: seven runs of the step, or of its WebKit tests,
  under the gate lock (runs 1-2 the whole step; 3-7 `-- webkit`; 5 without
  `approve.mjs`'s own WebKit, `--skip an_app_s_entry_is_approved`). Every
  one is red, 4-6 WebKit tests each.
- `webkit-log.txt`: run 7 in macOS's unified log (`/usr/bin/log show`).
  The shared WebKit, `Playwright[9522]`, answered its last navigation
  policy at 01:27:37.998. From 01:27:38.077 on, each navigation asks
  `tryAuthorize`, `akd` takes the ask, and no answer arrives. Page 1700
  (`runs.mjs`) asks the policy for its navigation 1707 at 01:27:54.579 and
  is still waiting when it closes at 01:28:24.
- `sample-ui-threads.txt`: `sample` of that UI process while stalled.
  64 threads are on `com.apple.root.user-interactive-qos`, each in
  `-[NSAnimation _runBlocking]`, and the main thread is idle in its run
  loop.
- `wall.mjs` → `run-wall.txt`: one WebKit, pages one after another, each
  in a context of its own that is closed after it loads. **Page 64 does not
  load. With `-NSAutomaticWindowAnimationsEnabled NO`, 80 of 80 load.**
- `leak.mjs` → `run-leak.txt`: threads in `NSAnimation _runBlocking`
  after ten pages, and 15 s later. **11 and 11 with no defaults; 0 and 0
  with `-NSAutomaticWindowAnimationsEnabled NO`; 11 and 11 with
  `-NSScrollAnimationEnabled NO`.**

Playwright refuses a browser argument that does not start with `-`, so
`NO` can't be passed as an argument. Both scripts hand the default to
Playwright.app through a wrapper around `pw_run.sh`. `-NO` and `-0` as
values don't take (11 threads each, `run-leak.txt`).

Ruled out on the way, from the same runs: the proxy (after the stall it
still answered each CONNECT and finished TLS in 2-4 ms, and no request
followed); the page servers (they answered every request that reached
them); name resolution (no name is resolved: every page name goes through
the proxy); and the Mac's load.

To run again: `node wall.mjs`, `node wall.mjs
-NSAutomaticWindowAnimationsEnabled NO`, `node leak.mjs [defaults]`, from
this directory, with `npm ci --prefix web` done.

## The fix, js-webkit (b)

page.mjs `starting` starts WebKit on macOS through `web/tests/webkit.sh`,
which adds `-NSAutomaticWindowAnimationsEnabled NO`. Both places that start
a browser use it: browsers.mjs's shared one and `launch`'s own. The
launcher ends its line with `about:blank`. Playwright.app opens its last
argument that doesn't start with `--` as its first page. A first try ended
on `NO`, and every browser started with a profile waited the full 180 s on
it (four tests, `launchPersistentContext: Timeout 180000ms exceeded`). With
`about:blank`, a profile's browser starts in 581-751 ms against 573-575 ms
without the launcher, on that same first page, and 70 more pages load in it.

`browser-step-fixed.txt`: the step under the gate lock.
- **Plant:** the launcher without the default, so the windows animate again.
  The four are red again, along with
  `carried::a_new_version_runs_as_each_person_only_after_their_yes::webkit`,
  whose page also opened past the 64th; 230.53 s.
- **With the fix:** 94 of 94 at load 7.4 (140.74 s) and at load 13.1
  (141.83 s).
- **Between those two runs, at load 10.5:** 93 of 94. That same `carried`
  test failed, on an assertion and not a timeout. The phone's yes was
  written (`POST /carry answered 200`), Money was opened right after, and
  it ran "Money as brought" where the test wants "Money as kim changed it"
  (`carried.rs:696`). `runs.mjs` `opened` takes any `Money …` title as
  loaded. That race is the test's own and not this stall, and it is left
  as it is: this queue changes no test (its fence 3).

## The `carried` failure, read again (js-webkit's close)

The gate run at 83b372b4 (`lockf -k /tmp/svrngs-gate.lock bash
scripts/check.sh`, 1336 s) failed `browser` on this one test and nothing
else, 93 of 94 in 159 s, again at `carried.rs:696`. So the race is not
rare: 2 of the 4 full steps since (b). And it is not only the test's.

- `carried-gate.txt`: bo's page in that run. Its yes asked
  `POST /runs` of kim's computer and carried the signed line to the
  link's computer, ada's (`POST /carry answered 200`). The next read and
  the open both went to kim's computer, which did not have the line yet.
  So Money ran there as brought.
- `carried-trace.txt`: the nodes' own trace of a solo run that passed,
  with `rails`, `folder` and `svrngs_node::ring_sync` added to
  `harness.rs` `traced`. **Every round took 4003-4054 ms.** Each one
  dials bo's key and waits out the 4 s dial (`exchange failed, so the
  member is unreachable this round peer=ea4a6c63…`). When no round was
  running, bo's line reached the other computer 18 ms after it was
  carried (`chunk=1 … pushed=1`). A nudge only wakes the round after the
  one running (`ring_sync.rs` `spawn_ring_sync_loop`, the comment on
  `notify_one`). So a yes carried while a round runs waits up to 4 s
  before it leaves, and the phone opens Money again sooner than that.
- **Why bo's key is dialled.** The round never dials a browser's key
  (`node.rs` `dialled`, `Device::dialled`). But `tests/browser/main.rs`
  `computer_on` admits `admitted`, which is bo's phone in this test, with a
  bare `Admit`, and a bare `Admit` counts as a computer. Other tests count
  on that: `drew_the_door` pins "bo did not answer" as an asleep computer,
  and pia is its phone (`from_a_phone`). A phone admitted the way a
  phone joins (`from_a_phone`) is never dialled, and no round waits on it.
- What a person would meet: a group where one computer is asleep has the
  same 4 s rounds. A yes or a write made on a phone during one reaches the
  other computers up to 4 s late. Opened at once on one of them, the app
  runs as before. Reopened a little later, it runs as the yes asked.

- **Tried, not kept:** the test's `node(…, &bo)` replaced by `seeded`
  and `from_a_phone(&ada, "bo", 7)`, so bo is admitted as a phone. Solo
  under the gate lock, both engines passed in 27.59 s (51.35 s before),
  and its 10 rounds took 8-70 ms each. That is one run, so not yet a
  rate (AGENTS.md 7).

Not fixed in this queue: the fix is a fixture change (fence 3) or a
change to the node's round, and neither is this row's to choose,
so it is the operator's question (js-webkit-2's package).

## The rate, after (c) (56f8bb29)

`carried-fixture.txt` is (c)'s plant: rounds of 4003-4061 ms with
option-a.diff reverted, 7-44 ms with it applied. Then three `browser` steps
in a row, each `lockf -k /tmp/svrngs-gate.lock cargo test --quiet
--workspace --test browser` at 56f8bb29:

| run | 1-min load (12 cores) | result | time |
|-----|----------------------|--------|------|
| 1 | 7.28 | 94 passed, 0 failed | 136.89 s |
| 2 | 8.99 | 94 passed, 0 failed | 139.52 s |
| 3 | 11.95 | 94 passed, 0 failed | 139.10 s |

3 of 3, with the 1-minute load between 7 and 12.

## The gate after (c): `browser` passed, a `home_group` test did not

js-webkit-4's `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh` at
48d09878 passed `browser` (156 s), `scenes` and every other browser step.
It failed on two counts:

- `rustfmt`, on a line 56f8bb29 added to `carried.rs`; formatted in 28fef69a.
- `tests`, on one test:
  `home_group::the_home_page_joins_another_group_from_its_link_while_svrngs_runs::webkit`
  at `crates/svrngs/tests/home_group.rs:334`. It wanted `["Bring more to Trip"]`
  in what the page said once Join was pressed, and found none.
  `home-group-gate.txt` holds the panic, that read and the page's trace.

What the read shows: "You’re in Trip, as Dee.", Trip's heading with nothing
under it, Lab whole, and "Reading your groups…" under "What acts as you"
(`page/acts.js:439`, drawn while a group is not yet told). So the harness
read the page before Trip was. `web/tests/home.mjs` reads after a press
at the first frame whose words changed and hold no "…" (`settled`). The
trace has `ACTS: read: reading/idle` straight after `HOME: read (2
group(s))`, so ACTS started again from `reading`. A frame between, with no
"…" in it, would explain the early read. These runs do not show which frame
it was.

This race is older than this row. The same test failed in the page queue's
gates, `page-parity-4` (both engines) and `page-parity-5` (Chromium, at
`home_group.rs:335`), before the acts section said a new group was being
read (`page/acts.js:328`). Rerun alone three times under the gate lock at
28fef69a, at loads of 9-11, it passed each time (`home-group-gate.txt`). It
is no `browser` test, and this row changes no page byte. It goes to the
operator (`ralph/next/js/NEEDS_HUMAN.md`) and is not fixed here.
