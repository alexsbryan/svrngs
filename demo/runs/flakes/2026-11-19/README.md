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

Not fixed here. What would: a Playwright with the race fixed, or WebKit's
browser per test so a crash costs its own row and not the step's.
