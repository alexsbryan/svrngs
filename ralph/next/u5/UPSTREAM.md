# U5 — what upstream and svrngs have for Films (the seat's inventory notes, 2026-10-05)

Read-only survey of commonwealth-ai at e1a43bcd2/HEAD, svrngs at 49c3be4 and tag e0-bot. U5's inventory row writes DESIGN §8 "U5's inventory" from it, re-verifying every path:line it uses. Input to the queue, not a record.

# U5 Films inventory: what can be forked, what must be respected, and the gaps

## Scope and bottom line

**What was read, and at which versions**
- **commonwealth-ai** was read at `e1a43bcd2`. HEAD is now `b1d992295`.
  - The media code moved between the two, and every file I compared by blob hash is byte-identical.
  - `commonwealth/crates/*` → `cmnwlth/crates/*`
  - `sovereign/crates/sovereign-cli-mesh` → `cmnwlth/crates/sovereign-cli-mesh`
  - `sovereign/crates/sovereign-daemon` → `svrn/crates/sovereign-daemon`
  - `sovereign/crates/sovereign-desktop` → `clients/desktop`
  - `sovereign/apps/*` → `cmnwlth/apps/*`
  - Three files differ at HEAD:
    - `docs/RING_APP_LIBRARY.md`: path strings only. The diff hunks show the same line counts, so the line numbers below hold.
    - `scripts/ring-room-demo.sh`: changed (16 lines differ); I did not check what.
    - `quality/campaigns/house-mesh.toml`: changed (2 lines differ); I did not check what.
- **svrngs** was read at HEAD `49c3be4`.
  - **Its `docs/DESIGN.md` moved while I worked.** `d3a39b7` inserted U4's inventory into §8 and five calls into §3. Every DESIGN line below is at HEAD and has been re-checked. DESIGN has not changed since `e32a4ac`.
  - The code I cite (`svrngs-node`, `svrngs` command, `apps/`, `demo/`) is unchanged from `632d5d3` to HEAD. The running U4 loop has uncommitted edits in `svrngs-core` and `redeem.rs` only.
- **`e0-bot`** was read with `git show e0-bot:…`. Its only app is `apps/expenses`; there is no Films code there.
- **Outside both repos**, `~/dev/house-mesh/library.py` is listed because commonwealth-ai cites it (`quality/campaigns/house-mesh.toml:39,58`). It is the only merge-by-provider-id code that exists anywhere.

**Bottom line**
- Nothing upstream is a per-title shelf, a player, or progress kept in a record. The closest pieces are:
  - `library.py`: merges by IMDb id, CLI only, outside both repos.
  - commonwealth-ai's `LibraryView.svelte`: one card per member's library, opening Jellyfin's own web UI.
  - `viewer.rs`: Jellyfin account and policy calls.
  - svrngs's own dry run: the only record of the real Jellyfin web client's API sequence through the door.
- No code anywhere uses `MaxStreamingBitrate`, `PlaySessionId`, `master.m3u8`, Tmdb ids, or `UserItems/Resume`. They appear only inside the dry run's HAR (data captured from Jellyfin's own web client).

**How sizes were counted** (non-blank, non-comment lines). Use `"${R}:path"` in zsh, because `$R:s…` is a zsh history modifier.
- Rust: `git show "${R}:<path>" | sed '/^#\[cfg(test)\]/,$d' | grep -cvE '^\s*(//|$)'`. This drops inline tests and all `//`, `///` and `//!` lines.
- JS/TS/Svelte: `grep -cvE '^\s*(//|<!--|/\*\*|\*|$)'`
- sh/py: `grep -cvE '^\s*(#|$)'`
- Ranges: insert `sed -n 'a,bp'` before the grep.

## (a) Table

**Jellyfin API use**

