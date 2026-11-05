# Grouped home B — reuse and capability proof

This is the native browser's opt-in presentation; no membership, lending,
signing, authentication or phone behavior changes.

## Economy and inventory

The pre-edit base was `0f2a2e02`: `python3 scripts/lean_check.py` passed at
3,486 core, 19,789 non-JavaScript and 8,066 JavaScript lines. The caps are
exclusive: only 23 additional JS lines fit below 8,090. The separate +98 cap
increase and its boundary test are recorded in `BUDGET.md`.

Inspected before adding a module:

- `home-page.js` composes every group and home place. Reuse that composition
  and keep it mounted, hiding unselected presentation. Unmounting groups
  would starve `acting.js`'s `groups.every(told)` readiness rule.
- `group-read.js::PLACES` and existing feature components own every group
  capability. B groups those same slots; no app row, form or mutation drive
  is copied.
- `home-reading.js::first` parses root query destinations. Ordinary links
  need no SPA router, preference store or extra listener. Making and joining
  must retain B's address fields through their existing reread.
- `home-page.js::Arrival` and `group-folder.js::Folder` own the welcome and
  guarded OS opener. Add continuation links, not another arrival or folder API.
- `acting.js`, `ui/changes.js`, `running-here.js` and `what-you-lend.js`
  provide computer controls and version consent. Reuse the changed-version
  answer for the optional workspace review notice.
- Upstream `commonwealth-ai/cmnwlth/apps/ring-runtime/web/index.html`
  (inventory HEAD `d8b61abe`) is the guest connection surface, not a multi-group
  native home. It supplies no navigation module that can replace this
  composition; no upstream code was copied.

Economies selected: one set of feature components/read machines for A and B;
native links/history and disclosures; existing query parsing and version data.
No supported deletion in the pre-existing product was found. Merging GROUP
with HOME or removing background reads would lose distinct ownership/readiness;
copying controls would duplicate policy. Removing A would prevent the requested
comparison. No packing or formatter exception funds the build. The remaining
addition is presentation, destinations and typed view helpers. The net growth
is 121 JavaScript lines and one non-JavaScript serving entry.

## Capability destinations

| Existing capability | B destination |
|---|---|
| App open, opening-time consent, sign-in, raw Play | Unchanged group app index |
| Every shared folder's actual path and opener | Group's primary area |
| People/presence, guest Stop, visitors, removals | People disclosure |
| Shelf apps, bring result/refusal | Add an app disclosure |
| Short invitations/closure, ordinary invites, devices, guests | Manage this group |
| WiFi open/close, program address copying, group screen | Manage this group |
| Keeper handover and leave, explanations/refusals | Manage this group |
| Versions, app/device permissions and Stop, WiFi/Play ledger | This computer: What acts as you |
| Running-program lookup and lending | This computer: Already running here |
| Capacity, sign-in setup, secrets shown once, stop lending | This computer: What you lend |
| Join, create, open an app link, departed-group record | Named navigation destinations |
| Removal's terminal recipe | This computer |
| Native welcome and folder recovery | Existing welcome, then Continue to stable group id |
| svrngs update/refusal/recovery | Existing page-top update component |
| Phone, consent questions, presentation screen | Existing independent surfaces |

## Watched failure and browser proof

The new native browser test initially ran against A's unchanged source:

```text
lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser grouped_home_keeps_capabilities_and_group_identity -- --nocapture
TimeoutError: waiting for getByRole('navigation', { name: 'Your groups' })
test result: FAILED. 0 passed; 1 failed
```

The expanded test uses the actual authenticated native shell and a synthetic
three-group answer, two groups sharing a name. It inventories A independently
of B's slot manifest, then opens each B destination and compares original
interactive elements, including names, hrefs and form field identities. It
records folder writes and checks the stable group id and exact path. Other
mutations stay at a recorded fixture boundary; this is not an end-to-end test
of the node's unchanged mutation rules. The original browser/scenes tests
remain those rules' witnesses.

The expanded run passes with 23 original controls per group, and home-place
inventories of 6 access, 2 open-link, 2 join, 3 create, 1 discovery and 5
lending controls. It exercises Back, refresh, visibility reread, welcome
continuation, retained B after making/joining, optional version review,
unknown targets/views (including prototype names), left/removed groups,
unavailable lends/groups, no groups, and narrow-width presentation.

One instrument failure was corrected: intercepting every POST also intercepted
authentication bootstrap, so no HOME read ran. The fixture now intercepts only
named product mutations and leaves session enrollment real.

These are technical capability proofs, not Xenia accounts or people results.
The dedicated multi-group model comparison has not run.

The independent inventory was also watched fail on a missing capability:
`GROUPED_HOME_PLANT=missing-wifi` replaces only B's presentation module in
the browser, omitting WiFi/program-address controls. A's inventory remains
unchanged and the comparison fails with those links/buttons missing. Without
the plant the same gate passes. No product source is temporarily changed.

## Full gate

`check.log` records `lockf -k /tmp/svrngs-gate.lock scripts/check.sh`, exit 1,
1,432 seconds. Every step except lean and browser passed. Lean measured
3,486 core, 19,790 non-JS and **8,187 JS**, against the then-current JS cap
of 8,090. The cap was subsequently raised to **8,188 (+98)**; see `BUDGET.md`.

The browser step passed 97 tests, ignored 2 and failed only the original
`the_home_page_lends_what_runs_here`. Its captured trace ends with
`LENDING: read: read -> reading, running read`, while the full-page snapshot
still contains the previous Jellyfin-only lending list. The driver had waited
for the existence of any `#lent h2 ~ *`, which the old list already satisfied;
the running-program row's success is not the lending reread's completion.
The test now uses the driver's existing `after` facility to await the expected
chores lending row before taking that snapshot, without changing the product,
its assertions or any timeout. Targeted/browser recheck evidence follows.

After the snapshot-wait correction, the full browser suite passed:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test --quiet -p svrngs-node --test browser
test result: ok. 98 passed; 0 failed; 2 ignored; finished in 144.45s
exit 0 (browser-recheck.log)
```

The final inventory additionally excludes invisible controls (except hidden
form fields, which are part of the action's identity). Its targeted run passes,
with the same counts above (`grouped-green.log`, exit 0). The missing-WiFi plant
still fails exactly on capability parity (`grouped-plant.log`, expected exit
101); it does not edit the product bytes on disk. A first attempt to fetch the
mutated module through Playwright's request client received 403 instead of
exercising the plant; the corrected plant fulfills the locally read module
bytes in the browser, leaving real session enrollment intact.

Final `oxlint`, `oxfmt`, `scripts/types.mjs` (130 files), and typed lint
(126 page files and 3 worker files) passed. `cargo build -p svrngs` passed;
`target/debug/svrngs` embeds B. The subsequent `closing-check.log` passes
through the page checks but is incomplete; it has no full-gate exit-0 verdict.
