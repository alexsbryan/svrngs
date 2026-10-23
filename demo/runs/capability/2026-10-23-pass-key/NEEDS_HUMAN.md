# capability-pass-key: no fix inside `web/` keeps the idle reopen path

At a6ff5804. The row says: if no fix keeps the path, NEEDS_HUMAN.md with
the trade-off measured, never the friction shipped. This file is that.

## The question

A lend's pages and its worker share one origin. Whatever the worker can
read back after the browser stops it, the lend's own pages can read too.
So which of these does the operator want?

- **(H) A key that can't be copied, and a door that takes a second
  pass.** Sizing below. It needs svrngs-core lines past this bundle's +3,
  and computers running an older svrngs.
- **(B) The key held only in the worker's memory.** This ships the
  friction the row forbids.
- **(K) Leave the key where it is, and say so.** Under the note's §5
  trust, maybe with a shorter pass.

## What was found

1. **The browser keeps no store for a worker alone.** IndexedDB, the Cache
   API and the worker's script URL (`registration.active.scriptURL`) can
   all be read by any script on the origin. The worker's memory is the
   only thing a page can't read, and the browser drops it when it stops the
   worker.
2. **On iPhone Safari a page restores the pass, not the worker.** u4-10: the
   worker there read nothing its page had kept, so `lend.js:296-300`
   redirects to `open.html?for=`. That is a page at the lend's origin, and
   it reads the pass from the store (`open.js:46`) and hands it over. If
   open.html can read the restore path, so can the lent page's script.
3. **Workers really do restart mid-session.** `ralph/REVIEW_FINDINGS.md`
   #65: WebKit ended a lend's worker. Its trace starts over with "the pass
   read again from the origin's store" 28 s after the seeks, and a request
   went to the network instead of the worker.
4. **Automation can't say how often.** `idle.mjs`
   (`data/idle.txt`): with no traffic for 70 s, neither Chromium 147 nor
   WebKit 26.4 under Playwright stopped the worker. DevTools keeps it
   alive. So how often (B) bounces a person can't be judged here. It
   would take a field run on a phone.
5. **A key that can't be copied works in both engines.** `probe.mjs`
   (`data/probe.txt`): an Ed25519 key made with `extractable: false` can
   be kept in IndexedDB, read back, sent to a worker and used to sign
   there, and `exportKey` is refused for `pkcs8` and `jwk`. This holds in
   Chromium 147 and WebKit 26.4.
6. **iroh can't dial with that key.** The handshake signs synchronously
   from raw bytes (`store.js:24-28`; `lend.js:135-137`, `holder_endpoint`
   at `web/src/lib.rs:330`). WebCrypto's `sign` is async. The door lets
   in only a dialler who is the pass's holder (`pass.rs` `Pass::check`,
   `self.holder != holder`; `door.rs:1076`).

## The options, measured as far as this host can

| | Copy of the key's bytes from the lend page | Idle lend reopens without the group page | Phone's acts signable (capability-signed) | Cost |
|---|---|---|---|---|
| **H** | refused (5) | yes: the page reads the key from the store and hands it over, as today | yes, async WebCrypto | door + check, below |
| **B** | refused | **no**: each restart (3) sends the person through the group page, and a fetch made during one fails `503 pass` (`lend.js:282`) | only while the worker runs | about +20 lines in `web/` |
| **K** | readable, as `ask.mjs` step 5 shows | yes | yes | 0, plus docs |

**H in full.** The pass's holder becomes a WebCrypto key that can't be
copied (`app.js:264` makes it, `open.js:44` keeps it). Each run of the
worker makes a fresh endpoint key in memory. The holder signs a second
`Pass`: holder → that endpoint, for the same lends, for a short time. The
door checks the chain: the outer pass against whoever dials, and the inner
pass with the outer's signer as its holder. The outer pass may name no more
than the inner one.

What H costs:
- **svrngs-core:** the chain check belongs beside `Pass::check`, since the
  decider is one place. That is about +15 to +25 lines. svrngs-core is at
  3,149 / 3,150, and the bundle's ceiling is 3,153 (+3). The SURVEY.md §6
  estimate for this row is 0 lines in svrngs-core. So H needs the
  operator's ceiling.
- **Repository:** about +40 to +70 lines across the door, the wasm side
  (sign the outer pass's message through WebCrypto, not a second copy of
  the format in JS) and `web/`. That is inside the row's +20 to +80.
- **Older computers:** a phone page served from group.svrnme.sh would send
  a chained pass to a computer whose door doesn't know one. There are two
  ways to handle it:
  - refuse those lends. That breaks a path someone uses today until their
    computer updates (fence 2);
  - fall back to raw bytes for older doors. That keeps today's hole open
    for them.
- **What H does not close:** while the lent page is open, its script can
  still read the key object from the store and use it to sign outer passes
  for keys it sends elsewhere, valid until the inner pass lapses
  (`PASS_LIFE`, 12 h, `app.js:81`). A short limit on the outer pass doesn't
  help, because a page that plans ahead can sign one for each window. So H
  stops passive copying (a library dumping IndexedDB, a backup) and keeps
  the key's bytes off other machines. It does not stop a hostile app that
  was chosen, and the note's §5 already trusts a chosen app with what it
  reads. A real wall is a page that isn't on the worker's origin: the
  sandboxed carried page, which waits for its trigger (STATE.md, "Waits").

**K, if chosen.** DESIGN §3 would say that the pass and its holder are as
readable to a lend's pages as what the lend reads, and are trusted the
same way (§5). If wanted, `PASS_LIFE` could be shortened to limit how long
a copied pass works. A lapsed pass already reopens through the group page
without a tap (`lend.js:367`, `app.js:478-479`), but that trip has to reach
a computer.

## Recommendation

(K) now, with its words in DESIGN §3. Then (H) **with capability-signed**.
That row already changes the door's checks and asks the operator for a
svrngs-core ceiling (CHARTER.md, "the budget"), and it will need the
phone's worker to sign with this key anyway. Doing H there means the door
changes once and older computers are handled once. (B) is out: it is the
friction the row forbids.

## Reproduce

    node demo/runs/capability/2026-10-23-pass-key/probe.mjs
    node demo/runs/capability/2026-10-23-pass-key/idle.mjs chromium webkit

Nothing in product code changed in this session. The red test in (a) is
not committed: without a fix it would fail every gate.
