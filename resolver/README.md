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
signed `status` with null token. GET `/<phrase>` returns the ordinary
invitation and canonical page origin as JSON, for the native-arrival UI.
An HTML navigation instead redirects to that canonical page's native-install
mode, carrying the ordinary token in its fragment. It uses `no-store` and
`no-referrer`; missing/invalid canonical-page data refuses the handoff. HTML
answers that are not a redirect contain only fixed text, and at most one
quoted word of letters, under a script-free policy.
The canonical page reuses its name form and installation/copy controls without
redeeming a browser identity. The short-address host is never a download source.
There is no list route, CORS access, background retry or generation seeding.
Debug events contain operation/verdict only, never phrases, IPs or tokens.

`read()` turns the path into the one phrase it can mean before any store call:
any case, any run of other characters between parts (`-`, `%20`, `.`, `_`,
`+`, `/`), the number glued to a word or split, a query string ignored, and one
slip in a word (a letter changed, added, dropped or two neighbors swapped)
read as the only word it can be. Each request is still one lookup of one
phrase against the same budgets, so forgiveness gives a guesser nothing. Five
of EFF's words with no number are a phrase issued before 2026-11-16 and keep
resolving: an open one until its day ends, a closed or expired one as ended.

Each answer says what happened. An address that cannot be read is 404
`wrong` and names the part: the shape ("three words and a four-digit
number") or the word ("“acron” isn't one of the words"), and to check it with
whoever gave it. A well-formed phrase never issued is 404 `missing`, "No
invitation has this address", since reservations are permanent and tell it
apart from `ended`. Ended (200) includes instructions to reopen svrngs. A
limit, a pending publication or a provider failure is 503 `unavailable`,
`Retry-After: 60`, with manual-retry guidance and the way around it: ask for
the link in a chat or its QR code, which do not use this address.

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
quotas.

Guessing is bounded by misses, not lookups. A miss is a lookup of a phrase
never issued; finding an open, ended or pending invitation spends none, so
real arrivals no longer spend the guessers' budget. A miss spends its
network's budget, 30 an hour and 100 a day, and the global one, 1,000 a day
and 20,000 a month (`MISSES`). A network over its budget has every lookup
refused, hits too, with one answer (429, "Too many addresses from here
didn't match an invitation", try again in an hour or tomorrow); a spent global
budget refuses every lookup until its bucket turns (503). Both are checked
from the counters alone, so a refusal says nothing about which phrases exist.
Every lookup and status also counts per network, 30 a minute and 300 a day,
and toward 30,000 a month, which keeps accepted commands inside Upstash Free's
500,000: 30,000 public operations at 9 commands at most plus 3,000 mutations
at 16 is 318,000. Refused requests still cost 5 or 6 commands each, and no
limit inside the store can bound them; only one at the hosting edge could.
A network is an IPv4 address, or an IPv6 /56, one household's usual
allocation, so a host cannot rotate through its /64 for fresh budgets; a
mapped IPv4 address (`::ffff:a.b.c.d`) is kept whole.

The worst case, every miss of a month spent guessing while all 100 slots
(`OPEN`) stay open, finds one with chance 20,000 x 100 / (818^3 x 10^4) =
3.7e-7, about 1 in 2.7 million; `resolver-contracts.test.mjs` holds it at
1e-6 or less, reading the values the script runs. Commands per lookup,
measured on redis-server 7.2.4 with every inner call counted: a hit 9, a miss
7, an ended phrase 8, a refusal 5 or 6, an address that cannot be read 0.

Global/IP/owner counters are shared and expire after 32 days; calendar
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

It is the one home of the words. Read aloud, five of them caused typos, and
1,079 pairs in it are one letter apart (afar/ajar, alike/alive), so a slip
could spell another word. `handler.mjs` derives the drawn list from it as it
loads, in about 20 ms: sorted shortest first, then alphabetically, a word is
kept unless a word already kept is within one edit of it (Levenshtein
distance 1). That keeps 818 words, no two within one edit, and needs no second
list to keep in step; `words.txt` itself still reads the five-word phrases
issued until 2026-11-16. A phrase is three independent `crypto.randomInt`
draws from the 818 and a four-digit number, such as `cedar-otter-robin-4821`,
with rejection sampling and no modulo bias: 818^3 x 10^4 possibilities,
42.32 bits. A collision refuses this attempt; a later explicit retry samples
again, with no tight internal retry loop.
