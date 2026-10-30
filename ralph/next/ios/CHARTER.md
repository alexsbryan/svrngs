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
