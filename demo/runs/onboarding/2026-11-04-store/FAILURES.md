# Graceful-failure contract for the resolver adapter

This is the policy required by `onboarding-store-proof` (e), not a claim that
a resolver or UI has been built/tested. Sources: `docs/ONBOARDING.md:96-103`,
`ralph/next/onboarding/CHARTER.md:47-52`, the official limit/error pages in
[README.md](README.md) §1, and the real REST refusal in [live.txt](live.txt).
It applies to any provider non-success, without depending on a particular
HTTP status or Redis error spelling. Reset handling is proven in [RESET.md](RESET.md).

## One answer and explicit retry

Host and attendee wording:

> Joining is temporarily unavailable. Please wait at least a minute, then
> try again. If it keeps happening, ask the person who invited you to check
> joining. If you've already joined, open svrngs on your computer to continue.

Resolver failures use HTTP **503**, machine code **unavailable**, and
`Retry-After: 60` as a minimum retry delay, not a promise that capacity will
return in a minute. A known later reset time may increase that delay. Do not
invent a quota-reset timezone. The UI offers a manual retry; no automatic
polling/republication loop. Native HTTP callers retain their pending receipt
and expose explicit retry/status recovery. No automatic paid upgrade, card
entry or plan-switch action is offered.

| Trigger | Resolver/host/attendee behavior |
|---|---|
| Local per-IP/owner/global minute/day/month budget | Reject before publication/close writes; show unavailable and retry guidance. Count attempts according to the approved shared limits. A quota rejection is not a missing phrase. |
| Provider command/day/month refusal | Same unavailable answer whether the refusal is HTTP non-2xx or a Redis error inside HTTP 200. Preserve pending operations. Free monthly command exhaustion is documented; a provider daily quota is handled if encountered, not asserted to exist. |
| Bandwidth or storage exhaustion; capacity watermark; active-secret or reservation cap | Same unavailable answer; never enable eviction, drop reservations, reset the store or upgrade. A creation-only capacity cap may leave lookup/close available when their own checks succeed. If the provider rejects them too, they are unavailable. |
| Body/response too large; malformed/truncated provider reply; unexpected result type | Stop before mutation for ingress size rejection; otherwise unavailable/uncertain. Never interpret a missing or malformed provider result as absent invitation state. Bound both request and response sizes at the handler. |
| Handler/edge rate limiting, provider 429 or any 5xx/other non-success | Same person-facing unavailable answer. The handler normalizes errors it receives; platform errors before the handler require the welcome's network-error view to show the same guidance. |
| Timeout, connection/TLS/DNS failure, dropped response | Unavailable; mutation result is **unknown**, not rolled back or safely absent. Retain its binding/receipt and reconcile with status or an idempotent retry, after the delay. Never mint a replacement just because a response was lost. |
| Missing/mismatched generation, suspected archival/reset/rollback | Unavailable for every short-address operation, including lookup/status/close; no automatic initialization. Operator reconciliation or namespace retirement, per RESET.md. |

An ended answer is allowed **only** after a successful, generation-checked,
authoritative read explicitly confirms missing/closed/expired state. No
transport, decoding or quota failure becomes ended. Existing members' record,
folders and ordinary native operation do not depend on the resolver
(`docs/ONBOARDING.md:92-94`); installed members continue during its outage.

## No partial publication or close

The host must not display a short address until publication is confirmed and
the underlying invitation is still current. Keep the owner-only pending
receipt and original operation binding through a lost response/restart;
resending that binding cannot extend expiry, replace another owner, or reopen
a closed reservation. An explicit close still invalidates the underlying
native invitation first; a failed remote withdrawal remains pending and
cannot be reported as admission closed until the native invalidation is
confirmed. Recovery must never resurrect an intentionally invalidated Invite.

The primitive scripts prove valid-input atomic isolation, **not rollback on
Redis runtime errors** (`ATOMIC.md:31-40`). The adapter must check generation,
arguments/types, limits and stored state before writes, keep uncommitted
capabilities invisible to readers, and make publication/close state a final
commit marker. On uncertain/error outcomes it must reconcile the same
operation, not return a guessed success. Failures must not expose a partial
publication or falsely report closure; tentative data needs safe cleanup.

This policy does not pretend an EVAL is an ACID rollback transaction. Before
the adapter lands, its own red/green tests must inject failures before and
after each write/commit and a lost reply after commit. They must establish
that no reader can use a partial publication, no failed close is reported as
complete, and retry/status reaches the committed result. The proof scripts
here are not deployable storage mutations and must not be copied as if they
already established those properties under storage errors.

The adapter must also inject each table row at the handler/client boundary,
including arbitrary HTTP errors, HTTP-200 Redis errors, timeout and malformed
JSON. Assert the unavailable wording/retry guidance, no tight retries, no
false ended answer, and retained native recovery state. These are future
adapter checks; this survey does not replace them with a simulated UI verdict
or exhaust the live Free allowance to manufacture a quota response.
