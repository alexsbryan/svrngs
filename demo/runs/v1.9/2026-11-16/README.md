# V1.9 release, 16 November 2026

Released at the operator's word, 2026-11-16: "Let's do the whole check,
then commit, push, release".

## Scope and source evidence

The release before this one is V1.8.2 at d2510734; its readback is
`../../v1.8.2/2026-11-15/README.md`. Before this run, `group.svrnme.sh`
was still serving V1.8.2's page: at 2026-11-16 23:52 UTC its `sw.js`
named version `e022fd11ba8f`, V1.8.2's own. Since `v1.8.2`, the shipped
files that changed are the home page (`crates/svrngs-node/src/home/`: its
`page/` and `ui/` files, `question.js`, `look.css`, `index.html`,
`session-worker.js`) and, of the phone's page, `web/authority.js`,
`web/sign.js` and `web/page/`'s `enter.js`, `visit.js`, `dial.js` and
`page.js`. The phone's page also draws the home page's `ui/shelf.js`. All
other changes are tests, guides, docs, the lean check and records.

| Release note | Source or run |
|---|---|
| A group's page opens on People, then Shared apps and Shared files, then visits and the invitation buttons | 11600759, 1772f6ce, `crates/svrngs-node/src/home/page/grouped.js` `GroupPlaces`; 12e5681c (People open at first), b39afb9d (inviting out of "Manage this group"); `web/tests/pages/home.test.mjs` 'a group's page is its banner, what needs you, its people, what they share, then inviting' |
| One line a person, 28 people in four rows | 03f99b31, 49926552, `home/look.css`. Seen in Chromium at 1280x800 with a 28-person fixture, not a law |
| A group's card says who is on and what is shared | 12e5681c, `grouped.js`; a7644309 (who you are, said once above the cards); `home.test.mjs` 'a group's card on the overview says who is on and what is shared, or that it cannot be read' |
| Pressing a person shows how they stand, when they joined and what they share | b89c49e8, `home/page/people.js` `Person`, `home/ui/door.js`; `home.test.mjs` 'a person in the door is pressed for what the page holds of them, …'; `web/tests/pages/members.test.mjs` holds the phone's door as it was |
| Removing someone is two presses, or an ask; "Done in a terminal" is gone; `svrngs remove` still works | 95acdd8c, `people.js`, `home/page/leaving.js`, `home/index.html`. The page posts the ask the command posts, `POST /v1/removals`: no route and no Rust is added. `home.test.mjs` 'removing a person is two presses in their popover, and every answer is said' and 'nothing the page does is left to a terminal'. Seen against stubs only, never a real group |
| Shared apps are grids of cells, the name the link, the sentence said once above | f1ec0354, `home/page/index.js` `says`, `Grid`; bb70a06c, c1b1f7b6, 4341574c (the computers not listed, one line, each named); 219cf6da, `docs/APP_ACCESS_AND_STOP.md` A1; `home.test.mjs` 'the words outside the index's cells are the same for 8 things and for 40' |
| Each caption says where its things run | e9931c6e, `index.js` `says`, `LENT`, its wording verbatim; the caption laws in `home.test.mjs` |
| Address, with Copy, "Open it to this WiFi" and Close | c2a0a505, ef27cd12, `home/page/wifi.js` `Address`; `home.test.mjs` 'a lend's address for a program is kept behind Address in its cell, and its Copy writes it and says so'. The fold "Addresses and WiFi" (b39afb9d, 69ec24a5) came and went between the two releases, so the notes do not name it |
| What you lend is a line a lend | ad8958e7, `home/page/lent-row.js`; `home.test.mjs` 'each lend is one line with its form folded under it, and the wide button is the phone's' |
| Add an app says what an app is; svrngs's own are examples, on the phone too | 22855229, c4c58ac0, `grouped.js`, `home/ui/shelf.js` (the phone's too), `home/page/shelf.js`; `home.test.mjs` 'Add an app leads with what an app is, then svrngs's examples, then the other way'; `web/tests/pages/shelf.test.mjs` |
| Screen readers and keyboards | 40673c4d, e315d5d8; `home.test.mjs` 'each way in is described by its caption, and each button is named with what it acts on' and 'a Stop under What acts as you is named with the app it stops'; `web/tests/grouped.mjs` walks a popover, a person and a removal by keyboard. Chromium only |
| Groups, lends and invitations work as in V1.8.2 | `git diff v1.8.2..HEAD --stat -- '*.rs'`: `crates/svrngs-node/src/home.rs`, one line, the list of home page files it serves (493e12d0, a9447fc8, 40673c4d and the folds before them), and four test files. No route, no act kind and no record format changes. `git diff v1.8.2..HEAD -- scripts/trust_floor.txt` is empty. |
| Small repeats said once | 5d68ed3f (your own guest listed once; groups with the same deciders in one line), 7abcb46c (one rule above a group's banner), `home.test.mjs` 'what acts as you names groups with the same deciders in one line, and heads no empty group'. Too small for the notes |
| Shipped files moved, behaviour kept | 493e12d0, a9447fc8 (home page parts drawn in the file that uses them), 59390cee (`home/session-worker.js`), 8522068c, d9963f77 (the phone's page) |
| Nothing else changes | The other commits since `v1.8.2` change tests, scenes, guides, the typed lint and the JavaScript cap only: 5cf5b345, 7af56ff5, 9b2c92ec, 1410e3e4, and the cap's 003b72b9, 1d13bdc4, b7048aa6, b198c990. The cap was raised once, by the operator (003b72b9), and stands at 8,545, under the 8,552 it stood at for V1.8.2 |

## Validation before packaging

`scripts/check.sh` ran whole on the Mac at c4c58ac0, under the gate lock:
exit 0, 45 steps passed, none failed, could-not-judge or never-ran, 1,303
s (`gate-c4c58ac0.txt`). Its `rate` line is a count, not a step: one
WebKit seek straight at the canary, not through a lend, took 5.7 s.

The release commit, tagged `v1.9`, changes no runtime file: only
`RELEASES.md` and this record. So the gate at c4c58ac0 stands for it, and
the whole check was not run again. What that commit can reach was run
with it in the tree: `scripts/check.sh --touched`, under the gate lock,
exit 0 in 127 s (`touched-prepare.txt`). It measures from the upstream,
`v1.8.2`'s readback commit, so it reached more than this commit does. Its
cheap steps passed, the principle pins, lint and format among them, with
clippy, types and the typed lint; `tests` and `app laws` said
could-not-judge for the fragile tests the quick tier never runs, all of
which ran in the whole check.

Not run on Linux: V1.8.2's whole Linux run (`gate-3da19985-linux.txt`) is
the last one, and this release has none.

`rustup target list --installed --toolchain 1.95.0` lists all four
targets `scripts/build-release.sh` builds.

## Publication

`publish.py` ran `scripts/deploy-page.sh` under the gate lock in a clean
detached worktree at 16b7eb46 (`v1.9`), with a target directory of its
own, from 2026-11-16 23:58 to 2026-11-17 00:05 UTC, exit 0
(`publish.json`, `publish.log`). Load was about 3.8 at the start, with 89
GiB free. The record was written outside the worktree and copied here
after. The build meets its groups at n0's relays (`SHIPPED`), so no relay
was asked. The page built as version fb5ab80cf0a6, and `svrngs` and `give`
built for all four platforms. Vercel deployment
`dpl_8A7h2fMpfw3ziaqJ4oVCDuYwyeL4` reached production as Ready, aliased to
`https://group.svrnme.sh` and `https://*.group.svrnme.sh` (`vercel inspect
svrngs-page-huntrwfsz-svrnmesh.vercel.app --scope svrnmesh`). The CLI
wrote `.vercel/`, `.env.local` and `.gitignore` into the upload
directory. All three answered 404 live and were deleted after.

## Readback

`verify.py` passed at 2026-11-17 00:05 UTC (`verification.json`). Its six
planted faults were rejected first. Then:

- 78 public files fetched from `group.svrnme.sh` each equaled the built
  site's bytes.
- The live `get/SHA256SUMS` equaled the built one (`SHA256SUMS`,
  `live-sums.txt`) and named exactly the eight downloads.
- The live page is version fb5ab80cf0a6, not V1.8.2's e022fd11ba8f, and
  the live `sw.js` names it and sends a navigation to the network first.
- The live `install.sh` equals the built one.
- Both frame policies were as `vercel.json` sets them.
- `release-16b7eb46.group.svrnme.sh` served the same page.
- `.env.local`, `.vercel/project.json` and `vercel.json` answered 404.
- The downloaded darwin-aarch64 `svrngs` and `give` started.

No other platform's program was run here, and no home page was opened
from the published build: the home page's own evidence is the gate at
c4c58ac0.

## Not verified, not done

- Safari with VoiceOver, and Firefox, were not run. What the notes say of
  screen readers and keyboards was checked in Chromium.
- The Stop beside "what apps use" (`home/ui/uses.js`) is still named
  "Stop" alone.
- The Join, Make and Open views have no first heading.
- Layout A (`/?ui=a`) remains.
