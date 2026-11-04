# Selected-group lending: fresh economy pass

Assigned row: `onboarding-scope-economize`. Source base **7ad807f131d8**,
2026-11-04. This is the pre-implementation survey, not a policy landing.
The two untracked V1.6 publish artifacts were inspected: `publish.json`
names source `75f9fd3b` and state `running`; `publish.log` is the prior build
transcript. They are not this unit's evidence and are neither edited nor staged.

## Measurement and instruments

`python3 demo/runs/economize/2026-11-04-lend-scope/measure.py survey`
records the actual commands and exits in `lean-before.txt`, `windows.txt`,
per-file counts in `counts.json`, and the textual caller inventory in
`callers.txt`. The count imports the existing lean rules, rather than making
a second platform counter. Give is separately measured with the same
nonblank/noncomment rule over `apps/give/src/**/*.rs`, test files excluded;
inline cfg bodies count, as they do in the platform instrument.

| Product | Count | Cap | Additional lines fitting strictly below cap |
|---|---:|---:|---:|
| core (subset of non-JS) | 3,486 | 3,494 | **0 authorized for this feature** |
| non-JS `.` | 19,789 | 19,833 | 43 |
| JS | 8,066 | 8,090 | 23 |
| Give Rust src, separately | 2,669 | app, uncapped | not platform savings |
| Swift | 0 | 1 | unrelated |

Lean passed, exit 0. The cap is exclusive: the earlier survey's 44/24
arithmetic differences permit only 43/23 added lines without a cap edit.
The exact-window instrument was rerun unchanged over current node, door,
OIDC and command source. It reports textual windows, not semantic clones.
No semantic-clone measurement was made in this pass: **could-not-judge**, not
zero duplicates; the old pass's claim that the developer executable was
absent is not reused. This row's source/window inventory makes no clone or
noun-acceptance claim and does not replace a later audit's prepared corpus.

## Reuse rechecked at HEAD

Paths below are under `crates/svrngs-node/src/` unless qualified.

