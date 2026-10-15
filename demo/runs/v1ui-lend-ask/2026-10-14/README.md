# v1ui-lend-ask (b): the failure again, the sweep only on request

The browser suite, five times, at 2a95bc2: the home page looks for what
runs here only when its button is pressed (e2f6b30). Each run is
`/tmp/v1ui-lend-ask-run.sh <n>`: `scripts/check.sh`'s own step
(`cargo test --quiet -p svrngs-node --test browser`, open files raised to
10240 as check.sh raises them), with `-- --nocapture` added so the
streaming test's time line reaches the file (2a95bc2). `run-<n>.out` keeps
the load before and after, the exit code, the streaming lines, the result
and any failure.

| run | load before | exit | chromium's 9 MB range | webkit's | suite |
|-----|-------------|------|-----------------------|----------|-------|
| 1 | 24.22 | 0, 75 passed | 24,954 ms | 26,427 ms | 108 s |
| 2 | 50.79 | 0, 75 passed | 25,300 ms | 31,269 ms | 105 s |
| 3 | 51.23 | 0, 75 passed | 5,162 ms | 6,328 ms | 76 s |
| 4 | 38.71 | 0, 75 passed | 11,669 ms | 12,573 ms | 80 s |
| 5 | 43.84 | 0, 75 passed | 3,114 ms | 9,311 ms | 64 s |

Five passed of five, the range within its 60,000 ms bound each time, the
slowest 31,269 ms (run 2, webkit). Beside it, with the sweep on every home
page's load (7007bb1, 0255842): the suite passed 3 of 7 runs, the streaming
test failing "no answer within 60 s" in the other 4. The same test alone,
at load 28.87, took 9,597 and 10,229 ms (2a95bc2's body).

What this does not say. The range's time spreads from 3 s to 31 s across
runs on one host under a load from 24 to 51, so the 60 s bound has room,
but not much more than one slow run's worth. With the sweep removed, nothing
here traces the earlier failures to it rather than to load: those runs had
no time line, so how near 60 s the passing ones came is not known. One
host, one run each: not a rate (AGENTS.md 7).