| Upstream path:lines (at `e1a43bcd2`; HEAD path if moved) | What it does | Size | Depends on | Constraints it encodes |
|---|---|---|---|---|
| svrngs `demo/runs/a/dry-2026-10-04/dry.sh:60-95` | Wizard over the API (`:66-72`); `POST /Users/AuthenticateByName` (`:74`); `POST /Users/New` viewer (`:76`); `GET /Items?Recursive=true&IncludeItemTypes=Movie` (`:79`); search by `&SearchTerm=One` (`:83`); `mint()` signs the viewer in with a fixed device header and writes the lend secret `"<LEND_DEVICE>, Token=\"<t>\""` (`:88-95`), piped to `svrngs share 8096 --upload 20` on stdin (`:94`) | 134 whole; mint 8 | curl, python3, macOS Jellyfin.app 12.0.0, svrngs binary | Header `MediaBrowser Client="…", Device="…", DeviceId="…", Version="1"` plus `, Token="…"` (`:60-61`, `:88`). The lend's DeviceId is fixed (`svrngs-lend-sam`, `:88`). An API key is ruled out (`:131-142`) |
| svrngs `…/dry-2026-10-04/drive.mjs:39-50` | Seeds Jellyfin web's `jellyfin_credentials` in localStorage with placeholder token `door`, so the door's header does the auth | 90 | playwright-core, Chrome | The web client needs a stored sign-in; something other than a member must write it |
| svrngs `…/data/seed.requests.tsv`, `seed.har` (data, not code) | Jellyfin web's real call sequence through the door (see (b)) | — | — | Shows which calls carry `Authorization` and which (images, video) carry none |
| commonwealth-ai `sovereign/crates/sovereign-cli-mesh/src/mesh_media/viewer.rs:36-463` (HEAD `cmnwlth/crates/sovereign-cli-mesh/src/mesh_media/viewer.rs`) | `provision()` (`:304-463`): `GET /Users`, `POST /Users/New`, `POST /Users/Password?userId=`, `GET /Users/{id}`, `POST /Users/{id}/Policy` with the policy merged (`:151-175`), read back (`:180-196`), then `AuthenticateByName` (`:438-452`). `already_provisioned()` (`:216-296`) proves a token with one `GET /Users/Me`. Credential is `MediaBrowser Token="{token}"` (`:461`) | 323 (530 with tests) | reqwest, serde_json, uuid, tracing | An `/Auth/Keys` key is admin-equivalent (`:6-12`). Policy POST replaces rather than patches, and needs `AuthenticationProviderId` and `PasswordResetProviderId` (`:23-27`, `:131-150`). A non-admin cannot reset a password or change a policy (403 "Invalid user or password entered.", `:198-215`). `MaxActiveSessions` is only preserved in a test fixture, never set (`:669`, `:708`). **It creates a user, which Demo A forbids** (`demo/a.md:6-7`) |
| `…/mesh_media/offer.rs:94-109`, `:201-231` (HEAD `cmnwlth/…`) | `WELL_KNOWN_ORIGINS` 8096 then 8920, `probe_origin`; the offer flow that calls `provision` | 296 | toml_edit, commonwealth_media | Already forked as `crates/svrngs/src/well_known.txt` (8096 only) |
| `commonwealth/crates/commonwealth-media/src/presence.rs:1-96` (HEAD `cmnwlth/…`) | Reads Jellyfin `/Sessions` JSON (`UserId`, `NowPlayingItem`) into IN_USE / FREE | 43 | serde | "Every member reaching this library arrives as that one viewer user" (`:17-20`). A missing answer is `None`, never FREE (`:22-27`) |
| `commonwealth/crates/commonwealth-rails/src/presence.rs:59-120` (HEAD `cmnwlth/…`) | `GET http://{origin}/Sessions` with the house credential every 10 s (`:45`, `:78-83`) | 113 | reqwest | A viewer token sees only its own sessions (`ralph/DECISIONS.md:225`, A40) |
| `scripts/cw-media-demo.sh:31-78` (same path at HEAD) | AUTH header (`:31`); wizard; picks an API key by app name (`:62-68`); declares `MediaBrowser Token=` (`:73`); `/Library/Refresh` (`:139`) | 102 | podman, curl | `Authorization`, never `X-Emby-Authorization`; Jellyfin 12 reads only the former (`:35-39`). It dropped X-Emby-Token, X-MediaBrowser-Token and `?api_key=` (`:56`). The wizard's own scan is cancelled (`:136-138`) |
| `scripts/ring-room-demo.sh:1136-1192` (HEAD differs) | `jelly()` header (`:1137`); Items list (`:1165`); first byte of `/Videos/$item/stream?static=true` with `-r 0-65535` (`:1171-1172`); `POST /Sessions/Playing {"ItemId","PlayMethod":"DirectPlay","CanSeek":true,"PositionTicks":0}` (`:1183`); `GET /Sessions` (`:1185`); `/Sessions/Playing/Stopped` (`:1192`) | 56 (range) | podman, node_curl | The only upstream progress-report call: one shot, admin token |
| `.canon/sources/notes/invariant/35104a7e.md:3-13` (same path at HEAD) | Probe record for Jellyfin 12 auth | prose | — | See (b) |
| `quality/campaigns/house-mesh.toml:53-71` | Measured Items with ProviderIds, stream, and Range (see (b)) | prose | — | — |