| Surface | Current evidence | Smallest extension |
|---|---|---|
| Policy home and parsing | `lend.rs:218-244,282-306,456-487,570-602` | Extend `LendFile` with explicit canonical IDs and parse into legacy/selected audience. Missing is unchosen, empty selected is paused. Preserve `Source::Carried`'s intrinsic group. No policy sidecar. |
| Current membership witness | `door.rs:950-957,1098-1129,1255-1286`; runtime `register.rs:253-275,326-338` | Resolve **within selected groups**, with both lender and device standing, before authorization/choice/identity. Gather all eligible witnesses: conflicting first-person keys refuse; matching identities select deterministically. `Standing::Left` confers no grant. Owner access remains an explicit bypass of audience, not invented membership. |
| Guest witness | `door.rs:1107-1108,1140-1160`; runtime `register.rs:346-354` | Keep live/named/host-owned visit checks and additionally require its actual selected group. Do not take the first excluded visit as the only candidate. |
| All declaration writers | `lend.rs:522-549,556-567,570-638`; `home.rs:402-407,426-457,571-573`; command `cli.rs:1550,1562` | One stable per-name lock spans complete read/modify/write, creation/existence test, secret replacement and withdrawal. `cli.rs:761-774` already uses std `File::try_lock`; no new dependency. Never unlink the lock on withdrawal (would split lock identity). Audience/limit/sign-in edits read latest inside it; avoid nested public locking calls. A visible busy/conflict leaves the grant unchanged. |
| Legacy and downgrade | `lend.rs:283,466,581-602`; node startup `node.rs:306-388` | Before advertising, migrate missing fields to explicit empty, preserving fields/secrets/choices. Failed migration leaves access refused; strict old parser rejects `groups` on downgrade. No automatic current-groups snapshot. |
| HTTP and OIDC front | `door.rs:1231-1343` | Find declaration before choosing a remote membership witness. Scope gate precedes OIDC authorize, stored-choice yes, secret injection and upstream contact. Keep `Who::Own` deliberate. |
| Discovery and transitive lists | `door.rs:518-539`; `node.rs:891-938,1009-1052,1062-1100`; `surface.rs:252-268`; `lend.rs:400-402` | Extend shared listing predicate, distinct audience metadata (not `Offered.group`). Intersect IDs with groups shared with the **final verified recipient**, including intermediary lists; filter before serializing. |
| Mixed versions | `version.rs:31-42`; `node.rs:896,918-933` | Single version decider must advance; caller marker required on `/lends` and `/people/lends`. New lenders refuse unmarked requests, new consumers reject v4 answers; old consumers otherwise accept future versions. Update phone request/cache paths as well as Rust. Marker grants no authority. |
| App context | `door.rs:1525-1542,1623`; `origin.rs:433-518`; `surface.rs` reads; phone `web/page/hand.js` | Filter pages/read/append contexts against current selected/shared IDs. App append validates every act before writing; retain that behavior. An old pass or app-choice yes never restores excluded reach. |
| OIDC back channel | `oidc/provider.rs:154-161,331-388,394-413` | Internal code needs device/group provenance and live recheck at redemption; current code has subject but no membership witness. Userinfo currently checks signature/client/expiry only: add current scoped standing, without changing issuer/client/secret on audience edits. Keep any needed token provenance node-local; no core token format growth. |
| Raw | `door/udp.rs:131-171,229-244`; `door/tcp.rs:35-70` | Reload declaration for admission and sweep, not captured `Lend`. Existing one-second sweep closes a connection dedicated to **one raw lend**. TCP child tasks need cancellation through that connection/owned task set; UDP must stop its per-flow tasks too. |
| Active uses | `door.rs:401-511,669-697,1459-1482,1657-1713` | Reuse task/drop/interval primitives. Counts are capacity accounting and omit home users; never use counted-only holds as the authorization registry. Prototype an independently scheduled guard around response pump and upgrade copy, not a timer polled only when downstream requests a frame. |
| Existing UI | `home/page/lend-found.js:48-90`, `lend-change.js:76-117`, `lent-row.js:23-76`; `home.rs:515-519` | Extend existing machines/forms with explicit selection, saving/refusal and owner pause explanation; use current stable IDs/name disambiguation. Capacity remains a separate choice. |

### Upstream inventory

Rechecked commonwealth-ai HEAD **d8b61abe55d29f573d71ca88fa0b698e2484608c**,
unchanged from LENDING_SCOPE's source survey. Read
`cmnwlth/crates/commonwealth-media/src/identity.rs:41-65,118-147`,
`commonwealth-rails/src/config.rs:79-95`,
`sovereign-cli-mesh/src/mesh_media/offer.rs:12-28`, and
`commonwealth-media/src/declared.rs` (crate paths share `cmnwlth/crates/`).
The admission function uses a mesh-member allow-list whose empty value means
everyone; re-offer explicitly clears it. Neither identity nor default fits
selected svrngs groups. Secret storage is already forked locally. No new
module/type is forked in this survey; the existing node is the extension site.

### Share callers and documentation

`callers.txt` is a broad textual inventory, with false positives (expense
shares, historical PLAN, and old removed folder commands) kept visible.
Production declaration calls are CLI `cli.rs:1550`, home creation `:573`
and sign-in `:457`; limit and withdrawal are separately listed above.
Give has **two production invocations** at `apps/give/src/keep.rs:539,683`
and **one adapter definition** at `apps/give/src/lend.rs:74-82` (three sites,
not three production calls). Both keepers already own group context; pass
its ID to the adapter instead of discovering or inferring an audience.

