# Demo A's dry run — 2026-10-04

Not a run of Demo A: one computer, nobody watching, so it leaves no
`runs/a/<date>.json`. It measures what `demo/a.md` "Before the run" needs and
DESIGN §11 risk 7 asks: what Jellyfin's own web player needs to play through
the door without a member typing a password, and whether any byte a member
receives holds a secret of Sam's.

**How it ran.** `dry.sh` end to end, at svrngs `4048cbb` (product code as at
`ab34a70`), in 97 s:

```
cargo build -p svrngs
demo/runs/a/dry-2026-10-04/dry.sh /tmp/u3-8-run demo/runs/a/dry-2026-10-04/data
```

A scratch Jellyfin 12.0.0 on 127.0.0.1:8096 — the macOS app's server with its
own data dirs, since this host's podman machine was stopped; upstream's
`scripts/cw-media-demo.sh:8-140` runs the same server in podman — with three
90-second 720p H.264 films, Sam's admin account and one viewer account, each
password random and kept in the scratch dir. Sam's computer and Alex's are
two data dirs: `svrngs new House --as Sam`, `svrngs join <link> --as Alex`
with the `svrngs://join/` link `new` printed, `svrngs share 8096 --upload 20`
with the lend's secret on stdin, then Alex's bare command, which printed
`Sam's jellyfin, in your browser: http://127.0.0.1:22710/lend/jellyfin/`
(`data/alex-bare-command.out`). Alex's browser is headless Google Chrome
driven by `drive.mjs`, one fresh profile per way of opening it, each
recording a HAR with bodies; `scan.py` searches every header and decoded body
for each secret. A first, hand-run pass the same afternoon gave the same
results (Jellyfin's log there: the lend's token logged out at 13:43:38 and
again at 13:45:46, each on a member's sign-in).

## What it found

1. **Opened with nothing typed, Jellyfin asks for a sign-in.** The address
   redirects to `web/`, the web client finds the server, stores it, and shows
   "Please sign in" (`none-1-open.png`, `none.driver.log`). Risk 7 holds.

2. **A member signing in as the viewer breaks the lend for everyone.** The
   door replaces the client's `Authorization` with the lend's, whose
   `DeviceId` is the one Sam's computer signed in with, so Jellyfin takes the
   member's sign-in for that device signing in again and logs out the lend's
   own token (`scan.json`, `jellyfin_logged_out_in_order`: `lend_token_1`).
   The browser is handed a new viewer token in the sign-in's answer
   (`browser_token_1`, one body), which the door then overwrites with the
   dead one: every request after answers 401 and the page stays on "Please
   sign in" (`signin-2-signed-in.png`; `signin.requests.tsv`). The door's
   `/Users/Me` answered 200 before the sign-in, 401 after, and 200 again only
   once Sam declared a new token (`probes.json`). And the member typed the
   viewer's password, which only Sam's computer was to hold.

3. **With a placeholder sign-in in the browser, it plays, seeks, and holds
   nothing of Sam's.** When the web client's stored sign-in names the viewer
   with the token `door` (`drive.mjs` `seed`), the door's header does the
   rest: the home page lists the three films, play starts in 1,137 ms, and
   seeks to 60, 20 and 75 s each play again in 65, 123 and 7 ms
   (`seed.driver.log`, `seed-4-playing.png`). Jellyfin plays the file as it
   is (`stream.mp4?Static=true`, five Range requests answered 206), 14.9 MB
   received, no answer 400 or over. But something has to write that stored
   sign-in (`jellyfin_credentials` in the page's local storage); here the
   driver did, and no member can without help.

4. **An API key as the lend's secret is ruled out.** `/Users/Me` answers 400
   to it (the web client needs a user), a member reads the key back from
   `/Auth/Keys` (one body holding it), and a member can make a Jellyfin user
   (`users-with-api-key-lend.json`), which Demo A's "list of users unchanged"
   forbids. The probe's user and key were deleted after.

5. **Jellyfin's live socket does not cross the door.** A WebSocket upgrade to
   `/socket` answers 404 through the door and 101 on Sam's computer: the door
   drops `Upgrade` as hop-by-hop (`crates/svrngs-node/src/door.rs:113-128`).
   Playback did not need it; Jellyfin's remote control and live session
   updates do.

6. **Sam cannot make the lend's secret from what Sam is told.** `share`
   asks for what Jellyfin "takes in its Authorization header"; what works is
   `MediaBrowser Client="…", Device="…", DeviceId="…", Version="1",
   Token="<token>"`, the token from signing the viewer in through Jellyfin's
   API (`dry.sh` `mint`). Here a script did it.

7. **No secret in anything a member received, or in either computer's
   svrngs output.** Sam's password, the viewer's password, Sam's admin token,
   every lend token and the API key: zero matches in every response header
   and body of all three ways, and in both computers' svrngs logs and output
   (`scan.json`). The one secret in a received byte is the viewer token
   Jellyfin handed the signing-in browser (finding 2). Jellyfin's own log on
   Sam's computer names each token it logs out, in clear. Jellyfin's list of
   users ends as it began (`users-before.json`, `users-after.json`).

Not measured: three members at once. Every member reaches Jellyfin with the
lend's one `DeviceId` in the header, so Jellyfin's sessions see one device;
direct play (finding 3) involves no transcode for it to kill, a transcode
was not tried. One computer, one member, one run of each way: not a rate.

## The files

`data/`: per way of opening (`none`, `seed`, `signin`) the HAR with bodies
dropped and every credential replaced by its name in `scan.json` (`<lend_token_1>`),
the requests as `*.requests.tsv`, the driver's log and screenshots;
`scan.json`, the secret search; `probes.json`, the door's answers around the
sign-in, the API key and the socket; the users before, during the API-key
probe and after; what each svrngs command printed, Sam's link replaced.
The full HARs and the secrets stayed in the scratch dir.