**Shelf and catalogue across libraries**

| Upstream path:lines | What it does | Size | Depends on | Constraints it encodes |
|---|---|---|---|---|
| `~/dev/house-mesh/library.py:72-102` (outside both repos; cited at `house-mesh.toml:39,57-59`) | One fan-out `GET /Items?Recursive=true&IncludeItemTypes=Movie&fields=ProviderIds`; merge `key_id = item.get("ProviderIds",{}).get("Imdb") or item["Name"]` (`:91`); `have[key].append((member, item.Id, item.Name))`; a missing library becomes an "absent" row (`:81-87`); play = `{url}/Videos/{id}/stream?static=true` (`:102`) | 102 (docstrings counted) | stdlib; svrn daemon `/v1/mesh/media/fanout` | Merge key is IMDb, fallback the title; there is no Tmdb. "Rewrite this. It is an example" (`:13`) |
| `commonwealth/crates/commonwealth-media/src/fanout.rs:1-19`, `:116-193` (HEAD `cmnwlth/…`) | The same request to every offering member, one attributed row each | 353 | commonwealth_core, transport | "No merge, no dedup, no streams… a shim merges by provider id because only it knows what an item is" (`:14-17`). Body cap 4 MiB (`:40`); timeout 10 s (`:37`) |
| `…/commonwealth-media/src/reach.rs:71-121`, `:180-240` | Offers list, `player_url` per member, refusals by name | 353 | iroh transport | The URL "stays valid for the life of the process" (`:5-8`). Self is refused: "'{}' is this node — its {} is already local" (`:94-95`). svrngs `Node::lends` already has this shape |
| `quality/campaigns/ring-apps-shelf.md:183-196` (+ `:19-28`, `:181`, `:242-246`, `:257`) | Library verdicts (see (b) and (d)) | 12 (play section) | — | — |
| `docs/RING_APP_LIBRARY.md:740-903` (§20) | Held, lent, carried; injected credential (`:802-811`) | prose | — | — |
| svrngs `e0-bot:docs/DESIGN.md:91-92`, `:135`, `:536-587`, `:825`, `:951-954`, `:1020-1027` | Superseded Films and TV design: `group.request` (GET only), `group.play`, masking the credential in bodies, a play session id per play | prose | — | Build from it only what a §8 unit names (DESIGN `:8-10`) |

**UI**

