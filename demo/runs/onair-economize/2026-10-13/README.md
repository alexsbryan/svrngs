# onair-economize: check.sh at the close, its first run

`scripts/check.sh` at a0c3f78, one run, 10:48 wall: every step passed but
`tests`, which failed on one browser test, `the_home_page_says_a_refusal_apart_from_no_answer::chromium`
(crates/svrngs-node/tests/browser/main.rs:2110). Its output is
`check-tests-failure.txt`, cut from the run's log: the home page's list read
as its placeholder, "Asking each computer in the group what it lends…"
(home.js `show`, since 4830603), where the test wants the four computers
named by how they did not say. The same test passed in the run's own
`browser` step, minutes later.

Not reproduced, so nothing was changed for it (AGENTS.md 2):

| How | Runs | Failed |
|---|---:|---:|
| the test alone, both engines (`cargo test -p svrngs-node --test browser -- the_home_page_says_a_refusal_apart_from_no_answer`) | 5 | 0 |
| the same under 16 busy loops on the host (`/tmp/econ-hog.sh 150`) | 6 | 0 |
| the whole browser suite, 61 tests at once | 3 | 0 |

One failure in about 20 runs is not a rate (AGENTS.md 7). What it looks
like: home.mjs reads `all` once the page's load is fired and its first
section is shown (web/tests/home.mjs), and the list is final only when
home.js's top-level await has finished; the read came before it once, under
the full `tests` step's load. This row's changes do not touch `show` or
`/v1/lends` (cd80ad1 changed `/svrngs-reads` and a lend's origin; 60d9b95
home.js `uses` and `ask`, which run after `show`). A follow-up, if it
recurs: home.mjs waiting for the list to leave its placeholder, its plant
the read made at once.

## The second run

`scripts/check.sh` at 78099c6, run whole by the loop (`ralph-result await`),
exited 0 after 10m17s. Its verdicts, from the log
(`ralph/next/onair/ctl/log/sessions/onair-economize-1.await.log`):

```
note       open files: 256, raised to 10240
passed     principles pinned
passed     lean and RUST_ARCH
passed     app nouns gate
passed     app nouns
passed     rustfmt
passed     clippy
passed     tests
passed     canaries
passed     ceiling canary
passed     web rustfmt and clippy
passed     browser
passed     joining in a browser
passed     conformance canary
passed     app laws
```

The test that failed in the first run passed here, under `tests` and
`browser` both: two runs of check.sh, one failure, still not a rate.
