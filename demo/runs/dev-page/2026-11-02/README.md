# The page served for a person working on it, 2026-11-02

Does `node web/tests/dev.mjs` give a person working on `web/` a page where an
app opens and saves? At 18bc628f with the change uncommitted (6dafd336, which landed during
the runs, changes RELEASES.md alone), on Darwin
27.0.0, node 20.20.2, Google Chrome 151.0.7922.174 driven by Playwright
1.59.1 (`channel: 'chrome'`, headless), the page from `scripts/build-web.sh`
and `target/debug/svrngs` built at that commit, n0's relay.

`run.sh` does what the printed text says: ada's computer makes house, carries
the approval test's Money (carried.rs
`an_app_s_entry_is_approved_by_the_group_s_signer_and_carried`), and points
it at the page with `svrngs link house --origin`. `drive.mjs` opens Chrome
with the printed line's flags (Playwright makes the profile directory, and
the link is opened in place of the bare page), joins as bo by typing the
name, opens Money from the group's page, saves an entry, and reads ada's
`/v1/rail/log`.

- `run-tls.txt` (`run.sh`): **passed.** `--origin https://group.svrnme.test:<port>/`
  is not judged (exit 3: nothing but that Chrome reaches the name);
  `--origin http://localhost:<port>/` is taken. At the https address bo
  joins, Money opens at `<label>.group.svrnme.test`, the signer adopts the
  open's approver and deletes its cookie, the save answers 200
  `counted: true`, and ada's log holds the entry once, as bo's.
- `run-plain.txt` (`run.sh plain`), the plant: the link opened as svrngs
  printed it, at `http://localhost:<port>/`. **Failed, as it should:** bo
  joins, but Money never opens. Its page can't frame the signer at
  `http://localhost` (`frame-src https:`), so "the group page did not verify
  this open", nothing is saved, and ada's log holds nothing.

Not run: Safari or WebKit, which give the framed signer no cookie on any
local name (`demo/runs/security/2026-11-02-webkit-partition/`); Chrome on
Linux (`google-chrome`, the line dev.mjs prints there); the line pasted by
hand into Terminal (`open -na`).

    lockf -k /tmp/svrngs-gate.lock demo/runs/dev-page/2026-11-02/run.sh [plain]
