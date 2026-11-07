# V1.7 release, 7 November 2026

Published from the commit tagged **v1.7**, on the operator's word of
2026-11-07 ("Ok push and release this").

## Scope and source evidence

The preceding release is V1.6 at 75f9fd3b. Its own record stops short:
`../../v1.6/2026-11-03/publish.json` still reads `running` and its log ends
mid-compile, so what `group.svrnme.sh` served before this run was not
recorded by that release. The readback below is this one's.

| Release note | Source or run |
|---|---|
| Keepers first: version 5 is a flag day | `crates/svrngs-node/src/version.rs` `STEPS`; `docs/LENDING_SCOPE.md` §"Updates, migration and mixed versions" |
| Choose which groups can open a lend | `share --group`, `unshare --group` (`crates/svrngs/src/cli.rs`); `home/page/group-choice.js`; dc533bb9, 3a0dd342, 95fa27b5 |
| Existing lends stamped with the groups their computer is in | 4e098499, 7a8af0ac; `docs/LENDING_SCOPE.md`, the operator's decision of 2026-11-06 |
| A removed group's open streams cut within two seconds | 178f214c, bed790e6, 571bcbfd |
| Sign-in and saved entries follow the choice | 496227ad, 4ae0d111, 434f452d |
| Short invitation addresses and the welcome | 96f40310, 26fa9693, 3f29ba16, 61ea3533; `resolver/`, `docs/ONBOARDING.md` |
| One page per group; A at `/?ui=a` | cdba5fb8 |
| A phone joins through the public meeting point and learns its group's | f28caa6f (`../../relay-fallback/2026-11-05/AUDIT.md`), d83507b0 |
| The page keeps asking; the group at home with the internet out | 4bd784f5, d74cf23e |
| A carried app is one label in its group (version 6, read by 5) | 6c6f292d |
| `restart`, `open --print`, the install line, a lost write's words | 382fd565, d881cb65, b2047664, c8c782dc |

The short addresses are served by `join.svrnme.sh` (Vercel project
`svrngs-join`), deployed apart from this page on 2026-11-04 and not
redeployed by this run.

## Validation before packaging

`scripts/check.sh` ran whole at 27f02cd1 (26 min). What it failed on was
fixed in 0e694fc8, and each failed step was rerun green: the browser binary
(the words a computer that did not answer is named by), typed lint, and
`cargo test -p give` with its three scenes once the Podman VM was up. Then
`scripts/check.sh --touched 27f02cd1` ran every slow step 0e694fc8 can reach
(browser, clippy, conformance canary, joining in a browser, pages, scenes,
types, typed lint), each passed, all 14 scenes among them, in 770 s. That
run exited 1 on one line alone: it said the ceiling canary never ran after
skipping it as untouched, a guard the tier lacked, fixed in a2b49e46, which
changes `scripts/check.sh` and no runtime file. The lean caps are the operator's of 2026-11-07
(`.` 20,835, `js` 8,396).

## Publication

To be filled by the publish run.

## Readback

To be filled by the verify run.
