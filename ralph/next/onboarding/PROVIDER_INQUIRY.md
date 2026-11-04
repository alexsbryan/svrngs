# Inquiry to Upstash — Free Redis, direct REST

**Superseded by the live database on 2026-11-04.** The operator provided the
Free database and asked to move forward; the resolver uses it with a reset
guard. Keep this as the exact question set if Upstash support is consulted.

Send this to Upstash support from the account that will own the planned
resolver. Ask for a dated written answer for the **Free plan, direct REST
database, no payment method**. Do not provision a database to ask.

We plan a small public resolver that stores short invitation phrases. It
needs one guarantee: an acknowledged create or close is never lost or
reordered in a way that lets a phrase be reused for a different invitation.
We will fail gracefully on any limit or error; we are not asking for an
availability guarantee or a paid plan.

1. **Recovery of acknowledged writes.** Does a successful REST `EVAL`
   acknowledgment mean both the reservation and the close state survive a
   supported crash, failover and partition recovery on the Free plan? Can
   concurrent REST `EVAL`s ever acknowledge incompatible owners, or can
   recovery or last-write-wins roll back a close or a reservation? Which
   endpoint or read is authoritative after recovery, and what configuration
   does that require? Per-script locking alone does not answer this.

2. **Bandwidth exhaustion.** At the Free plan's 10 GB/month bandwidth
   allowance, what does the database return when the allowance is exhausted:
   the Redis error, the REST HTTP status and body, which operations are
   affected (including close), the reset period and timezone, and whether any
   charge or automatic plan change can occur without a payment method?

3. **Permanent reservations after archival or restore.** How are permanent
   reservations recovered after archival or restore? If any acknowledged state
   can be lost, serving the same URL namespace cannot resume until complete
   reconciliation; otherwise it must be retired permanently. A new blank
   database is not recovery.

4. **Limit responses.** For commands/day/month, storage and request-size
   limits, confirm the response shape (HTTP status and body) so a client can
   distinguish a temporary limit from an ended or missing invitation. We will
   treat any non-success as "temporarily unavailable"; we want the documented
   shape, not a new guarantee.

Please include the date, the plan, and any configuration assumptions in the
answer. The resolver will not be deployed before the answer is recorded in
`ralph/next/onboarding/NEEDS_HUMAN.md`.
