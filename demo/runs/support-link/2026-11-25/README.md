# Support link, the lean lab run — 2026-11-25

svrn-docs `rings/current/SUPPORT_LINK.md`'s playback PoC, run as
`SUPPORT_LINK_API.md` §2 says to run it first: on today's primitives, with
nothing built in svrngs. The player's diagnostic port is lent to a lab group
of two (customer, desk), and the desk's agent reads it as Pantry reads a
friend's tool. svrngs `e5d9813b`, release build. One run of each row, so
these are not rates (AGENTS.md 7).

## Pinned

| What | Pin |
|---|---|
| Player | hls.js 1.7.4, sha256 `cfc70cc9…3bbe` (`make-fixture.sh`), stock config except Test fix's `autoLevelCapping` |
| Browser | Playwright Chromium 147.0.7727.15, headed (`record/browser.txt`) |
| Asset | `make-fixture.sh`, fixture rev 1: 14 min testsrc2, 24 fps, aligned 2 s H.264 fMP4 segments at 540p/720p/1080p (1.5/3/6 Mbit/s), 420 each |
| Origin | `origin.mjs`, uncached, on 127.0.0.1. Its log is the operator's witness; the agent never saw it |
| Bar | `player.html` `judge`: on `requestVideoFrameCallback`, from the first frame ≥ 12:30, 12:55 reached within 35 s, and no gap ≥ 2 s from 12:40 through 12:50 |

## 1. The proposal's incident doesn't qualify

With 1080p segment 381 (12:42–12:44) answering 404, stock hls.js 1.7.4 marks
that segment's 1080p copy as failed, loads the 720p copy within ms
(`fragLoadError 404 … action=2`) and plays through. Worst gap 0.07 s, 12:55
in 25 s, three runs of three (`record/qualify-broken.json`). By the
proposal's own rule the fixture doesn't qualify. A truncated 200 heals the
same way (`record/qualify-truncated.json`).

A 1080p segment that **hangs** (200 headers, then no bytes) qualifies. hls.js
retries the same rendition on each `fragLoadTimeOut` (`action=5`) and the
picture freezes at 761.96 s for good (`record/qualify-hang.json`). The rest
of this run uses the hang. That's a substitution for the proposal's 404, and
whether the proposal adopts it is the operator's call.

| Run (1080p seg 381 hangs) | Result | Record |
|---|---|---|
| A: normal player | failed: 28 s gap at 762.0 s, never resumes | `table-hang.json` |
| B: Test fix, 720p cap | **passed**: 12:55 in 25.0 s, worst gap 0.05 s | `table-hang.json`, `lab-run.json` |
| A again, normal | failed again; the cap didn't stick | `table-hang.json`, `lab-run.json` |
| Healthy, no cap | passed, 1080p throughout | `qualify-healthy.json` |
| Lookalike: seg 381 404 at every rendition, capped | failed, as it should | `table-lookalike.json` |

The first qualification pass showed a stall that my harness made: an
inverted timer paused every attempt at 15 s
(`record/qualify-broken-harness-bug.json`). The events showed the player
past the segment with 89 s of buffer ahead, which is how it was caught.

## 2. The link run: desk agent with the link, control without

The customer's svrngs lent `app.mjs`'s diagnostic port (`svrngs share 8403
--group Lab --name player-support --program node`). The desk joined Lab and
turned Play on for the lend. Two fresh agents got the same ticket
("Playback freezes around 12:43") and the same playbook line on what Test
fix does.

- **Desk agent** (only tool: `desk.mjs`, the desk's own svrngs). Three
  reads (`record/diagnostic-calls.jsonl`) and 51 s from the ticket to the
  diagnosis: *1080p segment 381 (762–764 s) never answered, four
  `fragLoadTimeOut`s at 10 s intervals, no switch down, neighbours in
  3–4 ms, so not the connection*. It proposed the cap, with why, before any
  result. After the customer's press it read `/replay` once. 1 min 56 s
  from the ticket to verified playback. No screen share, no log export.
- **Control** (ticket only, no tools, `record/control-agent.md`) ranked
  five hypotheses by how common each is. It asked four questions plus a
  screenshot, and asked for two replays (about half an hour). It said it
  couldn't see why without the player's state.
- Both agents' words are the agents'. The desk agent's account of its calls
  matches the app's log.

## 3. The boundary, from the desk (`record/boundary-probes.txt`)

| Probe | Answer | Who refused |
|---|---|---|
| POST to a listed check, POST /replay | 405 "support can only read" | the app: svrngs carries a member's POST (API §2, O7) |
| Another viewing session (`/timeline?attempt=1`) | 400 | the app |
| A route not listed (`/attempts`) | 404 | the app |
| `/incident` after **End support access** | 410 | the app |
| `/incident` at the lend's address after `svrngs unshare` | 404, `x-svrngs-refused: unlent` | svrngs |
| The app restarted on the port | 502 `away` until the customer approved it (`record/customer-approve.txt`) | svrngs (share binding) |

The probes ran after End and unshare, not before. I forgot `chmod +x`, so
the first try ran End and unshare but no probe. The lend was then shared
again for the probes. The scope probes don't depend on End: `app.mjs`
refuses a method, a route or a parameter before it checks End.

## What this run does not show

- **The link isn't the support link.** It's a lend to a lab group. Any
  member could read it, the desk is written into the group's record, and
  read-only is the app's rule, not svrngs's. Those are the API doc's O3–O7,
  and they're why its §3–§8 build exists. A forwarded link was not tried.
- One device, one loopback connection, one machine for both sides. The
  proposal's real-home repeat isn't done.
- The hang is the substitute incident. The proposal's 404 heals itself on
  this player.
- `/incident`'s `froze_at` floors 761.96 s to "12:41". `/replay` lists only
  the segments from 758–766 s; the desk agent asked why.

## Run it again

    ./make-fixture.sh                  # media/ and vendor/, git-ignored
    node origin.mjs hang out/x/origin.log & node app.mjs out/x &
    node drive.mjs out/x play help fix again && node qualify.mjs out/x

The two-node part is in this README's §2 commands, with the `HOME` and
`SVRNGS_DIR` of each node under `out/lab/`.
