# onboarding-scope-ui — scoped run

The UI half of selected-group lending, on `ralph/onboarding-scope-ui`, against
the frozen API in `ralph/next/onboarding/SCOPE_PARALLEL.md`. No cargo, no
`scripts/check.sh` here; the main integration worker merges this branch and
owns the final gate and the real-node scene.

## What landed (three commits)

- `2f155a28` — `home/page/lend-found.js` LEND takes `choose` and requires at
  least one group before `lend`; `home/page/lend-change.js` CHANGE takes
  `choose`/`audience` and posts `{groups}`. The planted law
  (`web/tests/pages/lend-groups.test.mjs`) failed all three cases at HEAD.
- `fd10e333` — HOME's groups thread into the rows; the shared
  `home/page/group-choice.js` `GroupChoice` draws one box a group, told apart
  by `whose(group.made)`; `lentTo` reads `[]` paused / `null` legacy /
  selected names; `question.js` `Lend` gains `groups`; `home.rs`'s served
  table gains `group-choice` on its existing line.
- `85849367` — `web/page/dial.js` `lends` sends `x-svrngs-version`
  (`core.current_version()`) at the phone discovery edge; `docs/DESIGN.md`'s
  home-page UI description updated (lending-policy paragraphs untouched).

## Actual growth (scripts/lean_check.py)

| rung | before (`024e0ca2`) | after | cap |
|---|---:|---:|---:|
| crates/svrngs-core | 3,486 | 3,486 | 3,494 |
| `.` | 19,795 | 19,795 | 19,833 |
| js | 8,187 | 8,300 | **8,301** |
| swift | 0 | 0 | 1 |

js growth is **+113**, within the original combined **+235 JS** guardrail; the
`js` cap moved one past measured (8,301) with its reason and named plants in
`scripts/lean_check.py`. No core and no non-JS cap moved. The `group-choice`
served-module entry rides the existing `home.rs` table line, so `.` is flat.

## Checks run here (green)

- `scripts/lean_check.py` — exits 0; the table above.
- `node --test $(find web/tests -name '*.test.mjs')` — 150 laws pass,
  including the 8 audience laws and the discovery-header law.
- `npx --prefix web oxlint -c .oxlintrc.json <changed files>` — clean.
- `npx --prefix web oxfmt --config .oxfmtrc.json --check <changed files>` —
  all formatted.

Every new law was watched failing on a named plant first:

- LEND with no `choose` (HEAD): `lend-groups.test.mjs` `not ok 1..3`.
- `choose` replacing instead of merging: laws 1 and 3 (`not ok 1`, `not ok 3`).
- `[]` read as legacy in `lentTo`: law 7.
- no maker in the group box: law 8.
- `x-svrngs-version` removed from `dial.js`: `discovery-header.test.mjs`
  `not ok 1`.

## Pending — deferred to final integration (not claimed passing)

- **Types** (`node scripts/types.mjs`): cannot run here — the wasm glue
  `target/web/site/wasm/svrngs_web.d.ts` is built by the backend lane. The
  `core.current_version()` call in `dial.js` needs that declaration, which
  the backend adds (`web/src/lib.rs` `current_version() -> u32`).
- **Browser/native run**: the audience selection, refusal, and audience-only
  save in a real browser are prepared to be driven by the existing generic
  `web/tests/pages/index.mjs` (its `steps` already tick boxes and read the
  line). A Rust case in `crates/svrngs/tests/pages/index.rs` that lends with
  no group (refused), then with House, then changes the audience, is the
  registration to add and run at integration; it needs the backend's
  `/running`+`/lent` group handling, so it is not compiled or run here.
- **Real-node House/Trip denial/withdrawal scene** and the locked full
  `scripts/check.sh`: the main integration worker's.
- **Backend**: `Changing`/`LendChange` in `home.rs` still accepts only
  `Limit|SignIn|Stop`; the `{groups}` arm and the response `groups` field are
  the backend lane's. The UI posts the frozen shape and reads
  `groups: string[] | null`; it never defaults a missing scope to everyone.
