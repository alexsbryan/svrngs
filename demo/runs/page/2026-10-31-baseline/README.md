# The baseline: parity on the empty pages

The first run of `scripts/parity.sh`, at e5f682df, where both v2 pages are
page-scaffold's empty frames and stubs (ralph/next/page/STATE.md, page-switch
(b)). `verdicts.tsv` is its `target/parity/verdicts.tsv`, copied as written.
`parity.sh` reads it from here: a pass that this run also had is counted apart,
since v2 did not earn it.

The loop ran it as `ralph-result await 3h -- bash scripts/parity.sh`, from
18:12 to 19:07 on 2026-10-31 (56 min). It exited 1, as it must while v2 is
short:

    parity: 39 passed (0 of them passed with no v2, in the baseline), 101 failed, 0 could-not-judge, 2 never-ran

That line reads 0 because no baseline existed yet. All 39 passes here are passes
with no v2:

- install's 11 and relay's 5 open no page.
- The other 23 are in browser (8), conformance (5), scenes (7), leave_page
  (2) and home_group (1). `grep ^passed verdicts.tsv` names each one.
  This run does not say why each one passes on an empty page.

## The two never-ran

`browser`'s binary got TERM at 18:52, 40 min in, and printed no
`test result` line (`target/parity/browser.log`'s last lines:
"process didn't exit successfully ... (signal: 15, SIGTERM)"). By then 90 of
its 92 listed tests had a verdict. These two had been running since about
18:13 and never ended:

- `a_phone_s_socket_opens_again_once_its_lender_answers_again::webkit`
- `the_home_page_makes_a_link_to_invite_someone::chromium`

`parity.sh` has no step limit of its own and did not send the TERM. Where it
came from was not found. Both tests wait on a page with no limit, so on the
empty pages they hang rather than fail. Until page-parity makes `parity.sh` a
step of `scripts/check.sh`, so its limit applies, a run of `browser` on an
unfinished v2 can hang the same way.
