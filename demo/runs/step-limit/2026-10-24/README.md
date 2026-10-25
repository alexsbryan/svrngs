# capability-step-limit: each step of `scripts/check.sh` has a time limit

capability-what-acts' first gate printed its last line 25 min in and then
nothing for 2h35m in `app laws`, until the loop's 3h budget ended it
(`ralph/next/capability/ctl/log/sessions/capability-what-acts-1.await.log`;
the loop's log, `~/.svrnmesh/ralph/svrngs-svrngs-capability/launchd.log`,
2026-10-24T08:41:26Z: "background run passed its 3h00m budget — ending its
process group"). `app laws` passed in 41 to 59 s in every other gate of
this queue.

## (a) What held `app laws`

In that run neither browser could start from the scenes step on
(`browserType.launch: Timeout 180000ms exceeded`, at lines 445 to 553 of
that log). `web/tests/page.mjs` `launch` then calls `process.exit(3)`, past
each page test's `finally`, so the server the test spawned is never
stopped. Events' server (`apps/events/page.test.mjs`) inherits the test's
stderr, the pipe `node --test` reads, and the runner waits on it for ever.

Reproduced with no browser to start, `PLAYWRIGHT_BROWSERS_PATH` an empty
directory under `target/step-limit/`:

    $ timeout 90 node --test apps/events/page.test.mjs       # at f88a03ab
    exit 124 after 90 s
    $ ps -axo pid,ppid,pgid,command
    27077     1 27073 node apps/events/server.mjs …/config.json
    $ timeout 600 node --test <the 21 files app laws runs>
    # tests 134  # pass 131  # fail 3      (then nothing; exit 124 at 600 s)

Grapevine's page test left its server running the same way (ppid 1), its
stderr piped, so it held nothing open. Both now end their server when the
process exits (2cb7c8a4): exit 1 in 0 s, "Playwright's chromium cannot
start — run: …", no server left. Why neither browser could start in that
run was not reproduced: the next gate, 3 min later, launched both
(`capability-what-acts-2.await.log`, exit 0).

## (b) The plant

Put in for one run as the gate's first step, then taken out: a step that
starts a stand-in server and a Playwright Chromium, then never ends.

    step "plant" sh -c 'cd web && echo "a server, a browser, then nothing"; sleep 100000 & node -e "…chromium.launch(); console.log(\"a browser up, its driver\", process.pid); setInterval(…)"'

What it started, 1 s in (`target/step-limit/watch-plant.sh`):

    52892 52892 sh -c cd web && …
    52895 52892 sleep 100000
    52896 52892 node -e import("playwright")…
    52899 52899 …/chrome-headless-shell-mac-arm64/…   (a process group of its own)

The gate's output, at 303 s:

    failed     plant: still running at its limit of 300 s, so it and all it started were ended; its last output:
               a server, a browser, then nothing
               a browser up, its driver 52896
    passed     principles pinned (0 s)

None of 52892, 52895, 52896 or 52899 was left. The browser, in a group of
its own, closed because its driver was told TERM. Then, with the plant gone,
the gate was sent TERM in its tests step (`target/step-limit/watch-term.sh`).
It exited 1, and the tests step's cargo processes ended with it; the one
process still alive was another repository's test binary
(`eutropia/target/debug/deps/takeover-…`).

## (c) The limits, from this queue's gates

Each step's passing runs in this queue's 14 gates
(`ralph/next/capability/ctl/log/sessions/*.await.log`, 2026-10-23 to
2026-10-24), in seconds:

| step | passing runs | fastest | slowest | limit (5 × slowest) |
|---|---|---|---|---|
| principles pinned | 14 | 0 | 1 | 300 |
| lean and RUST_ARCH | 13 | 0 | 1 | 300 |
| trust floor | 14 | 0 | 1 | 300 |
| app nouns gate | 14 | 0 | 0 | 300 |
| app nouns | 14 | 0 | 1 | 300 |
| rustfmt | 14 | 0 | 1 | 300 |
| clippy | 14 | 1 | 21 | 300 |
| tests | 12 | 225 | 307 | 1535 |
| canaries | 13 | 1 | 6 | 300 |
| ceiling canary | 12 | 3 | 46 | 300 |
| web rustfmt and clippy | 13 | 0 | 14 | 300 |
| browser | 10 | 130 | 246 | 1230 |
| joining in a browser | 11 | 29 | 56 | 300 |
| scenes | 6 | 204 | 544 | 2720 |
| conformance canary | 12 | 87 | 178 | 890 |
| app laws | 12 | 41 | 59 | 300 |

Steps under a minute share one limit, 5 × 60 s. The runs spread up to 2.7
times with the host's load (scenes, 204 to 544 s), so 5 leaves room past
the slowest seen, and the longest limit still ends a hang in about 45 min, not
3 h. The rule and the four named steps live in one place,
`scripts/check.sh`'s `slowest`.

## The close

The first full gate with the limits in took 870 s, inside this queue's
685 to 1261 s, and every step passed in its limit but the ceiling canary
(`ralph/next/capability/ctl/log/sessions/capability-step-limit-1.await.log`):
its three idle rounds read 0.181, 0.034 and 0.376 s of processor against
0.1 s, the third at 0.376 s in 0.048 s of wall. The step's 3 s is its usual
time; what the limits change is how a step is started and ended, not what
it runs. Run again by its own command right after, five times, at a load of 8
to 16: medians 0.0273, 0.0227, 0.0236, 0.0249 and 0.0469 s, all passed, the
first round of each at 0.18 to 0.21 s as in every gate here. The 0.376 s
round did not reproduce, so its cause is not known; it is left to the
ceiling's own reading, not to this row. The gate was then run once more.
