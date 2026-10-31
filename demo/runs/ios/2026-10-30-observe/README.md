# HUMAN-ios-observe: what the phone said, 2026-10-30

`ralph/next/ios/STATE.md`, row HUMAN-ios-observe; predictions committed first
in `PREDICTIONS.md` (4a533559). The operator's iPhone (iOS 18.7 by its user
agent, Safari 26.6.1), the spike's tunnel at the ladder's framework, step
"periodic", lending Jellyfin from the scratch lender on this Mac
(`b8109133…`). The phone's samples and the day's trace are in `data/`
(`samples.jsonl`, `trace-today.log`, cut from the operator's "trace 12").

| Check | Predicted | Chance | Observed | Verdict |
|---|---|---|---|---|
| 1. Cold start | 3 to 6 s | 65% | The door's part is under 1 s; the whole, toggle to frame, about 10 s on WiFi and 20 to 30 s on cellular (the operator's count) | failed, and the door is not the cost |
| 2. Handoff | drops at least once | 60% | one gap of 5.6 s, under the music's buffer; nothing heard | passed, as a drop |
| 3. Locked | stops within a minute of its buffer | 70% | stops at the lock | passed, and sooner; in Firefox, Safari's could-not-judge |
| 4a. Upload scan | accepted | 75% | accepted ("Ready to Submit"), after one refusal for a missing icon | passed |
| 4b. TestFlight internal | installs, the tunnel runs | 80% | installed from TestFlight; 29 connections from the tunnel's key in a minute | passed |
| 5. The house's chat | not predicted | — | not asked yet | never-ran |

## 1. The cold start

The tunnel's start to its lend open is 0.3 to 0.7 s on WiFi and on cellular:
`start: a runtime on a thread of its own` at 20:02:49.917
(`data/trace-today.log:453`), the lend open by 20:02:50.6. On WiFi the path
went direct, about 10 MB in its first 10 s (`data/samples.jsonl`). On
cellular every start stayed on the relay: holepunching retried every 5 s and
never went direct, where the spike's S1 had. So the seconds past 3 are
Jellyfin's sign-in and buffering, and on cellular a relayed path.

**What it decides:** the row says over 3 s keeps the endpoint warm. The
endpoint's start is under 1 s of the 10, so warm would save under 1 s, and
`ios-lazy` keeps the endpoint on use; the operator agrees or not at
HUMAN-ios-agreed. Why cellular stays relayed is its own question, for
`ios-patches` or upstream.

## 2. The handoff

Off WiFi at 20:13:25.2: iroh's major link change
(`data/trace-today.log:800`), the relay lost and redialled; the door's
connection timed out at 20:13:29.2 (`:810`), `held()`'s 4 s; the lend
reopened at the sampler's next tick, 20:13:30.8. A gap of 5.6 s, inside the
music's buffer. Relayed for about 40 s, then direct on cellular at 20:14:20.

**What it decides:** a drop, so `ios-lazy` changes the door's reopen on a
phone. Of the 5.6 s, 4 are the silence limit and 1.6 the wait for the next
tick; reopening on the link change itself takes both, and is the fix to
try before a silence limit of the phone's own.

## 3. Locked

Safari could not open a lend at all (below), so this ran in Firefox on the
same phone: video and music stopped the moment the phone locked. Firefox's
console: `WebSocket connection to 'wss://usw1-1.relay.n0.iroh.link/relay'
failed: WebSocket is closed due to suspension`. The lender's log agrees: the
person "Fepp" carried a request every few seconds, and nothing after
21:28:02, 21:36:22 and 21:38:38. Firefox on iOS is WebKit, as Safari is, and
the suspension is iOS's, so Safari is expected the same; it was not run
locked after its fix.

**What it decides:** a lend does not play in the pocket from the page. That
is a reason for the app.

## 4. The upload and TestFlight

The app record `sh.svrnme.phonetunnel` (a throwaway name, the spike's bundle
ID, so the GA identifier stays free for the organization's account). The
first upload, 19:09, was refused before Apple's scan: no app icon
(`Missing required icon file … 120x120`, `CFBundleIconName is missing`).
105d3e88 gave the app the brand's icon; the second upload, 19:11 to 19:13,
succeeded, and processing ended "Ready to Submit, Expires in 90 days": the
scan took the 26 SystemConfiguration functions. The operator answered the
encryption question "standard algorithms" (the tunnel's lock links rustls
and ring, ed25519-dalek, curve25519-dalek and aes-gcm; nothing
non-standard), installed the build from TestFlight as an internal tester,
and played Jellyfin through it: the tunnel's key `fe352c1f` opened 29
connections to the lend at 02:25 UTC.

**What it decides:** no `ios-sc` row. The demo needs no App Review: an
individual's team gives the house the app by TestFlight's internal testers
(each needs a seat in App Store Connect) or an ad hoc build.

## Found on the way

- **iPhone Safari opened no lend since 0c9032ed.** The worker never got the
  pass: iPhone Safari loses any CryptoKey posted to a service worker. The
  probe is in `keyprobe/` (41fed32e). The operator chose the key's bytes in
  the worker's memory alone; 3946c340 is the fix, deployed to group.svrnme.sh
  at 18:52 (page version 2abc99cac186, the downloads byte-identical to the
  live ones). At 19:01:45 Safari opened films for the first time: the person
  "Asb", 232 requests to Jellyfin by 19:03.
- **A tunnel's key admitted as a computer makes its person "did not answer"
  on the page.** `people_lends` (crates/svrngs-node/src/node.rs) asks each
  computer of each person, and the tunnel's key never answers, so "lender"
  showed as not answering though films opened. The app's key as a browser's
  (`Device::Browser`, dialled never) avoids it: `ios-design` (a).
- **The gate's time counted sleep.** The fix's gate said "passed tests
  (10396 s)" against a limit of 1535 s: the Mac slept 3.16 h of the run
  (`pmset -g log`). b026dd84 keeps it awake while `check.sh` runs.

## Left

- Check 5, the house's two questions: who runs a VPN on their phone already,
  and who would take a TestFlight invite or give a phone's identifier. The
  row stays open until the answers are in.
- Check 3 in Safari, now that it opens.
