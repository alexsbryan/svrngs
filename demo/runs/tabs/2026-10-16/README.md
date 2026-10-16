# groups-tabs: the group's page in two tabs of one browser (2026-10-16, this host)

Row groups-tabs (`ralph/next/groups/STATE.md`); ralph/REVIEW_FINDINGS.md
#176, which arch-stream inferred and did not run
(`demo/runs/stream/2026-10-15/README.md`, 5). One host, one run in each
engine: not a rate (AGENTS.md 7).

## (a) Two tabs, shown

`cargo test -p svrngs-node --test browser -- tabs:: --test-threads=1
--nocapture` at c43d2cc7 with the test added (`tests/browser/tabs.rs`,
`web/tests/tabs.mjs`). Bo's key is held in one browser's store. Tab one
opens house's page by its link and lists it. Tab two then opens the same
link. For 60 s after, each tab in turn asks "What apps use" of ada's
shelf, which asks. That dials ada's computer through the tab's own
endpoint (`web/app.js` `uses`, `dialler`), as a person's tap there does.

**It reproduces, in both engines, and worse than #176 inferred**
(`a-red.out`, `a-summary.txt`):

| engine | tab one's asks answered | tab two's asks answered |
|---|---|---|
| chromium | 0 of 9 | 9 of 9 |
| webkit | 0 of 9 | 9 of 9 |

- Both tabs said "You’re in house, as bo" at the start and at the end.
  Neither tab says anything is wrong.
- Every ask in tab one said "ada’s computer cannot say now what apps use
  ada’s shelf (k). ada’s computer did not answer.", 4.6 s after it was
  asked (`dial::DIAL_WITHIN`, 4 s, then the page). Its trace reads "not
  answered in Some(4s): asleep" each time. Ada's computer was awake: tab
  two reached it a quarter of a second later, every time.
- The relay logged "multiple connections found, deactivating old
  connection" once per engine, for bo's key `ea4a6c63e2`, when tab two
  bound its endpoint (`a-relay.log`; iroh-relay-1.0.2
  `src/server/clients.rs:80-92`). From then on, the relay sent every
  answer for bo's key to tab two's endpoint.
- **No retake in 60 s.** arch-stream saw the page's endpoint take its key
  back 16-24 s after a second bind, under load. Here the first tab never
  did within the minute, so it stayed set aside for the whole run.

So the tab a person opened first silently stops reaching anyone. Every
computer it dials is then said to have not answered, and named by its
person, while those computers are on.