| Upstream path:lines | What it does | Size | Depends on | Constraints it encodes |
|---|---|---|---|---|
| `sovereign/crates/sovereign-desktop/src/lib/components/library/LibraryView.svelte:61-119`, `:245-279` (HEAD `clients/desktop/src/lib/components/library/LibraryView.svelte`, identical) | "Libraries on the mesh": a card per member, "`{o.peer}'s library`" (`:264`), chip "in use right now" (`:268`), "offered to:" (`:271-273`); polls every 10 s (`:61-63`, `:114-118`); probes before opening (`:78-94`); title "`${o.peer} is watching this library right now`" (`:110`) | 76 in ranges (413 whole incl. CSS) | Svelte 5 runes, Tauri `invoke`, `api.ts` | A card per library, not per title. It opens Jellyfin's own web UI in a browser tab (`:93`). There is no poster grid, search or player |
| `…/src-tauri/src/mesh_commands.rs:644-793` (HEAD `clients/desktop/…`, identical) | `media_offers`: a row in use carries no `player_url` (`:707-714`); probe restricted to loopback http (`:746-793`) | 96 | reqwest, tauri | "A refused reach is a row with its reason, not a dropped row" (`:696-697`) |
| `…/src/lib/api.ts:1325-1350` (HEAD `clients/desktop/src/lib/api.ts` differs elsewhere) | `MeshMediaOffer` type | 15 | — | — |
| `sovereign/crates/sovereign-desktop/tests/e2e/demo/b9-shelf.demo.spec.ts` | Not a media shelf: a shelf of notebooks | — | — | — |

**Patterns for record-shaped apps**

| Upstream path:lines | What it does | Size | Depends on | Constraints it encodes |
|---|---|---|---|---|
| `sovereign/apps/ring-doc/adapter.js:307-436` (HEAD `cmnwlth/apps/ring-doc/adapter.js`) | Live-lane envelope `{kind, doc, awareness}`; skips other kinds (`:319-321`) | 62 (201 whole) | Yjs | Names on the live lane are self-asserted (`:311-317`); signed attribution comes from acts only |
| svrngs `apps/expenses/expenses.js:120-133` and `canary.mjs:35-52` | `actsOf(log)` reads `by = on_behalf_of ?? person`; append is `{op:'record', payload, on_behalf_of}` via `?namespace=` | 125 / 80 | node | JSON-only state; who an act speaks for is the signer or `on_behalf_of`, never a payload field (`expenses.js:19-21`) |

**svrngs surface Films would call**

| svrngs path:lines | What it does | Size | Depends on | Constraints it encodes |
|---|---|---|---|---|
| `crates/svrngs-node/src/node.rs:61-79`, `:231-268`, `:381-432` | `Lent{person, offered, address}` and `Lends{lent, unanswered}`; `Node::lends` asks every standing key's `GET /lends`; address is `{bridge}/lend/<name>/` (`:421`) | 95 | iroh, reqwest | Rust only; skips this computer's own key (`:361`) |
| `crates/svrngs-node/src/door.rs` (whole) | The door: `GET /lends` (`:262-315`), `/lend/<name>/…` (`:447-544`), the busy body (`:554-568`), header rewrite (`:368-399`) | 537 | axum, hyper | See (b) and (c) |
| `crates/svrngs-node/src/api.rs:35-54` | Loopback routes: `/v1/rail/{append,log,live,people}` only | 103 | axum | Callers must be on loopback (`:56-90`) |

## (b) Measured facts, verbatim

**svrngs dry run** (`demo/runs/a/dry-2026-10-04/README.md`, Jellyfin 12.0.0, one computer)
- `:35-37`: "Opened with nothing typed, Jellyfin asks for a sign-in."
- `:39-46`: "The door replaces the client's `Authorization` with the lend's, whose `DeviceId` is the one Sam's computer signed in with, so Jellyfin takes the member's sign-in for that device signing in again and logs out the lend's own token."
- `:47-49`: "The door's `/Users/Me` answered 200 before the sign-in, 401 after, and 200 again only once Sam declared a new token" (`data/probes.json`).
- `:52-61`: "the home page lists the three films, play starts in 1,137 ms, and seeks to 60, 20 and 75 s each play again in 65, 123 and 7 ms … Jellyfin plays the file as it is (`stream.mp4?Static=true`, five Range requests answered 206), 14.9 MB received".
- `:63-67`: "`/Users/Me` answers 400 to it [an API key] … a member reads the key back from `/Auth/Keys` … and a member can make a Jellyfin user".
- `:69-73`: "A WebSocket upgrade to `/socket` answers 404 through the door and 101 on Sam's computer: the door drops `Upgrade` as hop-by-hop … Playback did not need it; Jellyfin's remote control and live session updates do."
- `:75-79`: "what works is `MediaBrowser Client="…", Device="…", DeviceId="…", Version="1", Token="<token>"`, the token from signing the viewer in through Jellyfin's API".
- `:81-88`: zero secret matches "in every response header and body of all three ways". "The one secret in a received byte is the viewer token Jellyfin handed the signing-in browser".
- `:90-93`: "Not measured: three members at once. Every member reaches Jellyfin with the lend's one `DeviceId` in the header, so Jellyfin's sessions see one device; direct play … involves no transcode for it to kill, a transcode was not tried."

