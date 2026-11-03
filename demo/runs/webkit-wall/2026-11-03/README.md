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
