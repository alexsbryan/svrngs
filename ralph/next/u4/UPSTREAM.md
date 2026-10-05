# U4 — what upstream has for the browser (the seat's inventory notes, 2026-10-05)

Read-only survey of commonwealth-ai at e1a43bcd2 and HEAD bc0218ef1. u4-0 writes DESIGN §8 "U4's inventory" from it, re-verifying every path:line it uses. Input to the queue, not a record.

## Upstream inventory for U4 ("the browser"), from commonwealth-ai at e1a43bcd2 and at HEAD (bc0218ef1)

**The short answer.** Upstream has a working wasm iroh dialler with a static page around it, and the build and deploy scripts for it. It has nothing for the rest of U4: no streaming HTTP client in the browser, no fetch interception, no Range/206 handling in the browser path, no POST bodies, no person key held in the browser, and no redemption from the browser. The browser-dial-2 page itself was never committed. It was "all under `target/ralph/browser-dial/`" (DECISIONS.md:9925), and that directory is not on disk now. Commit 0575b9c42 turned that page into `sovereign/apps/ring-runtime`.

**What moved by HEAD.** The layout move (#68) only renamed paths. `ring-runtime` is now under `cmnwlth/apps/ring-runtime/`:
- `src/lib.rs`, `web/app.js` and `web/sw.js` are byte-identical (R100).
- `Cargo.toml` (R095) only re-points `kernel-types` to `../../../shared/crates/kernel-types`.
- `web/index.html` (R097) changes one comment path.

Other moves: `landing/` → `clients/landing/`; `mesh-join-vocab`, `sovereign-contracts` and `kernel-types` → `shared/crates/`; `sovereign-daemon` and `sovereign-grants` → `svrn/crates/`. The `docs/RING_SPEC.md` retirement list was rewritten (see (d)). `ralph/DECISIONS.md` line numbers are the same at HEAD (browser-dial-2 still at :358, :9909, :9948-9960). Nothing new was added at HEAD for the browser: I checked the added and modified file list.

**How I counted lines** (non-blank, non-comment; `//`, `///` and `/* */` count as comment; a trailing comment on a code line counts as code):
```
# Rust / JS
git -C /Users/alexsbryan/dev/commonwealth-ai show e1a43bcd2:<path> | awk '{l=$0; sub(/^[ \t]+/,"",l); if(inb){if(l~/\*\//)inb=0; next} if(l~/^\/\*/){if(l!~/\*\//)inb=1; next} if(l==""||l~/^\/\//)next; c++} END{print c}'
# shell / TOML: drop blank lines and lines starting with #
#   ... | awk '{l=$0; sub(/^[ \t]+/,"",l); if(l==""||l~/^#/)next; c++} END{print c+0}'
# HTML: the same, also dropping <!-- ... --> blocks
```
The 153 for `lib.rs` matches upstream's own baseline, `quality/baselines/lines.tsv:77` (`153 sovereign/apps/ring-runtime`).

---

### (a) Table

All paths are at e1a43bcd2. "(HEAD …)" gives the moved path.

**1. The browser dial — the forkable core**

- **`sovereign/apps/ring-runtime/src/lib.rs:1-203`** (HEAD `cmnwlth/apps/ring-runtime/src/lib.rs`, identical)
  - **What it does:** one export, `dial_guest(dial, token, path) -> String` (:62-63).
    - Parses `<hex>@<targets>` and requires an http(s) relay URL in it (:91-99).
    - Binds a browser endpoint (:119-123): `Endpoint::builder(presets::Minimal)`, `.secret_key(iroh::SecretKey::generate())` (a fresh key on every dial), `.relay_mode(RelayMode::Default)`, `.alpns([GUEST_ALPN])`.
    - Then `ep.online()` (:139), `connect` to the relay address (:142-145), and `open_bi` (:159).
    - Writes a hand-built `GET {path} HTTP/1.1 … Authorization: Bearer … Connection: close` (:166-168).
    - Reads with `recv.read_to_end(512 * 1024)` (:178) and calls `send.finish()` only after the read (:185-187).
    - Returns a text trace, which is also pushed to `window.__trace` (:27-35).
  - **Size:** 153
  - **Depends on** (`Cargo.toml:23-30`, lock `Cargo.lock`, 390 packages): `iroh =1.0.2` (lock: iroh-base 1.2.0, iroh-relay 1.2.0, iroh-dns 1.3.0, noq 1.3.0, ws_stream_wasm 0.7.5, tokio-websockets 0.13.3, getrandom 0.4.3/0.3.4/0.2.17, ring 0.17.14, rustls 0.23.45, ed25519-dalek 3.0.0-rc.0), `wasm-bindgen =0.2.128`, `wasm-bindgen-futures =0.4.78`, `js-sys =0.3.105`, and `kernel-types` by path.
  - **On the "wasm feature":** iroh 1.0.2 has none. Wasm is switched on by target: `build.rs:7` sets `wasm_browser: { all(target_family = "wasm", target_os = "unknown") }`, and `Cargo.toml:394-405` adds target-only deps (getrandom `wasm_js`, time `wasm-bindgen`, wasm-bindgen-futures). The default features include `tls-ring`.
  - **Browser constraints it encodes:**
    - Relay only; it refuses a dial string with no relay URL, so a bare key is never looked up.
    - The send half must stay open until the response arrives.
    - GET only, one request per stream, `Connection: close`.
    - The answer is buffered and capped at 512 KiB; there is no streaming and no body to JS.
    - It does not parse HTTP (it returns the raw status line and text).
    - The key is ephemeral and never stored.
- **`…/ring-runtime/Cargo.toml:1-34`**
  - **What it does:** a cdylib with its own `[workspace]` (:32-34). Its comment, "The LOCKED pin … measured WORKED at exactly these versions" (:21-22).
  - **Size:** 14
  - **Depends on:** as above.
  - **Constraints:** builds only for wasm32-unknown-unknown and stays out of the host workspace.
- **`…/ring-runtime/web/index.html:1-45`**
  - **What it does:** the static page. CSP at :25: `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' wss: https:; img-src 'none'; … base-uri 'none'; form-action 'none'`. `<meta name="referrer" content="no-referrer">` at :24. The script is absolute, `/ring/app.js?v=__VERSION__` (:43).
  - **Size:** 27
  - **Depends on:** none.
  - **Constraints:** a secure context; no inline script; the fragment must stay out of Referer; `img-src 'none'` would block media.
- **`…/web/app.js:1-68`**
  - **What it does:**
    - Reads the fragment fields `token`, `iroh`, `path` (default `/v1/rail/log`) and `s` (:25-29).
    - Runs `init()`, then races `dial_guest` against a 120 s timeout (:41-45).
    - Pulls the body and status out of the trace text and prints them as text (:49-53).
    - Registers the service worker after load, as a best effort (:64-68).
  - **Size:** 47
  - **Depends on:** the wasm-bindgen glue.
  - **Constraints:** shows the raw text; nothing is rendered.
- **`…/web/sw.js:1-34`**
  - **What it does:** a cache-first shell cache named by content hash (:5-11). It handles only same-origin GETs (:28-34): "The dial itself is not HTTP to this origin — it is iroh — so requests to it never reach here" (:26-27).
  - **Size:** 25
  - **Depends on:** none.
  - **Constraints:** shell only; it does not intercept any app traffic.
- **`…/ring-runtime/README.md:1-73`**
  - **What it does:** build and deploy notes. "~4 MB release" (:39). "Browser storage and passkeys are per-origin, so a mirror is a *different* place to a guest's browser, not a copy — pick one origin and keep it" (:48-50).
  - **Size:** — (prose)
- **`scripts/build-ring-runtime.sh:1-119`** (HEAD: same path, with paths re-pointed)
  - **What it does:**
    - Checks for the wasm target (:31-35) and that the `wasm-bindgen` CLI is exactly 0.2.128 (:36-49).
    - On macOS, picks Homebrew LLVM clang and llvm-ar automatically, because Apple clang has no wasm32 backend (:52-76).
    - Builds with `RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown` (:81).
    - Runs `wasm-bindgen --target web` (:85-86), then `wasm-opt -Oz` if it is installed (:91-97).
    - Stamps a 12-hex content hash into `index.html`, `app.js` and `sw.js` (:103-107). `--out <dir>` sets the output.
  - **Size:** 75
  - **Depends on:** rustup with wasm32-unknown-unknown, wasm-bindgen-cli 0.2.128, binaryen (optional), Homebrew `llvm` on macOS, sha256sum or shasum.
  - **Constraints:** the CLI and crate versions must match, or the page fails at runtime; ring 0.17.14 needs a wasm-capable C compiler.
- **`landing/scripts/build-ring-runtime.sh:1-24`** (HEAD `clients/landing/scripts/…`)
  - **What it does:** a wrapper: `--out landing/ring`. Lines :7-16 say the trailing slash is "LOAD-BEARING".
  - **Size:** 3
- **`landing/scripts/verify-ring-deployed.sh:1-92`**
  - **What it does:** after a deploy, resolves the page's script reference the way a browser does and fetches it (:26-81), retrying 6 times, 5 s apart.
  - **Size:** 55
  - **Depends on:** curl.
- **`landing/vercel.json:1-73`**
  - **What it does:** redirects `/ring` → `/ring/` (:4-10), sends `Cache-Control: no-cache` on `/ring/sw.js` (:64-67), and marks `/ring/wasm/*` immutable (:68-71).
  - **Size:** 73 (JSON)
  - **Depends on:** Vercel.
  - **Constraints:** no global `trailingSlash` (commit 44d11c9b1).
- **`landing/package.json:1-16`**
  - **What it does:** `deploy = build:ring && vercel deploy --prod … && verify` (:13).
  - **Size:** —
  - **Depends on:** node >=18 and the vercel CLI; no npm dependencies.
- **`landing/ring/**`** (generated, but committed despite `landing/.gitignore:8`; HEAD `clients/landing/ring/`)
  - **What it does:** the built site at version `0a71a2b5baf0`. `wasm/ring_runtime_bg.wasm` is 4,295,444 bytes (`git ls-tree -l`). `wasm/ring_runtime.js` is the glue, which uses `new WebSocket` (:370, :424), `crypto.getRandomValues` (:278-282) and `fetch` (:271-275).
  - **Size:** 964 (glue, generated)
  - **Depends on:** wasm-bindgen 0.2.128 output.
  - **Constraints:** the relay is reached over WebSocket.
- **`kernel-types/src/alpn.rs:26`** (HEAD `shared/crates/kernel-types/src/alpn.rs`)
  - **What it does:** `GUEST_ALPN = b"cwth/guest/0"`, imported by ring-runtime.
  - **Size:** 1
  - **Depends on:** pulls in `getrandom 0.3` (`kernel-types/Cargo.toml:27`), which is why the RUSTFLAGS cfg is needed.
  - **Constraints:** the ALPN has to be spelled the same at both ends of the dial.

**2. Page and phone shell — LAN only, not iroh**

- **`sovereign/crates/sovereign-contracts/src/guest_pages/ring_shim.js:1-171`** (HEAD `shared/crates/…`)
  - **What it does:** `window.ring` built on plain `fetch()` to the door's own HTTP origin.
    - The bearer comes from `location.hash` (:5).
    - The person's name is claimed with `window.prompt` and `POST /v1/guest/session`, and the handle is kept in `localStorage['ring.session']` (:30-61).
    - Rail routes go to the door with the bearer and an `x-ring-session` header (:69-84). It also has `fold` (:154-161).
  - **Size:** 105
  - **Depends on:** none.
  - **Constraints:** same-origin HTTP to a LAN door; it never touches iroh.
- **`…/guest_pages/shim.rs:1-54`**
  - **What it does:** fills in `{{NAMESPACE}}` and `{{RAIL_BASE}}` and loads the JS with `include_str!`.
  - **Size:** 18
- **`…/sovereign-contracts/src/guest_pages.rs:1-288`**
  - **What it does:** `PAGE_PREFIX = "/ring/"` (:47), `spells_a_page`, `wall_page_base` and `wall_page` (:154-198). These put the door route into the fragment as `path=` when the base is the runtime page.
  - **Size:** 145
  - **Depends on:** serde.
- **`mesh-join-vocab/src/deep_link.rs:228-300`** (HEAD `shared/crates/…`)
  - **What it does:** `build_https_guest_link` and `parse_https_guest_link`, with the fragment fields `token, exp, s, at, iroh, path`.
  - **Size:** 50 (that span only; the file is 757)
  - **Constraints:** never reads left of `#` (:276-277). svrngs already forked this into `crates/svrngs/src/link.rs`.
- **`sovereign/crates/sovereign-daemon/src/guest_door.rs:617-729`** (HEAD `svrn/crates/…`)
  - **What it does:** the LAN door's `proxy_request` and `proxy_get` for a published app.
    - Buffers the request body up to 16 MiB (:617-619, :646).
    - Forwards request headers except `host`, `connection`, `content-length` and `accept-encoding`, so `Range` does go out (:659-667).
    - Buffers the whole response (`resp.bytes()`, :689) and returns only status, `content-type` and body (:681-696).
  - **Size:** 104 (that span; the file is 534)
  - **Depends on:** reqwest, axum.
  - **Constraints:** `Content-Range` and `Accept-Ranges` are dropped and nothing streams, so it is no use for seeking.
- **sovereign-mobile ("the ring shell")**
  - Searched `sovereign-mobile/src`, `src-tauri/src`, `README.md` and `HANDOFF.md` for the word "ring": no hits. At e1a43bcd2 there is no ring shell code. The app is a native Tauri chat app. `src-tauri/src/iroh_bridge.rs:14-17` says the phone's iroh key is "ephemeral (fresh key per app run)" and it uses `presets::N0` (:56-60).
  - HEAD path: `clients/mobile/…`. Native, not browser.
- **`scripts/ring-room-phone/phone.mjs:1-424` and `qr.mjs`**
  - **What it does:** a Node instrument standing in for a phone browser: a fake `location`, `localStorage`, `prompt` and `fetch` around the real shim (:8-15). It decodes the wall's QR with `jsqr`.
  - **Size:** 293 + 49
  - **Depends on:** `jsqr 1.4.0` (`package.json`).
  - **Constraints:** a test-harness pattern only; plain HTTP.

**3. Fetch / stream runtime in the browser**

Not found. Searched every `.js`/`.mjs`/`.ts`/`.svelte`/`.html` file (node_modules, vendor and research excluded) for `respondWith`, `MediaSource`, `SourceBuffer`, `Content-Range`, `ReadableStream`, `TransformStream` and `duplex`. The only hits are the shell caches at `sw.js:31` (source and landing copy). I also searched for `wasm_bindgen`, `web_sys`, `target_arch = "wasm32"`, `wasm-bindgen` and `wasm-pack` across `.rs`/`.toml`/`.json`/`.sh`: only ring-runtime and its two build scripts. No PWA manifest exists anywhere: searched for `rel="manifest"`, `webmanifest`, `"display": "standalone"` and `apple-mobile-web-app`.

**4. Browser key handling**

Not found. Searched for `crypto.subtle`, `generateKey`, `extractable`, `indexedDB`, `IDBDatabase`, `'Ed25519'` and `WebCrypto` in `.js`/`.ts`/`.html`/`.rs`/`.md`. The only hits are prose (`RING_APP_LIBRARY.md:568,623`) and Yjs's lib0 random source inside the ring-doc bundle (`ring-doc-bundle.js:959-960`). Upstream held no browser key at all: it drew a fresh `SecretKey::generate()` for every dial (`lib.rs:120`).

**Can a non-extractable WebCrypto key be the iroh endpoint key?** No, as far as I can tell from the source. This comes from reading iroh's code; upstream never tested it.
- iroh-base 1.2.0 defines `pub struct SecretKey(SigningKey)`, an ed25519-dalek key (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iroh-base-1.2.0/src/key.rs:261`). It can only be built from 32 raw bytes (`from_bytes`, :337) or by `generate()` (:318).
- iroh 1.0.2 signs its TLS handshake with a synchronous call, `impl rustls::sign::Signer for IrohSecretKey { fn sign(&self, message) { Ok(self.key.sign(message)…) } }` (`iroh-1.0.2/src/tls/resolver.rs:92-95`).
- `crypto.subtle.sign` is async and its key cannot be exported, so it cannot be plugged into that call.

This matters because svrngs's redemption refuses a redeemer whose dialing key is not the new key (`svrngs/crates/svrngs-node/src/redeem.rs:383-387`, `NotTheDialer`), and the member ALPN admits only standing keys (`node.rs:440-459`). So the endpoint key has to be the person's key, and it has to exist as raw bytes in the page.

**5. Redemption from a browser**

Upstream had no browser-side join. Its browser tier is a guest: the host signs every act and the guest's name rides the payload.
- `sovereign-grants/src/guest_grant.rs` (627 lines, 326 counted): an in-memory bearer bound to a closed `Scope`.
- `guest_session.rs` (577 lines, 330 counted): a name is bound to an opaque handle under a grant.
- `routes_guest_session.rs` (132 counted): `POST /v1/guest/session`.
- `routes_internal/guest_grant.rs` (287 counted): mint, revoke and list, on the operator surface only.

svrngs DESIGN:442 already marks the grant "not forked". `ring-runtime` does not even claim a session; it sends one GET.

**6. Upstream docs and notes (prose — no code)**

- **`docs/RING_APP_LIBRARY.md`** (HEAD: same path, path edits only)
  - §17 "The page is an iroh endpoint" (:556-563).
  - The static origin is a secure context, and storage is per origin (:565-571).
  - "switching gateways is a new browser key" (:599-600).
  - Apps arrive in an `<iframe sandbox="allow-scripts">` and talk to the runtime by `postMessage` (:602-610). This is design only: `RING_SPEC.md:192-194` says "Unbuilt".
  - The boundaries table (:325-335).
  - Safari "may forget it after a week unused" (:359).
- **`docs/THE_LINK.md`**
  - "The dial, specified" (:224-249).
  - Bootstrap order, with `hash=` checked before the first render (:140-164); `hash=` is still "owed" (:125).
- **`docs/RING_SPEC.md`**: §3.6 (:205-212) and the retirement list (:21-29; HEAD :23-31).
- **`quality/campaigns/ring-apps-shelf.md`**: findings 1 and 3 (:19-45), stream (:83-88), identity/WebAuthn PRF (:92-125), gateway (:181), play (:183-196), phone constraints (:242-246).
- **Native relay measurements**: `research/engram/LAB-RECORD.md:241`; `sovereign/deploy/mesh/WORK_PLANE.md:540-585` (HEAD `cmnwlth/deploy/mesh/WORK_PLANE.md`); `sovereign/bench/media_bridge/TWO_MACHINE_RUNBOOK.md:144-241` (HEAD `bench/lanes/media_bridge/…`).
- **Per-host, not in commonwealth-ai's git at any commit**: `docs/internal` is a symlink to `/Users/alexsbryan/dev/svrn-docs`. Relevant files there are `rings/current/RING_ENTRY.md` and `rings/demo/RING_ROOM_RUN_OF_SHOW.md`.

---

### (b) Measured facts, verbatim

**browser-dial-2** — `ralph/DECISIONS.md:9909-10013`; the source file is `ralph/decisions/browser-dial-2.md`, unchanged at HEAD.

- **Browser and driver** (:360): "opened it in Google Chrome for Testing 147.0.7727.15 (headless) driven by the repo's vendored `playwright-core` 1.59.1. It dialed a live node's `GUEST_ALPN` (`cwth/guest/0`) through the n0 relay and returned **`HTTP/1.1 200 OK`** from `GET /v1/rail/log` with the guest bearer, carrying the named rail row **`ring-658e43cce7830b48`**. WORKED."
- **Layers** (:9948-9960):
  ```
  PAGE: engine=Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/147.0.0.0 Safari/537.36
  LAYER wasm-bindgen/iroh-endpoint: ok — local endpoint ac87abb2…11c3 bound, relay mode default
  LAYER relay-connect: ok — endpoint reports online (relay handshake done)
  LAYER tls/alpn+relay-hop: ok — QUIC connection established to 22a72bb4…86ce on GUEST_ALPN
  LAYER tls/alpn: negotiated ALPN = "cwth/guest/0"
  LAYER request-write: ok — 185 bytes of HTTP/1.1 sent on the bi-stream (send half kept open until the response, as the native bridge does)
  LAYER daemon-guest-admission: response received, 632 bytes
  RESPONSE-STATUS: HTTP/1.1 200 OK
  ```
- **Relay** (:9966-9968): "a browser with no route to `wss://usw1-1.relay.n0.iroh.link./` never reports `online`; the page's 120 s race would print `TIMEOUT`".
- **Build** (:9926-9934): "`iroh = "=1.0.2"`, `wasm-bindgen = "=0.2.128"`, `wasm-bindgen-futures = "=0.4.78"`, `js-sys = "=0.3.105"` … `Finished \`dev\` profile … in 19.03s` (exit 0). `wasm-bindgen --target web` (CLI 0.2.128) … iroh's wasm tree pulls `getrandom 0.4` with `wasm_js` … no `--cfg getrandom_backend` was needed."
- **Node** (:9939-9946): a throwaway `sovereign-cli-daemon`, `[iroh] enabled = true`, `[discovery] mdns = false`, "the same n0 relay the deployed daemon uses"; the grant had `scopes = {rail:"browser-dial-ns"}`, ttl 1200s.
- **The one path fact** (:9977-9983): "on the FIRST attempt the page called `send.finish()` immediately after `write_all`, and `read_to_end` returned `Ok(0 bytes)` with no status line. Holding the send half open until after the response … returned the 200."
- **Side finding** (:9985-9997): the deployed daemon answered `/status` → 200, `/v1/models` → 200 and `GET /v1/rail/log` → `HTTP/1.1 404 Not Found`, because it was a stale binary.
- **Timings:** none for the dial itself. No dial latency, time to first byte, throughput or seek time was recorded for any browser. The only numbers are the build time and the 120 s timeout.
- **Discrepancy with svrngs DESIGN:112.** DESIGN:112 calls this "a stock Chrome tab". The record says headless Chrome for Testing on Linux x86_64, driven by Playwright. It was neither a phone nor a stock browser.

**browser-dial-1** (:9854-9893)
- Mac, exit 101: "error: failed to run custom build command for 'ring v0.17.14' … 'No available targets are compatible with triple "wasm32-unknown-unknown"'".
- Halo, exit 0: "Compiling ring v0.17.14 / Finished `dev` profile … in 15.90s", with rustc 1.95.0 and clang 21.1.8.
- The cause is that the Mac's C compiler has no wasm32 backend, not the version pin.

**Size and phone facts**
- Commit 5eca4a280: "Verified: build → site (4.1 MB wasm)". The committed module is 4,295,444 bytes. README:39 says "~4 MB release". (`the-link-3`, DECISIONS.md:385, measured a rail-core `probe.wasm` of 415,140 bytes; that is not iroh.)
- The only real-phone contact is a failure: "This shipped on 2026-09-22 and a phone hung on the wall link" (`landing/scripts/build-ring-runtime.sh:11-12`). The cause was the trailing slash. Commit 44d11c9b1 says it was "Reproduced in headless chromium against the deployed origin".
- Commit 0575b9c42's "packaging smoke" also ran in chromium.

**Native relay numbers** (no browser involved; they bound what a relay-only browser can expect)
- `LAB-RECORD.md:241`: "iroh relay floor (usw1, relay-only PathSelector BOTH sides): 182ms … 3.7MB/s bulk (… n0 public relay may throttle — self-hosted relay is the product answer there)".
- `TWO_MACHINE_RUNBOOK.md:148-155`: "**8.7 Mbit/s against a 25 Mbit/s bar** … the SENDER was a laptop tethered to a phone". The later legs were could-not-judge (:208-216, :229-235).
- `WORK_PLANE.md:553-555`: "7.8 Mbit/s is an UPPER bound on the relay rate for that run".
- `WORK_PLANE.md:577-579`: "CONNECTION SETUP, paid per viewer AND per seek, already measured at p50 189 ms / p90 1327 ms on an idle LAN with a cold connection".

---

### (c) Gaps — what U4 needs that upstream does not have at all

1. **A real HTTP client over iroh in wasm.** It needs any method, request bodies (POST), header parsing, chunked and content-length bodies, a response body that reaches JS as a stream, and stream reuse or keep-alive. Upstream has one GET with `Connection: close`, buffered to 512 KiB, returned as text (`lib.rs:166-199`).
2. **A way to send a lent app's own loads through the dial** (page navigations, subresources, XHR/fetch, and `<video src>` / `MediaSource`). Today's service worker caches the shell only (`sw.js:26-34`). Nothing upstream shows iroh running inside a service worker, or how long a service worker would live.
3. **Range/206 end to end in the browser.** It is absent. The LAN door even drops `Content-Range` (`guest_door.rs:681-696`).
4. **A persistent person key in the page** (storage, WebCrypto/IndexedDB, or a PRF-unwrapped seed). Absent everywhere, and as (a) item 4 shows, iroh needs the raw 32 bytes.
5. **Redemption from the page.** svrngs needs the base64url token parsed, the link key derived (BLAKE3), the `AskRequest`/`RedeemRequest` signed, and a dial on `svrngs/redeem/0`, then on `cwth/http/0`. ring-runtime only parses the old `token/iroh/path/s` fields.
6. **Dialling a bare key from wasm.** ring-runtime refuses a dial string without a relay (`lib.rs:91-99`), but a svrngs `Bare` token carries none. iroh 1.0.2's N0 preset in wasm resolves keys with `PkarrResolver::n0_dns()` over HTTPS (`iroh-1.0.2/src/endpoint/presets.rs:123-129`). Upstream never used it.
7. **Rendering an app** (the sandboxed iframe plus `postMessage` runtime). It is design only (`RING_SPEC.md:192-194`); the deployed page prints raw text.
8. **Any measurement on a real phone browser** (iOS Safari, Android Chrome) of the wasm dial, its size, cold start, relay throughput, video playback or seek. None exists.
9. **A PWA manifest.** None.

---

### (d) Upstream decisions that would bind U4

- **Version pin.** "The LOCKED pin (Cargo.lock of the measuring build, browser-dial bd-1): the browser dial was measured WORKED at exactly these versions" (`ring-runtime/Cargo.toml:21-26`: iroh =1.0.2, wasm-bindgen =0.2.128, wasm-bindgen-futures =0.4.78, js-sys =0.3.105). svrngs is also on iroh =1.0.2.
- **The send half.** "the one path fact a wasm client must get right and the iroh API does not make it obvious" (DECISIONS.md:9981-9983): hold the send half open until the response.
- **Origin hosting.**
  - Trailing slash: "The trailing slash is LOAD-BEARING" (`landing/scripts/build-ring-runtime.sh:7`).
  - Shell URLs: "Every shell URL is now absolute … `<base href>` … but the page's CSP is `base-uri 'none'` on purpose" (commit 8ecefcd6a).
  - Caching: `sw.js` no-cache and wasm immutable (`vercel.json:64-71`).
- **No plaintext proxy.** "the plaintext-terminating proxy stays refused either way" (`THE_LINK.md:241-242`). "Below the ladder, never: anything that terminates plaintext at a party that matters" (`THE_LINK.md:85-90`).
- **Origin is identity.** "browser storage and passkeys are per origin, so a mirror is a different identity silo" (`RING_APP_LIBRARY.md:569-571`). "switching gateways is a new browser key, joined to the same person by an ordinary `Admit`" (:599-600).
- **Upstream's stance against a browser key in someone else's group.** svrngs DESIGN §10 (:573-575), 2026-10-04, sets this aside, but it is the recorded upstream position:
  - "There is no browser-held identity in anyone else's ring." (`RING_APP_LIBRARY.md:315`)
  - "A browser holds no key in anyone else's ring … The browser crate `sovereign/apps/ring-runtime` … holds no rail-core and signs nothing, and stays so." (`RING_SPEC.md:205-209`)
  - Per-host `RING_ENTRY.md:134-143` (2026-10-03): "A web app added to the home screen could hold an Ed25519 key (WebCrypto has it in every engine since 2025) … On an iPhone it also needs Share → Add to Home Screen … its storage can be evicted after seven days … It reaches the node relay-only … Choose the app". Also :298: "A phone browser reaches the node relay-only (§17) and plays nothing remotely".
- **Keys and passkeys.** "**Passkeys unlock, never sign.** … Use WebAuthn PRF (Chrome 147, Firefox 148, Safari 18, iOS keychain) to unwrap the local Ed25519 key; `largeBlob` is NOT usable on Android." (`ring-apps-shelf.md:110-115`)
- **The browser is never a peer.** "iroh in wasm is relay-only ("all connections from browsers ... flow via a relay server"; hole-punching cannot be ported) … iroh-blobs in wasm is MemStore-only" (`ring-apps-shelf.md:19-28`).
- **Media.**
  - "one LAN HTTP endpoint `hash + Range → 206`, never 200 | no sink speaks iroh" (`ring-apps-shelf.md:181`).
  - "MKV … no Chrome `<video>` support … HEVC in Chrome is GPU-only" (`ring-apps-shelf.md:193-194`).
  - "Unverified: Safari MKV" (:191).
- **A seek bar was proposed.** "time-to-first-byte for a Range request on a COLD connection over the relayed path, p90 under 1 s — measured with and without HTTP keep-alive" (`WORK_PLANE.md:582-585`). Also "`--relay-only` is a request, not a guarantee — gate every relayed reading on the `path=` line" (:561-563).
- **Retirement trigger.** At e1a43bcd2, ring-runtime and `ring_shim.js` retire on svrngs bar `phone-to-own-key` (`RING_SPEC.md:26`). At HEAD the row reads "the browser runtime (U5) | Demo B, split the pizza" (`docs/RING_SPEC.md:27`), which does not match svrngs's current U4/U5 numbering.