**The web client's calls through the door** (`data/seed.requests.tsv`, `data/seed.har`)
- `:129`, `:136-138`: `GET /Items/{id}/Images/Primary?fillHeight=…&fillWidth=…&quality=96&tag=…` answered 200 with the browser sending no Authorization. The door injected it, so posters work as a plain `<img src>`.
- `:130-132`: `/UserItems/Resume?userId=…&mediaTypes=Video` and the like. This is Jellyfin's resume list for the one shared viewer user.
- `:154-155`: `/Users/{userId}/Items/{itemId}`.
- `:171`: `POST /Items/{id}/PlaybackInfo` → 200, 2131 bytes.
- `:175`: `GET /Videos/{id}/stream.mp4?Static=true&mediaSourceId=…&deviceId=…` → 206 for `bytes=0-`. The web client's deviceId rides the query.
- `:176`: `POST /Sessions/Playing` → 204.
- `:182-187`: `/Sessions/Playing/Progress` → 204.
- `:183`, `:185`, `:188`: further Range reads → 206.
- The PlaybackInfo request body in `seed.har` (from HeadlessChrome/151 on macOS):
  - `"MaxStreamingBitrate":189573062`.
  - DeviceProfile `"MaxStreamingBitrate":120000000,"MaxStaticBitrate":100000000`.
  - DirectPlay includes `mkv` with `h264,hevc,vp9,av1` and `hls`.
  - Transcoding profiles: `mp4/hls` and `ts/hls`.
- `Sessions/Playing` body carried `"PlaySessionId":""` and `"PositionTicks":0`. Response bodies were dropped from the committed HAR.
- The web client's DeviceId is base64 of `<User-Agent>|<ms timestamp>` (decoded from `seed.requests.tsv:107`).

