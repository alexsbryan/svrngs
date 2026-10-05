<!-- section: u4 after=intro -->
This queue is U4 (`docs/DESIGN.md` §8): the browser. Sam's group names a page
origin; Sam texts its link, `<origin>#<token>`; a phone opens it in its stock
browser and is in, with nothing installed: the page makes and keeps the
person's key, redeems the link at Sam's computer over iroh in wasm, lists what
the group's people lend, and renders a lent service through the dial — fetch,
POST, streaming, byte ranges — so a film plays and seeks. The page is static
files any origin can serve. The U4 row's bar is the bar of the rows that carry
it, and the last row ends in a package for the operator, since this host has
no phone.

The plan for every row is DESIGN §3 "Joining", §8 "U4's inventory" (written by
u4-0 from `ralph/next/u4/UPSTREAM.md`, the seat's survey of upstream) and §10.
Upstream is commonwealth-ai at `e1a43bcd2`; its browser dial is
`sovereign/apps/ring-runtime/` there (`src/lib.rs`, `web/`), built by
`scripts/build-ring-runtime.sh`. Read a file there with `git -C
~/dev/commonwealth-ai show e1a43bcd2:<path>`.

**The seat's calls** (2026-10-05, from the survey; u4-0 writes them into DESIGN):
1. The page holds the person's key as 32 bytes, sealed by a non-extractable
   WebCrypto AES-GCM key, both in IndexedDB. iroh signs its handshake
   synchronously (`iroh-1.0.2/src/tls/resolver.rs:92-95`) and the redeem route
   refuses a redeemer whose dialling key is not the new key (`NotTheDialer`), so
   a non-extractable signing key cannot be the endpoint key.
2. A browser's key stands in the register as a key that dials and is never
   dialled: the round, the live fan-out and the lend list skip it. A browser
   cannot accept a dial, and without this every round pays a dial timeout per
   phone.
3. The service worker holds the iroh endpoint and serves the lent service's
   requests under its scope; the page joins and lists. If a measured run shows
   the worker dying under playback, the endpoint moves to the page and the
   worker relays — a re-scope the director may make, with the run as evidence.
4. The phone dials the person who lends a service, directly by key (it stands
   in the group), relay-only; the node it first dials, the link's issuer, only
   answers what the group's people lend. No node proxies another's lend.
5. The command and the page parse a link and sign the ask and the redemption
   with one body, in svrngs-core (pure: bytes and keys in, bytes out).

**Building the page on this Mac:** the wasm32-unknown-unknown target,
`wasm-bindgen` 0.2.128 and Homebrew LLVM (`/opt/homebrew/opt/llvm`, which has
the wasm32 backend Apple clang lacks; upstream's browser-dial-1 failed without
it) are installed. Playwright drives the browsers (`npx playwright install
chromium webkit` the first time). Nothing in `scripts/check.sh` reaches the
internet: browser tests run a relay on this computer, plain HTTP on
127.0.0.1, and use links that carry it.

A fork lands in two commits at least: first the files as upstream had them,
with only the edits the row names; then each change of meaning, named. New
code is held to `docs/principles/RUST_ARCH.md` from its first line. Anything a
person reads — the page's words, its refusals — uses only DESIGN §1's words:
the group's name, people's names, *computer*, *phone*, never key, member,
node or relay. The infra never learns an app's nouns (`scripts/vocab_check.py`):
the page says what a person lends by its name, never *film* or *library*.
