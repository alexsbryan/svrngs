# Relay fallback — connect first

## Direction and authorization

The operator reported a real invitation blocked by CSP at the HTTP Pi relay,
said it should have fallen back, and asked for a simple spec and line budget.
The draft below was presented in chat; the operator then authorized execution:
"Ok start the ralph loop on this". This pair is this queue's launch directive.

1. Keep a public fallback reachable while online, even when the local relay works.
2. Carry preferred and public fallback routes in the invitation.
3. Browsers skip incompatible routes and try eligible routes immediately.
4. Apply that behavior to joining, reopening and opening apps; keep a healthy connection.

The demo: a fresh browser opens an invitation naming a native-only HTTP mesh
relay, reaches the same keeper/group through the public fallback, joins once
and opens a lent app, with production CSP unchanged. Internet-off phones still
need a compatible local HTTPS path; no route policy can manufacture that path.

## Budget

Net product growth: at most 150 nonblank, noncomment lines: 40 native/core Rust,
80 browser Rust, 30 JavaScript. Core may grow at most 10 lines, included in the
40. Tests: at most 150 added code lines using existing harnesses. Product-doc
changes: at most 30 added lines; queue instructions and run evidence are separate.
These are ceilings, not targets. Baseline is source f8a57035206be60212d283493ffd3a944976fbdd.

The launch approves the work envelope, not an advance repository-cap increase.
Economize first; preserving cuts get their own commits and ratchet caps down.
If measured necessary growth does not fit the current caps, stop with the
post-pass counts and the smallest ready cap diff for the operator. Do not edit
BUDGET or the trust floor without the required operator decision. Do not re-ask
for the feature scope or envelope already authorized here.

## Reuse and feasibility

Read core relay::Meeting::{decide,carried}, node tiers and invite::Host::link,
web/src/lib.rs::{bind_for,connect_to_join,endpoint,holder_endpoint,relay_of},
web/page/dial.js and web/page/hand.js, web/lend.js, and the existing relay_tiers
and browser test harnesses. Survey commonwealth ring code before a new module.
One policy owns fallback configuration; do not duplicate n0 names in JavaScript.

First prove, from a cold keeper and fresh browser, that the advertised fallback
actually reaches the expected issuer while a local primary exists. A relay's
/ping or online handshake alone is insufficient. Inventory pinned iroh before
claiming simultaneous registration. If its existing API cannot keep both warm,
prefer a browser-compatible public home online and prove native local recovery
when internet disappears. Stop if a new transport subsystem or dependency is
required; do not build one under this budget.

Race transport establishment, not writes/redemptions. Keep one endpoint per key;
multiple endpoints on one relay displace each other. Cancel and close losers.
Authenticate the expected peer; refusal/expiry is not a reason to bypass access
rules. Reuse existing deadline constants if evidence supports them; trace skipped
routes, attempts, winner, timeout and cleanup without invitation secrets.

## Boundaries and verification

Use this worktree and synthetic test identities only. The live Pi, keeper data,
public resolver and other worktrees are outside worker write/process scope.
No deployment, public publication, DNS change, purchase, push or main merge.
Read-only incident evidence is at /Users/alexsbryan/dev/svrngs-lanes/relay-meet/
target/relay-demo/public-browser-pi-http-fallback.json; never commit its invitation.

Production CSP stays unchanged. The regression must use an HTTPS page with that
policy, not the harness's relaxed HTTP-loopback policy. Plant each new gate and
watch it fail. Focused checks close intermediate units; one unchanged locked
scripts/check.sh closes the explicit final audit. Real-phone/cut results remain
never-ran until the operator performs them. Every new direction rewrites DESIGN
in place alongside its behavior change. The loop alone marks STATE rows.
