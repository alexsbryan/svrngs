# Parity: every v1 test on v2

`scripts/parity.sh` at 668dbb66, which the loop ran in the page-parity lane as
`ralph-result await -- bash scripts/parity.sh`. It finished in 8 min 34 s on
2026-11-01 and exited 0:

    parity: 142 passed (39 of them passed with no v2, in the baseline), 0 failed, 0 could-not-judge, 0 never-ran

`v2.tsv` is that run's `target/parity/verdicts.tsv`, copied as written. Each
target's `test result` line:

| target | passed | ignored |
|---|---|---|
| browser | 92 | 2 |
| conformance | 7 | 1 |
| scenes | 14 | 0 |
| install | 11 | 0 |
| leave_page | 6 | 0 |
| relay | 5 | 0 |
| home_group | 5 | 0 |
| join_page | 2 | 0 |
| phone_dry | 0 | 1 |

The baseline's two never-ran tests
(`a_phone_s_socket_opens_again_once_its_lender_answers_again::webkit` and
`the_home_page_makes_a_link_to_invite_someone::chromium`,
`../2026-10-31-baseline/README.md`) both passed here.

## The 39 passes with no v2

These 39 tests also passed on the empty pages
(`../2026-10-31-baseline/verdicts.tsv`). For 38 of them, no row earned the
pass, because none of them opens a v2 page. They do not count toward parity.
They still run on v2, and check.sh still runs every one of them.

- **No page at all (16):** install's 11 and relay's 5
  (`../2026-10-31-baseline/README.md`).
- **The API alone (2):** leave_page's
  `a_leave_reaches_a_computer_off_as_it_landed` and
  `a_cut_by_a_handed_on_keeper_reaches_a_computer_off_as_it_landed`. Each
  reads `/v1/groups` from a node, and no browser opens
  (`crates/svrngs/tests/leave_page.rs:441`, `:573`).
- **Pure judges (6):** browser's `film_seeks_by_load::*` (3) and
  conformance's `seeks_by_load::*` (3). These judge recorded seek times
  against the load (`browser/main.rs:898`, `conformance.rs:307`).
- **A lend's own origin, with no page served (2):** conformance's
  `on_a_computer::chromium` and `::webkit`. In `computer` mode,
  `web/tests/conformance.mjs:33-34` serves no site and opens each lend at
  its loopback origin.
- **The apps' own pages and the API (7):** the scenes films, give,
  give_gone, give_move, grapevine and pantry on chromium, plus
  `redial::restarted::alone`. Each joins by a link through the command and
  drives the app's page or `/v1/*`. pantry and redial open no browser at all
  (`scenes/pantry.rs:14`, `scenes/redial.rs:1`).
- **question.js run by node (3):** home_group's
  `the_command_and_the_page_say_a_name_held_already_in_one_wording`
  (`home_group.rs:364`), and browser's
  `acts::what_acts_as_you_is_said_in_one_wording` (`acts.rs:24`) and
  `a_lends_alias_never_collides_within_a_group` (`main.rs:2116`). v1 and v2
  both import that one module.
- **v1's source text (2):** browser's
  `the_index_s_row_is_judged_once_for_both_pages` (`main.rs:486`) and
  `opens::the_open_says_what_an_app_does_in_one_wording` (`opens.rs:133`).
  Each reads `web/app.js` and `crates/svrngs-node/src/home/home.js`, which
  are v1's files, so on v2 they still judge v1. v2 draws its rows through
  question.js's `opens` and `agreed` too (`web/v2/index.js:41`,
  `crates/svrngs-node/src/home/v2/index.js:33`). But `web/v2/index.js`
  calls `opens(l)`, while the test looks for `opens(l, `. When the
  switchover deletes v1, these two tests have to point at v2's files, or
  they fail on a missing file.
- **The published page's policy (1):** browser's
  `the_published_page_allows_no_loopback` (`main.rs:77`). It reads `site()`,
  which on v2 is the v2 build (`harness.rs:213-219`). So it does judge v2,
  and it passed on the empty frame because the policy comes from the head
  that v1 and v2 share. page-scaffold earned it when it kept that head
  (`web/v2/body.html` has no `<head>` of its own).

## The count beside the bar (f)

The counts below are from `scripts/lean_check.py` at d3d255c6: `.` is at
19,121 of 19,122 and `v2` at 1,189 of 1,207. The switchover's bar is `.`
at or under its cap once v1 is deleted. CHARTER's 19,481 was that cap
before 963dcdac stopped counting the diagrams' generators, so today the
bar is 19,122.

- **S, v2's lines that stay: 1,076 to 1,079.** That is the rung's 1,189,
  less `scripts/parity.sh` (37), `home/v2/serve.rs` (62) and
  `web/v2/build.sh` (19), plus the 5 to 8 lines of build.sh that fold into
  build-web.sh (copying `v2/`, `ui/` and `vendor/`, and the `SHELL` line).
  CHARTER estimated 770 to 1,042, so S is 34 to 37 over the top of that
  range. Per path, counted by lean_check's own rules: `web/v2/` 503 less
  build.sh, `home/v2/` 470 less serve.rs, `home/ui/` 164, `web/vendor/` 3
  (each vendored file is one line).
- **T, home.rs's table of v2's files:** +8 to +30, as CHARTER has it. There
  are 31 modules under `home/v2/` and `home/ui/`.
- **H, the both-live hooks the switchover deletes:** 13 to 16. That is
  the 14 in STATE's page-scaffold row, plus `page v2` and `parity`, +1
  each. The low end applies if check.sh's `page laws` (+3) stays.
- **D, v1 deleted:** 703.

So `.` after the switchover = 19,121 − H + S + T − 703, which comes to
**19,486 to 19,514**. That is **364 to 392 over** the cap of 19,122, and
past CHARTER's 59 to 353. On the current base, `.` breaks even only if
S + T is at most about 717 to 720.

**Sizing the switchover's pass.** These are the candidates the code shows:

- **Imports: 185 of the 1,076 lines** (`grep -c ^import` over `web/v2`,
  `home/v2` and `home/ui`). The 45 modules were split by who owned each
  flow, so that rows could run in parallel (CHARTER, "Who owns each
  file"), not by need. Folding each page's flows into a few modules
  could cut 100 to 130 lines. It would also shrink T, since there would be
  fewer modules to table.
- **POST helpers, four of them** (`home/v2/door.js`, `leave.js`,
  `running.js`, `shelf.js`): one helper would cut 3.
- **Duplicated primitives** that fence 8 put in each row's own files, for
  the pass to fold. None was found in a single read, beyond the POST
  helpers.

Past passes cut 3 to 93 lines (CHARTER). This pass's import fold is
larger than any of them, and it has not been tried. A pass that does
everything above lands at about 19,350 to 19,410. That leaves an
expected raise of **about +230 to +290**, which is the operator's, set at
HUMAN-page-switchover after the pass.