**commonwealth-ai probes**
- `.canon/sources/notes/invariant/35104a7e.md:3`: "Jellyfin 12.0.0 authenticates ONLY via `Authorization: MediaBrowser Token="<key>"`… GET /Items: X-Emby-Token 401, X-MediaBrowser-Token 401, ?api_key= 401, no auth 401, Authorization 200 … Distinct from the already-banked finding that `X-Emby-Authorization`, the CLIENT-identification header, is also dead."
- Same note, `:9`: "The value has an inner space and two quotes and both are load-bearing."
- `quality/campaigns/house-mesh.toml:57-63`: "`/Items?Recursive=true&IncludeItemTypes=Movie&fields=ProviderIds` -> 200, items carrying ProviderIds.Imdb. library.py merges on that key and it is in the real payload … `/Videos/{id}/stream?static=true` through the peer bridge -> 200, 122,048,071 bytes at 5.25 MB/s, and `Range: bytes=60000000-68388607` -> 206 with exactly 8,388,608 bytes."
- `house-mesh.toml:73-75`: "STILL NEVER-RAN … the merged catalogue."
- `quality/campaigns/ring-room.toml:176`: "listed 8.39 s, narrowed 10.47 s, first byte 0.33 s on one host … Cross-host first byte was never measured".
- `ralph/DECISIONS.md:225` (A40): "the presence poll reads `/Sessions` with the read-only viewer's credential, which sees only its own sessions".
- `viewer.rs:6-12`: an `/Auth/Keys` key "has a zero `UserId`, passes elevation, and answers `GET /Auth/Keys` with 200".
- `viewer.rs:203-208`: "`POST /Users/Password` answered `403 "Invalid user or password entered."` — that string is the non-admin branch".
- Relay bounds (no browser involved), svrngs DESIGN `:576-578`: "a 182 ms floor and 3.7 MB/s bulk through n0's relay … a cold connection's setup, paid per seek, p50 189 ms and p90 1,327 ms on a LAN".
- `sovereign/bench/media_bridge/README.md:4-6` (HEAD `bench/lanes/media_bridge/README.md`): the relayed bar is "could-not-judge".
- svrngs `ralph/REVIEW_FINDINGS.md:869-883` (#33): with two nodes on one host, "iroh moves the connection dialled on loopback onto this computer's private address, which `guess` … takes to be home". Fixed with `RelayConfig::on_loopback_only`. Any Films test on one host needs the same, or viewers get no share.

## (c) Gaps

### svrngs surface (infra)

1. **An app cannot list what the group lends.**
   - `api.rs:36-54` serves only append, log, live and people.
   - `Node::lends` (`node.rs:231-268`) is called once, when the bare command starts (`cli.rs:690-697`), and only printed (`cli.rs:553-565`).
   - U4's u4-5 adds a member-protocol route over iroh, not a loopback one (`ralph/next/u4/STATE.md:8`).
   - So Films on a computer can learn neither the lends nor their bridge addresses. A bridge to a computer that was offline at start is never made (`iroh.rs:210-265`, built on demand).
2. **An app cannot find the record API or the group ids.**
   - The API binds port 0: "A port an app finds without being told waits for an app that needs one" (`cli.rs:110-113`).
   - Every route requires `?namespace=` (`rail.rs:78-108`). No route lists groups. The bare command prints group names, not ids (`cli.rs:539-551`).
3. **"This computer's person" is not available to a local app.**
   - A page opened on one's own computer never passes the door, so it gets no `x-svrngs-person`.
   - `/v1/rail/people` does not mark which person is this computer's (`rail.rs:378-399`).
4. **The lender's own use of their lend has no path.**
   - `Node::lends` skips this computer's own key (`node.rs:361`); upstream refused self too (`reach.rs:94-95`).
   - The lend's secret lives only in the door's store. So Films on Sam's computer cannot reach Sam's Jellyfin through §3.
5. **One app's page reaching another lender's lend, under the viewer's own key.**
   - The door names and counts the dialling key (`door.rs:460`, `:473`; `limit.rs` counts per actor).
   - If one Films server proxied every Jellyfin, every lender would see that server's person, which breaks shares and limits. Call 4 says "No node proxies another's lend" (DESIGN `:132-135`).
   - On a computer, the per-viewer path exists (the bridge address), but gap 1 hides it.
   - On a phone, U4's runtime serves "a lent service" under one scope "to the lender's door (`/lend/<name>/<path>`)" (`STATE.md:9`). Nothing is designed for a Films page addressing (lender key, lend name) for several lenders.
6. **Choosing the bigger share before playing.**
   - The list item `Offered` carries name, kind, limit and using, but not the declared upload (`lend.rs:264-272`).
   - The share arrives only on a door answer, after the person has been counted for 30 s (`door.rs:519-539`, `limit.rs:38`). Probing a lender occupies a slot there.
   - A viewer guessed to be at home, or a lend declaring no upload, gets no share at all (`door.rs:525-536`).
7. **CORS.** The door adds no CORS or `Access-Control-Expose-Headers` (`door.rs:681-705`). A page on another origin cannot read `x-svrngs-declared-share-bps` or fetch the JSON (`door.rs:100`). Jellyfin's own CORS behaviour is unmeasured in both repos.
8. **The secret in response bodies.**
   - The door drops only headers that echo the secret (`door.rs:31-32`, `:422-443`).
   - e0-bot's design masked bodies because "Jellyfin writes its token into playback info and into the HLS playlists' subtitle, trickplay and variant URLs" (`e0-bot DESIGN:583-586`).
   - Only direct play was scanned clean. The transcode / HLS path is unmeasured and may break U3's bar, "the lender's secret appears in no byte a member receives" (DESIGN `:596`).
9. **A device id per person and per play.**
   - The door replaces the whole `authorization` header (`door.rs:113-119`, `:368-399`), and that header carries the lend's one DeviceId.
   - Jellyfin 12 reads no other auth header (`35104a7e.md:3`; `cw-media-demo.sh:35-39`).
   - So Films can carry `deviceId` / `PlaySessionId` only in query strings or bodies.
   - Whether Jellyfin keys its transcode kill on those is unmeasured. DESIGN cites `DynamicHlsController.cs:1504` "in the source read" (`:394-396`), but neither repo holds that source.
   - If query ids do not suffice, a per-person device id needs the door or the lend to vary it. That is infra that may not learn Jellyfin's header grammar (`scripts/vocab_check.py`; `apps/films/nouns.txt:2`). This is a decision, not a fix.
10. **No queue for "you're next".** "nothing queues" (DESIGN `:507`), and nothing announces a freed slot. Films can only poll `using`, which needs gap 1.
11. **The live lane cannot carry a trustworthy "now playing".** A drain returns `payloads` and `dropped` with no sender (`rail.rs:604-605`), from one buffer per computer that any app drains (`rail.rs:428-440`). Anything Films put there would be self-asserted and contended.
12. **A phone's progress writes.** A phone cannot call any node's loopback API (`api.rs:56-90`). Its progress must be appended by an app server, `on_behalf_of` the person the door verified (`rail.rs:185-197`). Where that server runs for a phone user is undecided.
13. **The WebSocket does not cross the door** (`door.rs:120-129`). Films must poll, never use `/socket`.

### Films app

1. **The shelf.** Nothing per title exists in either repo.
   - Fork `library.py`'s merge: IMDb, then title; extend with Tmdb, which has never been measured.
   - Fork `LibraryView`'s attribution, in-use wording and polling.
   - Posters: `/Items/{id}/Images/Primary?…`, measured working without auth from the browser.
2. **Search.** Only `&SearchTerm=` (`dry.sh:83`) exists.
3. **Playback.** No code builds PlaybackInfo with `MaxStreamingBitrate` set from the share, chooses direct play versus `master.m3u8`, generates a `PlaySessionId`, or reports `/Sessions/Playing`, `Progress` and `Stopped` with per-person ids.
   - The only templates: the web client's HAR bodies, and `ring-room-demo.sh:1183`, `:1192`.
   - Reporting to Jellyfin under one viewer user mixes everyone's resume data (`seed.requests.tsv:130-132`).
4. **Progress in the record.** No prior art upstream. Constraints:
   - Integers only: "pick a unit and use an integer" (`payload.rs:32-44`; `RING_APP_LIBRARY.md:157-160`). Ticks or milliseconds are fine.
   - At most 64 KiB per payload (`payload.rs:65`).
   - "Writes faster than one a second run the stamps ahead of the clock, one second per write" (DESIGN `:359-360`).
   - Sync rounds every 60 s, nudged on append (`ring_sync.rs:103`).
   - Upshot: write on pause and stop or at long intervals, keyed per person per merged title so it follows across lenders (DESIGN `:397-399`). Shape acts like Expenses: `{op:'record', payload:{kind,…}, on_behalf_of}`, read back with `actsOf`.
5. **Minting the credential on the lender's computer.** Build it from `dry.sh:88-95`, not `viewer.rs`, which creates a user.
   - Fork `viewer.rs:177-196` and `:216-296` (prove the token with `GET /Users/Me`; read `MaxActiveSessions`). Setting `MaxActiveSessions` needs admin (`viewer.rs:198-215`).
   - Pipe the header into `svrngs share 8096` on stdin (`cli.rs:626-653`).
   - Supply `--refuse` paths (`cli.rs:88-96`), e.g. `/Users/AuthenticateByName`, `/Sessions/Logout` (`e0-bot DESIGN:586-587`), `/Auth/Keys`, `/Users/New`. These must live in Films' code or instructions, because the nouns gate keeps Jellyfin out of infra.
6. **Busy into a choice.** Map the 503 body `{"error","using","limit"}` (`door.rs:554-568`) to other lenders of the same title.
7. **Codecs on phones.** The evidence conflicts.
   - The shelf says "MKV … no Chrome `<video>` support … HEVC in Chrome is GPU-only" (`ring-apps-shelf.md:193-194`).
   - Jellyfin web's profile in headless Chrome 151 listed mkv h264/hevc as direct play (`seed.har`).
   - Safari MKV is unverified (`ring-apps-shelf.md:191`).
   - **HLS on iOS Safari versus MSE on Chrome: not found.** I searched `MediaSource`, `ManagedMediaSource`, `SourceBuffer`, `hls.js` and `m3u8` across both repos; there is only the shelf verdict at `ring-apps-shelf.md:190`.

## (d) Recorded decisions that bind U5

**From svrngs `docs/DESIGN.md` at HEAD**
- `:100`: "An app that needs anything else is a decision, not a convenience."
- `:89-98`: the whole surface an app gets.
- `:33-37`: words a person reads: names, *computer*, *phone*, *the chat*; never ring, member, key, sync, journal, node or relay.
- `:215-219`: the nouns gate; "a change that serves Films and breaks one [canary] fails the build".
- `:223`: the line ceilings exclude tests and `apps/`.
- `:376-382`: Films is the one experience, and "Nothing else is built until Demo B passes, except the two canaries".
- `:383-405`: who fixes what (quoted in full in this inventory's source).
  - `:390-393`: "asks Jellyfin for no more than the share (`MaxStreamingBitrate`), plays the file as it is whenever the share and the phone allow … transcodes only when it must".
  - `:394-400`: one device id and one play session per person and per play; progress and watched kept in the record; "The lend's limit sits below the account's own session cap (`MaxActiveSessions`)".
  - `:401-404`: the busy choice, and play "from whoever can give you the bigger share".
- `:127-135`: call 3 (the service worker holds the endpoint) and call 4 ("No node proxies another's lend").
- `:501`: lends are asked of each computer, never stored.
- `:505`: an app's act is signed by its member's node, "with `on_behalf_of` naming the person the door verified when the app serves someone else".
- `:507`: "nothing queues".
- `:516-518`: "Localhost is the trust boundary".
- `:519-520`: "Jellyfin is GPL-2.0-only, so svrngs talks to it at arm's length, over its API" (also `ring-apps-shelf.md:187`: "cannot be vendored, forked or linked into AGPL-3.0").
- `:610`: U5's row and its bar.
- `:646`: watching together apart waits for its trigger; `:649`: files and live channel beyond U3 wait; `:661`: the per-process tie waits.
- `:725-734`, risk 7: "Demo A plays in Films, after U5, which never needs that page; no unit builds a way into Jellyfin's own player."

**From `demo/a.md`**
- `:3-7`: three of three people, each seek playing again "within five seconds, whoever else is seeking"; "Sam's Jellyfin password is in nothing any of them received, and Jellyfin's list of users is the same after the run as before."
- `:10-12`: there is one viewer account whose password only Sam's computer holds.
- `:30-32`: the steps from the local address on are "Films' to write when U5 lands".

**From U4's queue**
- `ralph/next/u4/PROMPT.addendum.md:50-53`: the page never says *film* or *library*.

**Prior designs (superseded; build only what a §8 unit names)**
- e0-bot DESIGN `:572-573`: "a play session id per play, so one person's seek never stops another's transcode".
- e0-bot `:586-587`: refuse `/Sessions/Logout`, "which would sign the whole group out".
- e0-bot `:878`: "`Remote-User` and per-person accounts on lent services" wait for "a named service that needs them".
- e0-bot `:538`: "Video over the project's relay is capped, and the app says so."
- Upstream operator posture, mesh-era and not adopted by svrngs, `quality/campaigns/ring-room.toml:232`: "the holder surfaces the library READ-ONLY and decides WHEN it is offered … a library the holder is watching reads 'in use'". svrngs has no signal that the holder is watching.
- `RING_APP_LIBRARY.md:189`: "Every act carries `kind` and `v`."
- `RING_APP_LIBRARY.md:808-811`: mapping a trusted proxy header onto per-person identity is "From memory — verify per app."
