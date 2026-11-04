# Native short-invitation publication — 2026-11-04

Scope: `onboarding-native-publication`, ONBOARDING §3. Fake resolver on
loopback only. No hosted resolver, deployment, release or laptop-scene verdict.
The inherited V1.6 `publish.json` and `publish.log` are untouched and untracked.

## Reuse and boundaries

The inherited implementation extends `Host::link`/`link_held` and `MintInvite`,
the home page's existing proof/origin guard, `declared::write_secret`, and the
shared HTTP/TLS construction in `http.rs`. `http::builder` was exposed in its
own behavior-preserving commit before publication disabled redirects and
automatic retries. Existing client callers keep their defaults.

The upstream inventory in ONBOARDING §4 found no hosted invitation resolver;
this row adds no provider store, phrase generator, admission rule or new
HTTPS/TLS implementation. `Admission::current_invite` is a small projection of
the existing membership fold, surfaced by `RingRail`, rather than a second
invitation fold in the node. It selects the last counting invitation **by
issuer**, including a device invitation or an expired one, rather than the
last invitation naming an old key. Existing `invite_for` is unchanged.

## Native boundary and resolver contract

The existing authenticated home origin serves:

- `GET /publication?group=<namespace>`: local status, or `null` before any
  publication. No resolver I/O.
- `POST /publication {group,id?}`: first publication, or an explicit retry of
  the persisted pending one. A new publication after close/supersession
  requires the previous receipt's `id`, avoiding accidental replacement on
  reload. An already-open publication is returned locally.
- `POST /publication/close {group,id}`: close exactly this receipt. A different
  id is refused without changing the newer invitation. Duplicate completed
  close is local and writes no new invitation.

Success returns `{id,state,phrase,until}`. `state` is `pending`, `open`,
`closing`, `closed`, or `ended`; `phrase` is null until publication succeeds.
`ended` means the native invitation was superseded or its expiry reached,
never that the resolver failed. A provider failure is HTTP 503:

```json
{"error":"joining is temporarily unavailable; retry in a minute from this computer"}
```

The host's `join-resolver` file names the resolver mutation endpoint for this
native boundary. HTTPS is required except numeric loopback HTTP used by the
fake. Credentials and fragments in the URL are refused. No endpoint is
provisioned by this row. A pending receipt retains that exact endpoint across
retries; editing configuration cannot silently send its token somewhere else.

