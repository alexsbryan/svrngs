# svrngs ios — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided

- **2026-10-28: "One app with both, fixed (~70%)"**, the door, and the
  replica on the charger, then "one more mini sprint of validations … so
  we can actually build something useful and demonstrable (like a lean
  scrappy startup)".
- **2026-10-30: "plan out the ralph rows for this work -- scrappy startup
  scope".** The design is svrn-docs `rings/current/ROAD_TO_IOS_GA.md` §2,
  "The app, as the house meets it", drawn on the board
  https://claude.ai/artifact/NGraGqYV7KmaiMwgTZ6XKs.

## Scope: the demo, and no further

**The demo.** Ana taps the chat link on her iPhone, types her name, and
presses Continue once. Jellyfin's app then plays Sam's film over cellular,
with nothing to turn on, and a stranger's key is refused.

**In scope:**
- the door: the spike's tunnel, made product;
- joining from a group link;
- the lends on the phone's loopback;
- two screens and the sheet;
- the recorded run.

**What waits, each for its trigger (into PLAN §2 by `ios-design`):**
- **the replica on the charger:** no one in xenia's cast feels it
  (ROAD_TO_IOS_GA §2.1);
- **"Ask the house"** as an App Intent;
- **a yes given in the app,** for lends that ask one (§ "Decide");
- **the packet path,** only if review asks (TN3120);
- **joining as one's Safari self,** which needs a phone that makes a device
  link (PLAN §2);
- **on-demand by domain,** which would keep the VPN slot free between uses;
- **Android.**

## Why the order

1. **Earn the build** (canon `can-52873aa9dc49`). `HUMAN-ios-observe`
   comes first: four observations on the phone, about twenty minutes, each
   of which changes a row.
2. **Design before code** (memory, 2026-10-25). `ios-design` writes DESIGN
   in place, and the operator agrees to it before any builder runs.
3. **A move is its own commit** (AGENTS.md 2). The spike lands with its
   behaviour kept, then changes.
4. **The budget.** An economizing pass, then the operator's raise. The
   spike sits outside the budget under `demo/runs/`; made product, it
   counts.

## Settled before launch, 2026-10-30, from what exists

- **The loop can build the app unattended.** `xcodebuild` for
  `generic/platform=iOS` with `CODE_SIGNING_ALLOWED=NO` built the spike
  in 9.5 s on this Mac with no keychain or phone. The gate's iOS step needs
  no signing.
- **Joining is reuse.** The page's join is three requests to the issuer
  (`web/app.js:81`, `:172`, `:204`) over `svrngs_core::joining`'s types
  and `Device::Browser`, which build natively. `web/src/join.rs` is 78
  counted lines of wasm glue around them.
- **The team is an individual's** (the local provisioning profiles'
  TeamName). Guideline 5.4 holds a VPN app on the store to an
  organization's account, so:
  - the demo goes by TestFlight's internal testers or an ad hoc build,
    neither reviewed;
  - the operator starts the organization enrollment (D-U-N-S, weeks) now,
    in parallel, since it is GA's longest wait.

## Estimates: lines added, by `scripts/lean_check.py`'s count

The cap stands at 19,509 of 19,510 lines, with svrngs-core at 3,241 of
3,242. The repository's cap counts `.rs`, `.js`, `.mjs`, `.ts`, `.py` and
`.sh` (`lean_check.py` `CODE`); Swift has its own cap, at 0 of 1.

| Row | Counted | Swift | From |
|---|---|---|---|
| ios-move | +200 to +250 | +120 | The spike's `lib.rs` is 313 counted lines, about 50 of them inline tests, which move to `tests/` and stop counting. `build-xcframework.sh` is 33 and `key.sh` 21. The probe, plants and steps (about 60) are cut. Swift: the app's 76 and the extension's 42. |
| ios-patches | +30 to +45 | 0 | Fetch by the lock's checksum, apply, the patch flags; one policy constant. |
| ios-lazy | +40 to +70 | 0 | A listener that starts the endpoint, an idle timer, the stop; a silence limit, about 5. |
| ios-join | +40 to +80 | +60 to +90 | Native requests over svrngs-core's types, against the wasm glue's 78, less `key.sh`'s 21. Swift: the Keychain, the universal link, the Join screen. |
| ios-lends | +80 to +130 | 0 | `GET /people/lends`, a listener per lend, ports, the list to the app group. |
| ios-screens | +20 to +40 | +300 to +450 | The list and the leave across the FFI. Swift: House, This phone, the sheet. |
| **The queue** | **+410 to +615** | **+480 to +660** | svrngs-core: 0. |

These are judgments from the spike's sizes, not counts of written code;
`ios-economize` (d) corrects them. Past passes cut 3 to 93 lines, so
the raise asked lands near the counted range.

**Swift counts, on its own rung** (the operator, 2026-10-30: "Swift should
count in its own rung. It will now be first party code that we maintain
fully so we should account for it. It's not just demo code. But it's not
the core core library which we're the utmost precious about").
`lean_check.py`'s `BUDGET` has a `swift` cap, one past 0, so HUMAN-ios-budget
sets two raises: the repository's, from the counted column, and Swift's,
from its own.

## Decide

- **Where the app lives.** Suggested: `phone/`, its own Cargo workspace as
  `web/` is, since it holds libc at 0.2.189. The Xcode project goes under
  `phone/ios/`.
- **How the patches apply.** The exact crate is taken from the registry by
  its checksum, and the patch is applied at build. No vendored iroh in the
  tree. The computers' build stays stock.
- **The reports policy.** The ladder's periodic step (1.31 MB an hour) or
  its relay step (1.10). Periodic is suggested: it refreshes a direct path
  at 300 s.
- **A lend that asks a yes.** Its row says in words that the app can't give
  the yes yet, and where it can be given. It is never hidden and never
  opened without the yes.

**Bars by their spirit** (operator, 2026-10-08).

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:
- **the budget:** any raise, and any svrngs-core line;
- **Apple:** anything that needs the developer account or App Store
  Connect, signing beyond the local team, or an upload;
- **outward-facing steps:** deploying the page (the
  `apple-app-site-association` file), filing upstream issues, publishing,
  pushing, history rewrites, `--no-verify`;
- **the trust floor:** a new listener or policy (`scripts/trust_floor.txt`);
- **dependencies:** a new crate beyond what the spike already uses;
- **the phone:** any step that needs the operator's iPhone in hand;
- **existing groups:** a change to what a record means or what a computer
  holds;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
