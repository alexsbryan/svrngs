# js-read close, 2026-11-03

Part (e), after parts (a)–(d) and the closing gate. The operator's ready
diff is in `ralph/next/js/NEEDS_HUMAN.md`, "The js jubilee". Only that
watching session lands the cap (CHARTER "Answered" 2).

## The queue's total, by row

These are counts of the tree at each row's close, not an attribution of
every changed line to that row: main's concurrent work was merged into
the lanes along the way. The initial count is STATE.md's opening
HUMAN-js-agreed record, 2,233. The remaining counts were remeasured from
`git ls-tree -r --name-only <ref>` and `git show <ref>:<path>`, applying
`scripts/lean_check.py:426-442`'s JavaScript suffixes, exclusions and
nonblank, noncomment line rule.

| Boundary | Commit | `js` | Change from preceding boundary |
|---|---|---:|---:|
| HUMAN-js-agreed | STATE.md's recorded baseline | 2,233 | — |
| js-format | 8e085041 | 5,472 | +3,239 |
| js-types | 5a088a49 | 6,039 | +567 |
| js-webkit | f32ecacd | 6,039 | 0 |
| js-lint | 4b7160c0 | 6,535 | +496 |
| js-read | 8e00b6e8 | 7,338 | +803 |
| **Queue total** | | | **+5,105** |

Part (e) adds only Markdown: before and after, `python3 scripts/lean_check.py`
exits 0 and prints:

```text
crates/svrngs-core     3477 / 3478 lines
.                     18861 / 18862 lines
js                     7338 lines, no cap until the js queue's jubilee
swift                     0 / 1 lines
```

## Closing gate

The loop ran the foreground-equivalent command below after js-read-16
requested `ralph-result await`. It returned exit 0 at 8e00b6e8. Full log:
`/Users/alexsbryan/dev/svrngs/ralph/next/js/ctl/log/sessions/js-read-16.await.log`.
The loop reported 23m04s, matching the log's 1,384 seconds. The excerpts
below preserve the gate's verdicts, including its abstention.

```text
$ lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh
note       open files: 256, raised to 10240
note       kept awake while this runs (caffeinate, pid 53431)
scripts/check.sh: child setpgid (53433 to 53433): Operation not permitted
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (0 s)
passed     trust floor (1 s)
passed     advisories (2 s)
passed     app nouns gate (0 s)
passed     app nouns (0 s)
passed     lint (1 s)
passed     format (0 s)
passed     page laws (1 s)
passed     rustfmt (0 s)
passed     clippy (1 s)
passed     tests (490 s)
passed     canaries (2 s)
passed     ceiling canary (5 s)
passed     web rustfmt and clippy (12 s)
passed     browser (182 s)
passed     types (2 s)
passed     typed lint (0 s)
passed     joining in a browser (86 s)
passed     pages (99 s)
passed     scenes (339 s)
passed     scene events (chromium)
could-not-judge scene doc (chromium): {"do":"has","texts":["Saturday's dinner","Bo brings chips","Cy brings drinks"],"who":"alex","within":15000}: not within its bound, as the doc's run says: {
passed     scene give (chromium)
passed     scene films (chromium)
passed     scene give_move (chromium)
passed     scene give_gone (chromium)
passed     scene grapevine (chromium)
passed     scene house_rules (chromium)
passed     scene money (chromium)
passed     scene life (chromium)
passed     scene pantry (chromium)
passed     scene ports (chromium)
passed     scene redial (alone)
passed     scene night (chromium)
passed     conformance canary (114 s)
```

The intervening conformance detail (log lines 41–70) reports each
computer and wildcard capability passed, including the seeking rate.
The final two lines are:

```text
passed     app laws (47 s)
note       took 1384 s in all
```

The gate ran the unchanged page laws (including `parts.test.mjs`),
browser tests, joining tests, pages and scenes. No test is changed by
part (e). The cap's positive control remains the watching session's
landing step, spelled out beside the ready diff.

## The watching session's jubilee

After js-read merged, the watching session applied CHARTER "Answered" 2's
preauthorized cap. JavaScript has no uncapped mode now. Removing that mode
cuts three Python lines; the non-JavaScript cap lowers with that cut.

The clean tree first passed:

```text
$ python3 scripts/lean_check.py
crates/svrngs-core     3477 / 3478 lines
.                     18858 / 18859 lines
js                     7338 / 7339 lines
swift                     0 / 1 lines
exit 0
```

Plant: a new, untracked `web/page/jubilee-plant.js` containing exactly one
line, `export const jubileePlant = true;`. The same command failed:

```text
crates/svrngs-core     3477 / 3478 lines
.                     18858 / 18859 lines
js                     7339 / 7339 lines
swift                     0 / 1 lines
BROKEN  js has 7339 lines of code, budget 7339: raised only by the operator, with the reason in the commit
exit 1
```

The plant was deleted. `python3 scripts/lean_check.py` returned to the
first output and exit 0; `git diff --check` also exited 0. Thus the new
cap was watched failing, and a JavaScript addition moved only its own
counter. These runs change neither the preceding gate's recorded
abstention nor the closing audit's outstanding instrument requirement.

## Focused doc-scene follow-up

The watching session reran the one scene that the full gate could not
judge, on main after js-read merged and the jubilee landed at 35bd78e6:

```text
$ ulimit -Sn 10240 && CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs --test scenes doc:: -- --test-threads=1 --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.29s
running 1 test
test doc::doc::chromium ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 32.56s
exit 0
```

`target/scenes/doc-chromium/` then contained alex/, bo/, doc/ and keep/,
and no could-not-judge or never-ran marker. This is a separate passing
run; it does not rewrite the original gate's load-related abstention.
