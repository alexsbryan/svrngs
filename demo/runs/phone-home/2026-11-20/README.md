# HUMAN-phone-home: a real phone, direct at home and relayed after (2026-11-20)

ralph/next/relay-commons/STATE.md `HUMAN-phone-home`. The operator's phone (WebKit), on
the page built from main at 2ef37e98 (`scripts/build-web.sh`, deployed to the
staging project svrngs-next at next.svrnme.sh, `*.next.svrnme.sh` for the lends), and
a lender of the same build on a computer on the home WiFi: `svrngs new phone-home
--origin https://next.svrnme.sh/ --relay relay.svrnme.sh`, a 10-minute 0.67 Mbit/s
film (ffmpeg testsrc2, 50 MB) lent by `svrngs share` from [film-server.py](film-server.py)
on 127.0.0.1, and direct connections turned on for that WiFi (`POST /v1/direct`).
The lender ran with `RUST_LOG=debug`; [trace.log](trace.log) is its path and carriage
lines, the router's hardware address and every IPv4 address redacted.

| The row's claim | | The trace |
|---|---|---|
| On an enabled network, it goes direct | passed | 05:36:45 `an offer answered, its peer on a network the lender enabled`, `a data channel opened`; 2 MiB ranges answered every ~0.5 s, past the relay's 1 Mbit/s share |
| WiFi dropped mid-stream, it falls back to the relay | passed | 05:37:14 the WebRTC path `Abandoned { reason: TimedOut }`; the range asked at 05:37:25 sent over the relay at about 80 KB/s (`ms=26724`); 05:37:35 `a peer ended ... ice disconnected` |
| On a network not enabled, it stays on the relay | passed | 05:47:51 `an offer refused why=direct connections are not on for this WiFi`; nothing bound on the LAN; every request `relayed=Some(true)` |

## What it found

1. **No way back to direct.** Back on the WiFi and reloaded, the phone made no offer
   (05:41-05:44: no `webrtc` line). `web/lend.js` `direct` returns when `directs`
   holds the lender, and only `letGo` removes it, so the dead attempt stands until
   the lend worker restarts: a fresh worker (the tab closed a minute) offered at
   05:47:51.
2. **A seek over the relay waits behind the lookahead.** The film's 2 MiB ranges at
   several places in it rode the relay at once (05:42:48-05:43:29, each 8-10 s), so a
   seek past the buffer started about 60 s later, the operator reported.
3. **The lender's folder sync woke every 51 ms** ("synced: the step woken, the OS saying
   files changed", `files=0`) with nothing in the folder, the whole run.

The first seek (05:38-05:40) is not counted: relay-load's second ramp held the relay's
box at up to 129% CPU then (svrn-docs rings/current/relay-load).
