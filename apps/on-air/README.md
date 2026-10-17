# On air

The proof of concept for svrn-docs `rings/current/ON_AIR.md` §7: an evening
on air to a group's computers and to a stranger's browser, built on three
primitives an app gets from svrngs (bytes by hash with a lifetime, a
signature for an app, the live lane naming who spoke). It is a test of the
infra, outside the size budget (`scripts/lean_check.py:14-16`).

- `station.mjs`: cuts an MP3, puts, signs, pushes and publishes each piece,
  and a signed `ended` last.
- `listen.mjs`: on a member's computer, drains the lane, gets and checks
  each piece, and appends `heard.mp3`.
- `page/`: the stranger's page, which checks before it plays. `page/air.mjs`
  is what all three share. `page/vendor/` holds `@noble` (see its README).
- `laws.test.mjs`: air.mjs's laws, run by `scripts/check.sh`.
- `harness/run.py`: the run, on one host, with each bar judged. Its runs are
  `demo/runs/onair/`.
- `nouns.txt`: the words svrngs's code may not use.
