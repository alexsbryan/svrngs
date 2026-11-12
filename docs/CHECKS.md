# Check the contract at its smallest real boundary

The default edit loop runs the actual decision/formatting code with explicit
inputs. Start a browser, native process, relay or service only when that mechanism
is part of the claim. A fixture supplies observations or inputs; it does not
reimplement authorization. Fast presentation tests are not signing/I/O evidence.

## Commands and ownership

| Check | Command | Required mechanism |
|---|---|---|
| One UI contract | `node --test web/tests/creator-consent.test.mjs` | Node, actual shared formatter |
| All isolated Node laws | `node web/tests/run.mjs unit` | Node; source files, crypto and bounded shell/Git checks; no browser, native build or service |
| Node resource/browser integration | `node web/tests/run.mjs integration` | Actual Redis/Lua persistence, expiry and restart; actual WebKit/CSS application |
| Seek judgment calibration | `cargo test -p svrngs-node --test seek_judgment` | Actual pure Rust instrument, explicit recorded measurements; no browser/host sampling |
| One real creator workflow | `cargo test -p svrngs --test scenes creator -- --test-threads=1` | Native publication/signing/version routes and computer/phone browsers |
| Complete regression gate | `scripts/check.sh` | Every tier, before shipping |

`--fast` keeps its static checks and isolated Node laws. The complete gate also
runs Node integration and the apps' tests. `--touched --files <path>...` is a
conservative regression selector, not an excuse to erase another session's work.
Unknown reach still runs the complete set. Shared instrument changes are held to
all their consumers. The whole gate remains the gate.

## Structural classification

Node tests that require a browser/service use `*.integration.test.mjs`.
Other `*.test.mjs` files under `web/tests/` and `resolver/` are isolated laws. `web/tests/run.mjs`
owns discovery, classification and execution; the touched planner reads its
integration catalog rather than maintaining a second list. `apps/` tests remain
the apps' own tier. Unknown tier names and empty selections report never-ran and
fail. The union of unit and integration must equal the previous complete Node
test set, with no overlap or missing cases.
The already-isolated `resolver/adapter.test.mjs` is now included in the unit
catalog; the old page-laws glob had not covered it.

## Extraction ledger — implemented and verified

| Previous location / claim | New owner | What must be preserved |
|---|---|---|
| `browser/acts.rs::what_acts_as_you_is_said_in_one_wording` | Direct question contract | All carrying/reading/Stop/selected-lend/decider outputs; independent expected dates |
| `browser/opens.rs::the_open_says_what_an_app_does_in_one_wording` | Direct question contract | Four exact examples; both pages call the shared formatter |
| `browser/main.rs::the_index_s_row_is_judged_once_for_both_pages` | Direct question contract | Full/on results; neither page owns competing full/sign-in rules; both call opens |
| `browser/main.rs::a_lends_alias_never_collides_within_a_group` | Direct question contract | Namesake/device distinctions, one carried copy, UTC+14/UTC-11 equality and all nine exact/ambiguous lookup results |
| `browser/main.rs::film_seeks_by_load` | `seek_judgment` Rust target | All three original calibration tests, including missing measurements, unknown load, 5,000 ms boundary and actual refusal/abstention distinction |
| Isolated phrase/vector, provider-failure and rejected-ingress cases from resolver tests | Direct resolver contract | Original assertions and actual signature bytes; rejected ingress makes zero store calls |
| Resolver lifecycle/consumption/limits/expiry/restart | Node integration | Actual Redis and original Lua, concurrent races, before/after write faults, AOF restart and timeout behavior |
| Page server stylesheet check | Node integration | Real HTTP content type and WebKit computed style; not merely a string or stub |

Keep these integration checks: actual HTML build/CSP, native ingress/signing,
per-person accepted code, real folder replication, withdrawals, resource limits,
long-lived connection closure, media playback and browser-origin behavior.
`#[test]` alone does not imply purity: publication's test restarts real processes.
Exact writer/reader row text formerly repeated in two native workflows now has
independent golden assertions in the direct question contract. The native cases
still compare their actual DOM to the shared formatter, plus all their original
signing, reach, response and ownership observations. Copy edits have one golden
home without removing the real rendering check.

## Verification contract

Run the migrated contracts before retiring their original copies. Keep every
original assertion and all legitimate controls. Observe the moved contracts fail
on a relevant source fault, and the tier classifier fail on a misclassified
integration suite; compilation failure or zero tests is not that witness. Check
the touched planner's unit/integration/helper/service reach. Measure isolated
and service-backed runs separately. Complete the full gate once after those
focused checks pass; do not use it as the wording edit loop.

The first extraction is recorded in `demo/runs/check-tiers/2026-11-11/`:
isolated Node laws 0.652 s, real Node integration 15.822 s, warm Rust calibration
1.418 s, and the complete gate exit 0 in 1,794.463 s. The unit tier also passed
with Redis absent from PATH and an unavailable Playwright browser installation.
These are individual timing observations. The record names the compiled/source
faults that the migrated checks caught before the full gate passed.