Implementation must update command harnesses (`crates/svrngs/tests/share.rs`,
`udp.rs`, `tcp.rs`, `scenes/{doc,events,films,life,night,pantry,ports}.rs`),
direct `Lend` fixtures/node/door/browser/canary tests, and fake Give commands.
Current executable recipes include `apps/{taste,movie-night,jellyfin}/check/`,
doc and pantry startup text; current guides include `docs/guide/{lend,build,
apps,games,models,how-it-works,troubleshooting}.md`, README and ARCHITECTURE.
Keep past `demo/runs` evidence byte-for-byte; do not rewrite past releases
or historical PLAN records into claims they never proved. New tests for
the behavior change belong to the next row; this pass changes no tests.

## Candidates, decided before cuts

| Exact candidate | Disposition |
|---|---|
| `door/tcp.rs:83-86` / `door/udp.rs:306-309`: same channel/spawn/handle tail | Four lines share framing but different concrete socket/loop types. A generic helper adds signature/bounds and abstraction for one expression, no supported net reduction. |
| `door.rs:1713-1716` / `origin.rs:630-633`: response construction | Small common tail surrounds different carriage/authorship/secret behavior; extracting construction adds function/call plumbing. No duplicated policy/drive removed. |
| `home.rs:200-204,235-239`: CSP header insertion | Same header on distinct response paths. A helper changes no policy and costs definition/call plumbing; not a measured saving. |
| `home.rs:428-457` / `lend.rs:561-566`: read then mutate declaration | Similarity is a real **missing serialization rule**, not safe deletion. Shared locked mutation belongs with the behavior change; no advance saving banked. |
| `door.rs:953-956` / `door/udp.rs:231`: resolve person | Already one implementation; selected-witness resolution replaces it for scoped requests. Count once, not one new resolver per transport. |
| `node.rs:896` / phone list requests: list version | Existing lower-version check cannot fence old requesters. Removing compatibility branches before implementing request fence weakens behavior. New work. |
| `play.rs:120-131` / `room.rs:163-176`, repeated turn windows | Different turn sets, outcomes and owners. Prior pass already identified this; not folded into scope work without a measured behavior-preserving cut. |
| Imports, extractor signatures, struct fields, Outcome traits in `windows.txt` | Language/interface shape, not redundant drive. No packed lines, identifier renaming or test relocation used. |

**Supported cuts: none. Actual platform/core/JS/Give change: zero.**
Caps stay unchanged; no new budget check or cap adjustment is introduced,
so no budget plant is owed in this row. Every later cap edit must show its
own failing named budget plant then passing recount under CHARTER's approval.

## Smallest remaining build (before scratch result)

Retain the original scoped bounds, but charge each concern once:

| Slice | Non-JS addition | JS addition |
|---|---:|---:|
| Parsed audience, full mutation lock, legacy pause | 110–180 | 20–35 |
| Eligible witness, guests, owner/choice/headers | 55–95 | — |
| Discovery/fence, reads/append contexts | 45–80 | 15–30 |
| CLI and home API | 50–90 | — |
| OIDC code provenance/redemption/userinfo | 40–70 | — |
| Raw live admission/sweep | 20–35 | — |
| Existing create/edit selectors | — | 100–170 |
| HTTP body/upgrade cancellation (prototype pending) | 80–150 | — |
| **Total** | **400–700** | **135–235** |

Zero core, no dependency/service/policy database/ACL or new pass format;
Give separately +10–25 product lines. These are source-grounded estimates,
not measured implementation or advance cap requests. At these bounds,
post-build counts would be 20,189–20,489 non-JS and 8,201–8,301 JS, with
necessary cap increases 357–657 and 112–212 respectively. Land only
`max(current cap, actual count + 1)`, using headroom first, citing the
fresh pass and the operator's CHARTER decision. No increase today.

Scratch prototype and closing evidence will be recorded in `PROTOTYPE.md`
and `CLOSE.md`. Only the next implementation row can claim lender policy,
mixed-version interoperability, actual iroh withdrawal or a House/Trip scene.