The native client POSTs UTF-8 JSON bytes containing `action` (`publish` or
`close`), `id` (the ordinary invitation's public key, hex), `group`, `token`
(ordinary encoded invitation on publish, null on close), `until` (Unix seconds,
the invitation's original one-day expiry), and `page` (canonical saved page
origin or null). Headers:

- `content-type: application/json`
- `x-svrngs-key`: host's existing Ed25519 public key, hex
- `x-svrngs-signature`: hex `App::parse("onboarding").sign(key, exact_body)`;
  the signed bytes are `svrngs-app:onboarding:` followed by the exact body.

The resolver must authenticate those bytes and bind the owner/id immutably,
as ONBOARDING §3 requires. Retries preserve the same body and expiry. A
successful publish acknowledges `{id,phrase}` with five nonempty lowercase
hyphen-separated words; successful close acknowledges `{id,closed:true}`.
Any non-success, malformed/oversized body, wrong id, invalid phrase, omitted
acknowledgment or ten-second whole-response timeout is unavailable. Responses
are bounded to 8 KiB. No redirects are followed and no automatic retries run.
The fake checks the real signature and the absence of a bearer token.

The host stores `publication.json` atomically with mode 0600 in the group's
directory **before** sending. A failed publish retains `pending`; failed
withdrawal retains `closing` and leaves the ordinary invitation valid.
Explicit retry is the recovery mechanism; status never retries. After a close
acknowledgment, the host checks that this is still its current invitation and
supersedes it with a fresh undisclosed ordinary invitation before it persists
and reports `closed`. Previous admissions still count. If the final local
write fails, retry uses the same persisted receipt; a newer invitation is
never invalidated by a stale close. I/O holds no invitation mutex; both
receipt identity/phase and current invitation are checked after the await.

The browser receives neither the ordinary invitation token nor the record API
bearer. It makes no cross-origin call to the resolver. Host controls and the
resolver adapter consume this boundary in their own rows.

## Instrument and red runs

Tests are in `crates/svrngs-node/tests/home/publication.rs`. The fake reads
actual HTTP bytes and verifies the signature with `App::check`. Its reply
channel deliberately withholds answers while another authenticated native
request mints a link or closes a publication. Restart runs use five separate
processes of the test executable over the same data directory, not a new
handler over retained in-memory state. The child phases are pending, open,
closing, closed and final inspection; each must exit successfully within 30 s.

Attempting an embedded `Node` restart hung in `Folders::start` after the
endpoint/cutover and before blob-store startup completed, even after dropping
its runtime. Those attempts were timed out, not counted as passes. The test
uses process termination to release database ownership, as a real restart does.

Pasted output below omits compilation/backtrace boilerplate. Each command
exited 101; all plants were removed before the green run.

**Plant: remove both publication routes**, reproducing the row's absent
feature premise:

```text
$ cargo test -p svrngs-node --test home publication -- --nocapture
running 4 tests
publication_survives_reload_restart_and_closes_only_its_invitation:
  assertion `left == right` failed: left: 404, right: 200
publication_unavailable_is_retryable_not_an_ended_invitation:
  assertion `left == right` failed: left: 404, right: 503
publication_failures_keep_one_recoverable_invitation_and_receipt:
  called `Result::unwrap()` on an `Err` value: Elapsed(())
publication_late_answers_cannot_override_new_links_or_closure:
  called `Result::unwrap()` on an `Err` value: Elapsed(())
test result: FAILED. 0 passed; 4 failed
```

The lifecycle test was subsequently renamed to
`publication_survives_reload_and_closes_only_its_invitation`, with restart
coverage moved to the separate process-boundary test. Existing home-page tests
were not renamed or edited.

**Observed inherited defect: follow a 307 before rejecting the final URL.**
The redirect target's listener observed a connection; rejecting the final URL
after sending the invitation was too late:

```text
$ cargo test -p svrngs-node --test home publication_refuses -- --nocapture
invitation must never follow a resolver redirect
test publication::publication_refuses_redirects_oversized_bodies_and_timeouts ... FAILED
test result: FAILED. 0 passed; 1 failed
```

**Plant: iterate oldest-first in `Admission::current_invite`.**

```text
$ cargo test -p svrngs-core current_invite_is_per_issuer -- --nocapture
assertion `left == right` failed
  left: OpId(ContentHash(c01ba834e202513b))
 right: OpId(ContentHash(b5e5797474373951))
test project::tests::current_invite_is_per_issuer_not_per_old_link_or_arrival_order ... FAILED
test result: FAILED. 0 passed; 1 failed
```

**Plant: skip native supersession after acknowledged close.**

```text
$ cargo test -p svrngs-node --test home publication_survives_reload -- --nocapture
copied long link must end
test publication::publication_survives_reload_and_closes_only_its_invitation ... FAILED
test result: FAILED. 0 passed; 1 failed
```

**Plant: accept a late publication without its current-invitation guard.**

```text
$ cargo test -p svrngs-node --test home publication_late_answers -- --nocapture
assertion `left == right` failed: a late answer must not mark an old invitation open
  left: String("open")
 right: "pending"
test result: FAILED. 0 passed; 1 failed
```

**Plant: omit the pre-request receipt write.**

```text
$ cargo test -p svrngs-node --test home publication_recovers_pending -- --nocapture
publication::publication_restart_child:
  called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
restart phase pending: exit status: 101
test publication::publication_recovers_pending_open_closing_and_closed_across_process_restarts ... FAILED
test result: FAILED. 0 passed; 1 failed
```

**Plant: let a stale close supersede unconditionally.**

```text
$ cargo test -p svrngs-node --test home publication_late_answers -- --nocapture
assertion `left == right` failed: stale close must keep newer ordinary invite
test publication::publication_late_answers_cannot_override_new_links_or_closure ... FAILED
test result: FAILED. 0 passed; 1 failed
```

## Green runs

```text
$ cargo clippy -p svrngs-node --lib --test home -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.62s

$ cargo test -p svrngs-core current_invite_is_per_issuer
test project::tests::current_invite_is_per_issuer_not_per_old_link_or_arrival_order ... ok
test result: ok. 1 passed; 0 failed

$ cargo test -p svrngs-node --test home -- --nocapture
running 16 tests
test publication::publication_survives_reload_and_closes_only_its_invitation ... ok
test publication::publication_late_answers_cannot_override_new_links_or_closure ... ok
test publication::publication_failures_keep_one_recoverable_invitation_and_receipt ... ok
test publication::publication_recovers_pending_open_closing_and_closed_across_process_restarts ... ok
test publication::publication_refuses_redirects_oversized_bodies_and_timeouts ... ok
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.05s

$ cargo test -p svrngs-node --lib invite
test invite::tests::a_clients_link_with_no_relay_is_at_most_90_characters ... ok
test invite::tests::a_minted_token_spells_no_kept_word ... ok
test result: ok. 2 passed; 0 failed

$ cargo test -p svrngs --test join_by_link --test joined
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.50s
running 1 test
test two_of_one_name_an_hour_apart_on_a_fall_back_night_are_told_apart ... ok
test result: ok. 1 passed; 0 failed

$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19739 / 19833 lines
js                     7499 / 8090 lines
swift                     0 / 1 lines
```

`cargo check --workspace`, `cargo fmt --all --check` and the scoped clippy
command also exited 0 after disabling redirects and automatic retries.
`python3 scripts/trust_check.py` reported
`trust floor: 10 policies, 11 listeners, 9 gates`.
After the final request-policy/formatting changes,
`cargo test -p svrngs-node --test home publication` passed all seven selected
tests in 11.99 s (nine existing tests filtered out for that focused rerun).
The first join regression invocation used the nonexistent target `joining`
and exited 101; the actual unchanged targets `join_by_link` and `joined`
above were then run successfully.

The final closing `scripts/check.sh` run is owed; the loop will run it once
after this implementation is committed. Its actual output/exit status must
be recorded before the unit is called done.
