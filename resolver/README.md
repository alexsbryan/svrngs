# Short invitation resolver (not deployed)

`createResolver({token, url?, debug?})` in `handler.mjs` returns a Fetch-API
handler `(Request, trustedIngressIp) => Promise<Response>`. Node 20+; no SDK
or dependency install. The hosting adapter must supply the actual ingress IP,
never a caller's forwarded header. Missing IP/configuration refuses service.
The default URL is the operator's Free database; credentials stay server-side.
This directory starts no listener and supplies no deployment configuration.

POST `/` implements the native signed contract in
`../demo/runs/onboarding/2026-11-04-native/README.md`. It additionally accepts
signed `status` with null token. GET `/<five-word-phrase>` returns the ordinary
invitation and canonical page origin as JSON, for the native-arrival UI.
There is no list route, CORS access, background retry or generation seeding.
Ended responses include instructions to reopen svrngs; every refusal/provider
failure is 503 `unavailable`, `Retry-After: 60`, and manual-retry guidance.
Debug events contain operation/verdict only, never phrases, IPs or tokens.

`store.mjs` owns the atomic lifecycle and limits. Permanent reservations bind
owner/id, digest, expiry and phrase, not group names or capability. A pending
reservation fixes the phrase before any secret write. Secret keys have absolute
expiry. The final open marker makes them visible; close deletes the secret
before its final closed marker. An interrupted close reads unavailable until
retry, not falsely ended. A response lost after commit is uncertain to the
caller; the same signed operation reconciles it. Never initialize the generation
from the handler. See the store survey's RESET.md for rollback limitations.

The 100,000 reservation cap counts hash fields conservatively (a published
reservation uses two fields), so allocation can stop earlier. Redis used_memory
at 100 MB also stops allocation; reads/closure remain available under their own
quotas. Global/IP/owner counters are shared and expire after 32 days; calendar
month boundaries are checked against Redis TIME. Active-secret bounds use
expiry scores reserved before writing a capability, independent of TTL cleanup
timing. New allocation leaves two close/recovery attempts per active-secret
slot near the global monthly mutation ceiling. Metered-command proof and
failure-injection transcripts belong to the recorded resolver run.

The existing `scripts/check.sh` page-laws step includes
`web/tests/resolver.test.mjs`. It needs `redis-server` and `redis-cli` on PATH;
the test owns a Unix-socket-only Redis child with AOF and cleans it on exit.

## Wordlist attribution

`words.txt` is EFF Short Wordlist 1, in source order with dice indices removed,
whitespace regrouped, and **only `yo-yo` removed** to keep hyphens as separators.
Copyright Electronic Frontier Foundation, [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
[Source](https://www.eff.org/files/2016/09/08/eff_short_wordlist_1.txt);
[explanation](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases).
Five independent `crypto.randomInt` draws use rejection sampling with no modulo
bias: 1295^5 possibilities, 51.69 bits. A collision refuses this attempt; a
later explicit retry samples again, with no tight internal retry loop.
