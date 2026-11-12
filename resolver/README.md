# Short invitation resolver

Deployed at `https://join.svrnme.sh/` in Vercel project `svrngs-join`.
The public native lifecycle proof and its limits are recorded in
`../demo/runs/relay-meet/2026-11-04/README.md`.

`createResolver({token, url?, debug?})` in `handler.mjs` returns a Fetch-API
handler `(Request, trustedIngressIp) => Promise<Response>`. Node 20+; no SDK
or dependency install. The hosting adapter must supply the actual ingress IP,
never a caller's forwarded header. Missing IP/configuration refuses service.
The default URL is the operator's Free database; credentials stay server-side.
This directory starts no listener.

## Vercel Node hosting adapter

`api/join.mjs` is the one hosting adapter, a `fetch` Web Handler (Vercel's
recommended shape; no build step). `vercel.json` routes every path on the
project's domain to it and bundles `handler.mjs`, `store.mjs` and `words.txt`
(their `import.meta.url` reads are outside `api/`). It sets no `runtime`: Vercel
autodetects the Node.js runtime from `package.json`'s `engines.node` `24.x`
(the project's default) — an explicit `nodejs20.x` is both discontinued and an
invalid runtime config on this project. `package.json` also sets
`"type": "module"` for the `.mjs` imports.

The adapter supplies the trusted ingress IP from the platform-overwritten
`x-forwarded-for` header, left-most value. Vercel's request-headers contract
(<https://vercel.com/docs/headers/request-headers>, read 2026-11-04) states the
platform "overwrite[s] the X-Forwarded-For header and do[es] not forward
external IPs ... to prevent IP spoofing", and that `x-real-ip` and
`x-vercel-forwarded-for` are "identical to the x-forwarded-for header". That
overwritten value is therefore the real client IP and a caller cannot set it;
there is no fallback to any client-supplied header. An absent or malformed
value, or a missing server-side token, reaches `handler.mjs`'s `configuration`
refusal and answers 503 `unavailable`, never a defaulted identity.

The Upstash bearer token is read only from the server-side environment variable
`SVRNGS_RESOLVER_TOKEN`; it is never written into the site, the build or a
process argument. Set it with `vercel env add SVRNGS_RESOLVER_TOKEN production`
and paste at the prompt (never on the command line), then deploy.

POST `/` implements the native signed contract in
`../demo/runs/onboarding/2026-11-04-native/README.md`. It additionally accepts
signed `status` with null token. GET `/<five-word-phrase>` returns the ordinary
invitation and canonical page origin as JSON, for the native-arrival UI.
An HTML navigation instead redirects to that canonical page's native-install
mode, carrying the ordinary token in its fragment. It uses `no-store` and
`no-referrer`; missing/invalid canonical-page data refuses the handoff. HTML
ended/unavailable answers contain only fixed text with a script-free policy.
The canonical page reuses its name form and installation/copy controls without
redeeming a browser identity. The short-address host is never a download source.
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

`node web/tests/run.mjs unit` runs the original isolated signature/ingress,
provider-failure and hosting-adapter contracts without a store process.
`node web/tests/run.mjs integration` runs `web/tests/resolver.integration.test.mjs`;
it needs `redis-server` and `redis-cli` on PATH, owns a Unix-socket-only Redis child
with AOF and cleans it on exit. Both tiers remain in `scripts/check.sh`.

## Wordlist attribution

`words.txt` is EFF Short Wordlist 1, in source order with dice indices removed,
whitespace regrouped, and **only `yo-yo` removed** to keep hyphens as separators.
Copyright Electronic Frontier Foundation, [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
[Source](https://www.eff.org/files/2016/09/08/eff_short_wordlist_1.txt);
[explanation](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases).
Five independent `crypto.randomInt` draws use rejection sampling with no modulo
bias: 1295^5 possibilities, 51.69 bits. A collision refuses this attempt; a
later explicit retry samples again, with no tight internal retry loop.
