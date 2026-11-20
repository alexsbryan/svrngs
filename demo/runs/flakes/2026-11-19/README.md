# Playwright's WebKit server dies in `navigations::` (2026-11-19)

Playwright 1.59.1's server throws its own assertion,
`WKPage._onTargetCreated` (playwright-core/lib/server/webkit/wkPage.js:331,
`assert(!this._provisionalPage)`): WebKit opens a second provisional page
while one is pending, in `navigations::a_navigation_from_another_lend_s_page_is_judged_as_its_fetch`,
a chain of cross-origin redirects ([crash.txt](crash.txt)). The server is
shared by the step, so every later row loses its browser (`ECONNREFUSED`),
33 of 45 failed.

It is on origin/main, not the relay-commons merge's. `navigations::` ten
times at each commit, in svrngs-gate from a plain clone
([nav-bisect.sh](nav-bisect.sh), [nav-bisect.log](nav-bisect.log)):

| commit | | crashed |
|---|---|---|
| a6f091c0 | origin/main, V1.12.2 | 2 of 10 |
| c2c16692 | the merge, before the redesigns | 3 of 10 |
| b20f2536 | the merge's head | 1 of 10 |

At b20f2536 also: the whole browser step alone 3 of 3 passed, and
`navigations::` 1 of 5 crashed ([browser-loop.log](browser-loop.log)); in
the full gate, 2 of 2 crashed at the step's 10th row ([gates.txt](gates.txt)).
The audit saw it once at a2661814 (../../relay-commons/AUDIT.md, run 1).
About one run of the test in five; more in the full gate, on a host loaded
by other work.

## Contained: its own browser

No Playwright fixes it yet: the race is
[microsoft/playwright#43067](https://github.com/microsoft/playwright/issues/43067),
open, WebKit on Linux swapping a tab's process twice in one cross-site
redirect chain once the tab's browser has been to those sites; its fix,
#43233, was closed unmerged, blocked on playwright-browsers#2644. 1.65.1,
the latest (2026-11-13), still asserts (`lib/coreBundle.js`,
`assert(!this._provisionalPage)`). The chain is one navigation, a hop into
a lend's origin, which the test exists to judge, so the test cannot step
around it and keep its claim.

So `navigate.mjs` drives a browser of its own (page.mjs `launch`, `own`):
the crash ends its driver, and its row fails alone. In svrngs-gate,
[contain.sh](contain.sh), [contain.log](contain.log):

| run | result |
|---|---|
| `navigations::` ×40, its own browser | 1 crashed, 39 passed; that run, its `webkit` row alone failed |
| the step ×2, `navigate.mjs`'s WebKit driver killed (the plant) | 41 passed, 1 failed, each: that row |
| the step ×3 | 42 passed, each |

Before, on the shared server: 7 crashes in 35 runs of `navigations::`, each
failing both engines' rows, and 33 of 45 rows in the full gate.

Not landed: the harness starting the shared server again when it finds it
dead ([contain.diff](contain.diff)). It saves no row: the step starts
nearly every row's driver within its first 20 s, so the shared server's
death fails every row then driving it. Killed at 20 s, 14 and 16 of 42
failed without it, 7 and 19 with it.
