# si-1: Jellyfin through the sign-in, again — 2026-10-08

si-1's row (`ralph/next/compose/STATE.md`) asks mwp-3's Jellyfin run
(`demo/runs/mwp/2026-10-08/run.sh`) green again once the sign-in is
decided at one site (ae898f0, #87), asked for each person apart (effb427,
#88) and its stand-in named in the door's ask (6028168, #90). `run.sh`
here is mwp-3's sequence (`apps/jellyfin/check/check.sh mw2`) split into
its steps, so each stays inside one ten-minute call, writing here and not
over mwp-3's record. Scratch Jellyfin 12.0.0 at `/tmp/si1b`
(`steps.log`, from 20:36 UTC); svrngs and the middleware at 6028168 with
`apps/jellyfin/check/mw.py` as committed beside this README; Playwright
1.59.1. One host, one run: not a rate (AGENTS.md 7).

One change to the check, beside this run: `mw.py`'s probe of the
middleware's port "as the door asks it" now names the stand-in as the
door does. A first attempt without it (`/tmp/si1`, 20:35 UTC, not
recorded) read 403 there where mwp-3 read 200 — the middleware refusing
an ask with no `x-svrngs-stand-in`, not anything a process on the
lender's computer cannot add. With it: 200 and a secret answered, as
mwp-3's (`steps.log`, "recorded"), the loopback trust DESIGN §9 keeps.

## What passed, as mwp-3's

- **mw1-again** (`mw-mw1-again.json`): every bar passed — a user each,
  made by the first ask; a second ask makes none; each person's own
  device; neither receives the lender's key; no key refused; body, range,
  page and socket carried.
- **The three plants**, each caught (`pair-mw-computer-chromium-plant-*`):
  no-sign-in, "no sign-in" failed; lender-key, "no lender's key" failed
  (`mw_key` in `/web/`); one-device, "apart" failed.
- **Bar 1's sign-in, all four pairs**: nothing typed, each opened
  signed in as themselves, the stored token the stand-in; no lender's
  key, password or the other's token in any byte either received
  (32 to 87 MB searched a pair); devices apart.
- **The door's sign-in traces** (`/tmp/si1b/sam/run.log`, info): each of
  the six people asked once a sign-in (`requests=1`) — Alex and Bo six
  times each, after five 401s each forgot them (`check.sh` `mw_forget`
  before each computer pair, as mwp-3's ten).

## What failed: ben's watcher, in WebKit, also before si-1

`check.sh judge` (`steps.log`, the end): **not all passed**.

| pair | seeks (ms) | ben while ana seeked |
|---|---|---|
| computer, Chromium | 393, 404, 419, 383, 382; 444, 437, 410 | left alone |
| computer, WebKit | 410, 618, 521, 524, 403; 404, 406, 430 | **new element 1**, back 0 s, still 0 ms |
| phone, Chromium | 381, 1,775, 1,175, 1,181, 2,073; 1,761, 383, 1,070 | left alone |
| phone, WebKit | 402, **5,371**, 403, 401, 405; 1,242, 1,292, 1,222 | **new element 1**, back 0 s, still 0 ms |

- *Left alone*, in both WebKit pairs: while ana seeked, the watcher
  (`apps/jellyfin/check/person.mjs` `watch`) bound a second `<video>` in
  ben's page, yet his film never stopped or went back (no `emptied`, no
  `loadstart`, back 0 s, still 0 ms). No WebKit pair on record before
  this run counted one (`grep -rE "new element [1-9]" demo/runs`, only
  this run's lines). Reproduced, the WebKit computer pair alone
  (`probe/probe.sh`, outside the record, then copied to `probe/`): at
  6028168 2 runs, 1 counted one (`head-1.json`, `head-2.json`); at
  37b4a4c, before si-1's first commit, built in a worktree, 2 runs, both
  counted one (`before-1.json`, `before-2.json`). So it is not si-1's;
  what the second element is, and why it began today, is not known — the
  watcher records a count, not the element.
- *Seeks within 5 s*, the phone in WebKit: Cleo's second, 5,371 ms. mwp's
  bar 1 predicted "one phone seek in WebKit perhaps missing"; mwp-3's
  slowest was 4,008.

**Judged by the bar's spirit** (bars by their spirit, the queue's fence
4): bar 1 guards that each person is signed in as themselves with
nothing typed, sees no secret, and has their film left alone by the
other's seeks. The sign-in's part passed in every pair and each plant was
caught; ben's film was never started over by its own measures. The
letter failed on a count that fails the same way before si-1. Follow-up,
not done here: the watcher records what each new element is (its source
and whether it plays), so the next run tells a second player from an
instrument's false count.
