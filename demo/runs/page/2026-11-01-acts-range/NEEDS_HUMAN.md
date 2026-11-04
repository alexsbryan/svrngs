# page-acts is past its range: widen it, or move (d) on

**The question.** page-acts has landed (a), (b) and (c) at +181 lines in
the `v2` rung, against the row's +108 to +144 (CHARTER, "Estimates"). Fence
11 says a row past its range stops. Widen the row to about +190, so it
finishes (d)'s five fixes and the v2-only checks, or close it at +181 with
(d) moved on?

## Where it stands

`v2` is 809 / 1150 at 9b049558. `.` is 19120, unchanged.

Each file by `lean_check.py`'s count, at the merge base d8009598 and at
HEAD, beside its line in CHARTER's Estimates table:

| Part | File | Lines | Estimate |
|---|---|---|---|
| (a) shared | `ui/acts.js`, `ui/changes.js`, `ui/uses.js` | +5, +11, +7 = +23 | none |
| (b) phone acts | `web/v2/acts.js` | +47 | +38 to +48 |
| (b) phone uses | `web/v2/uses.js` | +23 | +18 to +24 |
| (b) phone ask | `web/v2/ask.js` | +17 | +24 to +30 |
| (b) | `web/v2/enter.js` `dialled` | +1 | none |
| (c) home acts | `home/v2/acts.js` | +49 | +20 to +30 |
| (c) home ask | `home/v2/ask.js` | +21 | +8 to +12 |
| | **the row** | **+181** | **+108 to +144** |

The phone, +88, is inside its +80 to +102. The excess is:
- the shared parts, +23, which the table gives no line;
- the home, +70 against +28 to +42.

## Why the home is over

v1's home spread "What acts as you" through `show` and `uses`, pushing
lines by `add` as each read landed (h:105-107, :24, :118-124, :279-286). In
v2 it is one section derived from state, so it needs:
- its own machine, ACTS, for the /reaching and /choices reads, the yes to
  a new version, the Stop and the folds;
- a way to hear each group's read. GROUP's state reaches only a group's
  places (group.js:46), so `Told` tells HOME as GROUP reads, and acts.js
  `take` keeps it (`told`);
- holding the page's foot until it has read (`holds`). v1 drew `p.address`
  last (h:326-328), and home.mjs opens every `<details>` as `p.address`
  shows (home.mjs:72-75). Without the hold, "Open the new version" sat in
  a closed fold: target/page-acts/green-c.log, first run, "element is not
  visible".

The home ask is a machine of its own too, with its own read of each
group's lends (`lentIn`, h:63-66), since /ask draws no group's section.

## What the economizing pass tried

- The yes to a new version keeps its answer by key on both pages
  (`acting`, `acted`, the done change kept after a read again). Shared from
  `ui/changes.js`, its five lines would replace six: −1.
- Imports, 10 a file, are one module each.
- Every other cut found would join two lines into one, which games the
  count and makes nothing smaller. None was made beyond the first pass's
  (8c8da32a: the carry as one chain, its try/catch gone).

## What is left, and its cost

(d), each its own commit, with its v2-only test in `page_v2/acts.rs` and
`acts.test.mjs`:
- `unsent` carried on each read while not empty: +1 to +2 (acts.js `carry`
  is written to be reused);
- a refused yes keeps `#ask` shown, with its why: +0 to +1;
- the home's uses Stop asks again, in place of the reload: +0;
- `visible` reads `/reaching` again: +0, as ACTS reads on each of HOME's
  rounds already;
- the home's ask refusal said once: +0.

So the row closes at about +183 to +185.

## The options

1. **Widen the row to +190 (recommended).** It finishes (d) and the v2-only
   checks the row promises, then runs `scripts/check.sh`.
   - The rung has 341 left under its cap of 1,150.
   - The switchover's raise, already expected and the operator's, grows by
     about 40.
2. **Close at +181, with (d) moved to page-parity.** The five fixes and
   their tests move, and the row's check runs now.

## Also for the director, already done under its page-join decision

(c) edits two page-open files, each by one line, on lines page-join's
diff does not touch (`group.js:43`, `:46`; `home.js:43`'s `Group` call):
- `home/v2/group.js:26`: PLACES gains `told`.
- `home/v2/home.js:43`: the foot drawn once no feature's file `holds` it.

The director let page-join pass `q.joined` through both files
(demo/runs/page/2026-11-01-join-range/NEEDS_HUMAN.md, "The director
decided"). If these two need their own yes, they are the whole of it.

## Evidence

- `python3 scripts/lean_check.py`: `v2 809 / 1150`, `. 19120 / 19122`.
- Counts per file: `git show <rev>:<file> | grep -v '^\s*$' | grep -v -E '^\s*(//|#|/\*|\*)' | wc -l`
  at d8009598 and 9b049558.
- Red and green for (b) and (c): the commit bodies of 8c8da32a and
  9b049558, logs in `target/page-acts/{red,green}-{b,c}.log`.
