<!-- section: onboarding after=intro -->
This queue starts the self-service room welcome in `docs/ONBOARDING.md`.
Before the row, read this queue's `CHARTER.md` whole, `STATE.md`'s opening,
and `docs/ONBOARDING.md` whole. Use `docs/PLAN.md` §1 for priority.

The current-base economizing pass is complete. The operator approved its
upper estimate (3,494 core, 19,833 `.`, 8,090 `js`), chose Upstash Redis
Free with no paid overage, and approved the paired phrase/limits. The
operator then provided the live Free database
`https://true-pheasant-225442.upstash.io` with its key in Keychain (service
`svrngs-upstash`, account `alexsbryan`) and asked to move forward. Read the
key with `security find-generic-password -s svrngs-upstash -a alexsbryan -w`
inside a script; never print it, pass it on a command line, or commit it.
`demo/runs/onboarding/2026-11-04-store/live.txt` records the live atomic
proof. The written-answer gate is superseded by the reset guard.

Every provider limit or failure must fail gracefully: "joining is temporarily
unavailable" with retry guidance, never an ended invitation; existing members
keep working; no partial publication or close, no paid upgrade, and no tight
retry loop. A missing generation key means the store may have reset: refuse
short-address service rather than reuse the URL namespace. Native publication
and host controls may proceed independently against the documented contract
and a fake resolver.

The operator deferred publication of recent multi-ring join-flow fixes to
start this queue. Do not deploy Upstash or Vercel, publish, push, or modify
the V1.6 publish artifacts `demo/runs/v1.6/2026-11-03/publish.json` and
`publish.log`.

Keep the phone-write sync row after onboarding as `docs/PLAN.md` says. Preserve
existing behavior in every economizing cut, and keep each behavior-preserving
move apart from behavior changes. Each cut gets its own commit with the
unchanged tests named by `STATE.md`. The queue's worker commits its work; do
not stage unrelated files or use `git add -A`.
