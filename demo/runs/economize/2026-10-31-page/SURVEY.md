# The economizing pass before the page queue's hooks, 2026-10-31

page-scaffold (a) (`ralph/next/page/STATE.md`). The page queue builds v2 of
both pages on a rung of its own (`scripts/lean_check.py`, `v2`, 1,150,
510e4a0e), but its hooks into the rest are counted in `.`: build-web.sh's
`--ui`, build-release.sh's marker refusal, check.sh's `page laws`, `page v2`
and `parity`, home.rs's v2 merge and cli.rs's `--ui`, +14 to +17 with the
rung's own +3 already landed (CHARTER.md, "Estimates"). At `c986f1f9` `.`
stood at **19,114 of 19,115**, one line of room. So the pass came first
(AGENTS.md "Done"), scoped to those files: `home.rs`, `build-web.sh`,
`check.sh`, `build-release.sh`, `lean_check.py` and `cli.rs`. v1's page
files are frozen (PROMPT fence 2) and go whole at the switchover, so none
is cut.

The instruments are give-economize's (`../2026-10-25-give/scripts/`), run
again from there and not copied: `count.py 40` (`data/count-after.txt`),
`windows.py 2` and `3` over the six files (`data/windows-2.txt`,
`data/windows-3.txt`). `count.py`'s total, 19,951, is not lean_check's
19,098: it still counts `docs/diagrams/src/`, which lean_check leaves out
since 963dcdac. The compiler's dead-code pass and the clone report were not
run: the cuts sought are in shell and in one router, and the last passes
found no dead item in these files (give-economize §1).

## What the instruments found

- `windows.py 3`: build-web.sh and build-release.sh open alike (`set -euo
  pipefail`, `cd`, `OUT=`, the `--out` loop), two scripts each run alone;
  cli.rs says `lend, secret, accounts` in two argument lists, and
  `let page = match origin { Some(page) => Some(checked_page(page).await?),
  None => None }` twice (cli.rs:1558, :1592). The last is the next cut if
  a later row needs one: an `async` in a `map` is not one line, so it would
  be a small function, -2 at most.
- `windows.py 2`: the same runs, and argument lists.

So the cuts are again of the kind the instruments do not see: statements
written over several lines that the files around them say in one.

## Cut, behaviour kept

| Commit | Where | What | Lines |
|---|---|---|---:|
| b50592ca | `crates/svrngs-node/src/home.rs` | `JS`, `MODULE`, `QUESTION`, `LOOK`, `ICON` and their four routes as one `served!` table of path, type and body | -5 |
| ca1570b5 | `scripts/build-web.sh` | each page file stamped as it is copied, one `sed` in place of `cp`, `sed -i.bak` and `rm` | -4 |
| 72c0ea8a | `scripts/build-release.sh` | a platform that did not build, and the run's refusal, one `\|\|` list each | -7 |

**-16 in all**, 19,114 to 19,098 (`data/lean-after.txt`), the top of the
row's -13 to -16. Each commit says how its behaviour was watched kept: the
home page's tests by name, the site's 19 sha256 sums equal before and
after, and the release script's output and exit on a shim `cargo`.

Two forms of the home.rs table were tried and undone (b50592ca's body):
a `const` array folded into the router, +7, since rustfmt wraps its long
tuples, and a macro per route, +0, since the routes wrap.

## The cap

The room is now 17, the hooks' top estimate, so the cap stays at 19,115
and is not lowered: every cut line is spent by a hook this queue lands
(CHARTER.md, "Estimates"), and the switchover deletes those hooks. A cap
lowered to the count would ask a raise of the same 17 lines at the next
part. The ratchet holds: a cap one under the count, planted in a scratch
copy (`sed 's/".": 19115/".": 19097/' scripts/lean_check.py >
target/page-scaffold/lean_plant.py`), exits 1:

    .                     19098 / 19097 lines
    BROKEN  . has 19098 lines of code, budget 19097: raised only by the operator, with the reason in the commit
    planted exit 1
    real exit 0

## Not cut

- `check.sh` (55 counted lines): each step is one line already, and its
  awk lines are one statement each.
- `lean_check.py` (93): the rung landed in 510e4a0e at +3, its least.
- build-web.sh's closing `echo`s (`:138-148`, 11 lines) still name
  commonwealth's `svrn mesh grant`. Cutting them changes what the build
  says, so it is not this pass's (DESIGN §4); it is the next place to look
  when a cut must change words, and is said here so it is not lost.
