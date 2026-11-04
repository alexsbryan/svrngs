# The economizing pass before give-install's raise, 2026-10-26

give-install adds Give's program as an option of `web/install.sh` and builds
it in `scripts/build-release.sh`. Both are counted in the repository ledger
(`scripts/lean_check.py` CODE takes `.sh`), which stood at 19,107 of 19,109
at 2ce1fb4c. So the pass came first (AGENTS.md "Done"), over the two files
the row touches.

The instruments are give-economize's (`../2026-10-25-give/SURVEY.md`), two
days old. Its §2 found nothing that no check, test or person runs, and the
rows since each made their own cut. This pass read the two files line by
line for statements said in more lines than they need.

## Cut, behaviour kept (the install tests unchanged by name, 9 of 9 passed)

| Where | What | Lines |
|---|---|---|
| `web/install.sh`, the platform refusal | an `if` of two tests and one `no`, as `[ ] && [ ] \|\| no` | -2 |
| `web/install.sh`, the checksum | the sum program chosen once, then one `got=`, in place of an `if` with two `got=` | -3 |
| `web/install.sh`, the checksum refusal | the same `[ ] && [ ] \|\| no` | -2 |
| `web/install.sh`, the relay's names | one `[ -z ] \|\|` for an `if` | -2 |

-7 in all (per lean_check: 19,107 to 19,100), more than the 2 lines `\|\| no`'s
wrapped lines took back.

## Not found

- `scripts/build-release.sh`: each statement already one line, its loop one
  per platform.
- The rest of `install.sh` (the profile, the opener, the open's wait) says
  each step once. The relay's printed unit differs from the user unit in
  four of its lines, so they do not share a heredoc.
- The change itself shares rather than copies: fetching and checking a
  tarball, putting a program in `~/.local/bin`, stopping one kept at login
  and keeping one at login become one function each, run for svrngs and
  for Give's program. Copying them instead would be about 30 lines.
